use base_db::Change;
use crossbeam::channel::{unbounded, Receiver, Sender};
use fxhash::FxHashMap;
use ide::{Analysis, AnalysisHost};
use linter::spcomp::SPCompDiagnostic;
use lsp_server::{Connection, ErrorCode, Request, RequestId};
use lsp_types::{
    notification::{Notification, ShowMessage},
    InitializeResult, MessageType, ServerInfo, ShowMessageParams, Url,
};
use parking_lot::{RwLock, RwLockUpgradableReadGuard, RwLockWriteGuard};
use serde::Serialize;
use std::{env, sync::Arc, time::Instant};
use threadpool::ThreadPool;
use vfs::{FileId, Vfs};

use crate::{
    capabilities::server_capabilities,
    client::LspClient,
    config::Config,
    dispatch::{NotificationDispatcher, RequestDispatcher},
    from_json,
    line_index::LineEndings,
    lsp_ext,
    task_pool::TaskPool,
    version::version,
    Task,
};

// mod diagnostics;
// mod files;
// mod notifications;
mod progress;
mod handlers {
    pub(crate) mod notification;
    pub(crate) mod request;
}
// mod requests;

// Enforces drop order
pub(crate) struct Handle<H, C> {
    pub(crate) handle: H,
    pub(crate) receiver: C,
}

pub(crate) type ReqHandler = fn(&mut GlobalState, lsp_server::Response);
type ReqQueue = lsp_server::ReqQueue<(String, Instant), ReqHandler>;

/// `GlobalState` is the primary mutable state of the language server
///
/// The most interesting components are `vfs`, which stores a consistent
/// snapshot of the file systems, and `analysis_host`, which stores our
/// incremental salsa database.
///
/// Note that this struct has more than one impl in various modules!
pub struct GlobalState {
    req_queue: ReqQueue,
    sender: Sender<lsp_server::Message>,

    pub(crate) task_pool: Handle<TaskPool<Task>, Receiver<Task>>,

    connection: Arc<Connection>,
    client: LspClient,
    pub(crate) pool: ThreadPool,
    indexing: bool,
    amxxpawn_mode: bool,

    pub(crate) shutdown_requested: bool,
    pub(crate) config: Arc<Config>,
    pub(crate) config_errors: Option<serde_json::Error>,

    pub(crate) analysis_host: AnalysisHost,

    // VFS
    pub(crate) vfs: Arc<RwLock<Vfs>>,
}

impl GlobalState {
    pub fn new(connection: Connection, amxxpawn_mode: bool) -> Self {
        let client = LspClient::new(connection.sender.clone());
        let task_pool = {
            let (sender, receiver) = unbounded();
            let handle = TaskPool::new_with_threads(
                sender,
                num_cpus::get_physical().try_into().unwrap_or(1),
            );
            Handle { handle, receiver }
        };
        Self {
            client,
            pool: threadpool::Builder::new().build(),
            indexing: false,
            amxxpawn_mode,

            req_queue: ReqQueue::default(),
            sender: connection.sender.clone(),
            task_pool,
            shutdown_requested: false,
            config: Arc::default(),
            config_errors: Default::default(),
            analysis_host: AnalysisHost::default(),
            vfs: Arc::new(RwLock::new(Vfs::default())),

            connection: Arc::new(connection),
        }
    }

    pub(crate) fn snapshot(&self) -> GlobalStateSnapshot {
        GlobalStateSnapshot {
            config: Arc::clone(&self.config),
            analysis: self.analysis_host.analysis(),
            vfs: Arc::clone(&self.vfs),
        }
    }

    fn run_query<R, Q>(&self, id: RequestId, query: Q)
    where
        R: Serialize,
        Q: FnOnce(&GlobalStateSnapshot) -> R + Send + 'static,
    {
        let client = self.client.clone();
        let state_snapshot = self.snapshot();
        self.pool.execute(move || {
            let response = lsp_server::Response::new_ok(id, query(&state_snapshot));
            client.send_response(response).unwrap();
        });
    }

    // pub(crate) fn execute<F>(&self, job: F)
    // where
    //     F: FnOnce() -> T + Send + 'static,
    //     T: Send + 'static,
    // {
    //     self.pool.execute(job);
    // }

    #[allow(unused)]
    fn run_fallible<R, Q>(&self, id: RequestId, query: Q)
    where
        R: Serialize,
        Q: FnOnce() -> anyhow::Result<R> + Send + 'static,
    {
        let client = self.client.clone();
        self.pool.execute(move || match query() {
            Ok(result) => {
                let response = lsp_server::Response::new_ok(id, result);
                client.send_response(response).unwrap();
            }
            Err(why) => {
                client
                    .send_error(id, ErrorCode::InternalError, why.to_string())
                    .unwrap();
            }
        });
    }

    // /// Resolve the references in a project if they have not been resolved yet. Will return early if the project
    // /// has been resolved at least once.
    // ///
    // /// This should be called before every feature request.
    // ///
    // /// # Arguments
    // /// * `uri` - [Url] of a file in the project.
    // fn initialize_project_resolution(&mut self, uri: &Url) {
    //     log::trace!("Resolving project {:?}", uri);
    //     let main_id = self.store.write().resolve_project_references(uri);
    //     if let Some(main_id) = main_id {
    //         let main_path_uri = self.store.read().vfs.lookup(main_id).clone();
    //         self.reload_project_diagnostics(main_path_uri);
    //     }
    //     log::trace!("Done resolving project {:?}", uri);
    // }

    fn initialize(&mut self) -> anyhow::Result<()> {
        let (id, initialize_params) = self.connection.initialize_start()?;
        let lsp_types::InitializeParams {
            root_uri,
            capabilities,
            workspace_folders,
            initialization_options,
            client_info,
            ..
        } = from_json::<lsp_types::InitializeParams>("InitializeParams", &initialize_params)?;

        let root_path = match root_uri.and_then(|it| it.to_file_path().ok()) {
            Some(it) => it,
            None => env::current_dir()?, // FIXME: Is this correct?
        };

        let mut is_visual_studio_code = false;
        if let Some(client_info) = client_info {
            tracing::info!(
                "Client '{}' {}",
                client_info.name,
                client_info.version.unwrap_or_default()
            );
            is_visual_studio_code = client_info.name.starts_with("Visual Studio Code");
        }

        let workspace_roots = workspace_folders
            .map(|workspaces| {
                workspaces
                    .into_iter()
                    .filter_map(|it| it.uri.to_file_path().ok())
                    .collect::<Vec<_>>()
            })
            .filter(|workspaces| !workspaces.is_empty())
            .unwrap_or_else(|| vec![root_path.clone()]);
        let mut config = Config::new(
            root_path,
            capabilities,
            workspace_roots,
            is_visual_studio_code,
        );

        if let Some(json) = initialization_options {
            if let Err(e) = config.update(json) {
                let not = lsp_server::Notification::new(
                    ShowMessage::METHOD.to_string(),
                    ShowMessageParams {
                        typ: MessageType::WARNING,
                        message: e.to_string(),
                    },
                );
                self.sender
                    .send(lsp_server::Message::Notification(not))
                    .unwrap();
            }
        }

        let server_capabilities = server_capabilities(&config);

        let result = InitializeResult {
            capabilities: server_capabilities,
            server_info: Some(ServerInfo {
                name: "sourcepawn-lsp".to_owned(),
                version: Some(version()),
            }),
            offset_encoding: None,
        };

        self.connection
            .initialize_finish(id, serde_json::to_value(result)?)?;

        // self.client_capabilities = Arc::new(params.capabilities);
        // self.client_info = params.client_info.map(Arc::new);
        // self.store.write().environment.root_uri = params.root_uri;

        // self.store.write().folders = params
        //     .workspace_folders
        //     .unwrap_or_default()
        //     .iter()
        //     .filter_map(|folder| folder.uri.to_file_path().ok())
        //     .collect();

        let _ = self.send_status(lsp_ext::ServerStatusParams {
            health: crate::lsp_ext::Health::Ok,
            quiescent: !self.indexing,
            message: None,
        });
        log::trace!("Server is initialized.");

        Ok(())
    }

    fn send_status(&self, status: lsp_ext::ServerStatusParams) -> anyhow::Result<()> {
        self.client
            .send_notification::<lsp_ext::ServerStatusNotification>(status)?;
        Ok(())
    }

    pub(crate) fn send_request<R: lsp_types::request::Request>(
        &mut self,
        params: R::Params,
        handler: ReqHandler,
    ) {
        let request = self
            .req_queue
            .outgoing
            .register(R::METHOD.to_string(), params, handler);
        self.send(request.into());
    }

    pub(crate) fn complete_request(&mut self, response: lsp_server::Response) {
        let handler = self
            .req_queue
            .outgoing
            .complete(response.id.clone())
            .expect("received response for unknown request");
        handler(self, response)
    }

    fn send(&self, message: lsp_server::Message) {
        self.sender.send(message).unwrap()
    }

    pub(crate) fn respond(&mut self, response: lsp_server::Response) {
        if let Some((method, start)) = self.req_queue.incoming.complete(response.id.clone()) {
            let duration = start.elapsed();
            log::debug!(
                "handled {} - ({}) in {:0.2?}",
                method,
                response.id,
                duration
            );
            self.send(response.into());
        }
    }

    /// Handles a request.
    fn on_request(&mut self, req: Request) {
        let mut dispatcher = RequestDispatcher {
            req: Some(req),
            global_state: self,
        };
        dispatcher.on_sync_mut::<lsp_types::request::Shutdown>(|s, ()| {
            s.shutdown_requested = true;
            Ok(())
        });

        match &mut dispatcher {
            RequestDispatcher {
                req: Some(req),
                global_state: this,
            } if this.shutdown_requested => {
                this.respond(lsp_server::Response::new_err(
                    req.id.clone(),
                    lsp_server::ErrorCode::InvalidRequest as i32,
                    "Shutdown already requested.".to_owned(),
                ));
                return;
            }
            _ => (),
        }

        use self::handlers::request as handlers;
        use lsp_types::request as lsp_request;

        dispatcher
            .on::<lsp_request::GotoDefinition>(handlers::handle_goto_definition)
            .on::<lsp_ext::SyntaxTree>(handlers::handle_syntax_tree)
            .finish();
    }

    pub(super) fn on_notification(&mut self, not: lsp_server::Notification) -> anyhow::Result<()> {
        use self::handlers::notification as handlers;
        use lsp_types::notification as notifs;

        NotificationDispatcher {
            not: Some(not),
            global_state: self,
        }
        .on_sync_mut::<notifs::DidOpenTextDocument>(handlers::handle_did_open_text_document)?
        .on_sync_mut::<notifs::DidChangeTextDocument>(handlers::handle_did_change_text_document)?
        .on_sync_mut::<notifs::DidChangeConfiguration>(handlers::handle_did_change_configuration)?
        // .on_sync_mut::<notifs::DidChangeWatchedFiles>(handlers::did_open)? // TODO: Implement this.
        .finish();

        Ok(())
    }

    fn next_event(&self, inbox: &Receiver<lsp_server::Message>) -> Option<Event> {
        crossbeam::channel::select! {
            recv(inbox) -> msg =>
                msg.ok().map(Event::Lsp),

            recv(self.task_pool.receiver) -> task =>
                Some(Event::Task(task.unwrap())),
        }
    }

    pub(crate) fn register_request(
        &mut self,
        request: &lsp_server::Request,
        request_received: Instant,
    ) {
        self.req_queue.incoming.register(
            request.id.clone(),
            (request.method.clone(), request_received),
        );
    }

    /// Registers and handles a request. This should only be called once per incoming request.
    fn on_new_request(&mut self, request_received: Instant, req: Request) {
        self.register_request(&req, request_received);
        self.on_request(req);
    }

    pub(crate) fn is_completed(&self, request: &lsp_server::Request) -> bool {
        self.req_queue.incoming.is_completed(&request.id)
    }

    fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        let loop_start = Instant::now();
        match event {
            Event::Lsp(msg) => match msg {
                lsp_server::Message::Request(req) => self.on_new_request(loop_start, req),
                lsp_server::Message::Notification(not) => self.on_notification(not)?,
                lsp_server::Message::Response(resp) => self.complete_request(resp),
            },
            Event::Task(task) => match task {
                Task::Response(response) => self.respond(response),
                Task::Retry(req) if !self.is_completed(&req) => self.on_request(req),
                Task::Retry(_) => (),
            },
        }
        self.process_changes();
        Ok(())
    }

    pub(crate) fn process_changes(&mut self) -> bool {
        let mut file_changes = FxHashMap::default();
        let (change, changed_files) = {
            let mut change = Change::new();
            let mut guard = self.vfs.write();
            let changed_files = guard.take_changes();
            if changed_files.is_empty() {
                return false;
            }

            // downgrade to read lock to allow more readers while we are normalizing text
            let guard = RwLockWriteGuard::downgrade_to_upgradable(guard);
            let vfs: &Vfs = &guard;
            // We need to fix up the changed events a bit. If we have a create or modify for a file
            // id that is followed by a delete we actually skip observing the file text from the
            // earlier event, to avoid problems later on.
            for changed_file in changed_files {
                use vfs::ChangeKind::*;

                file_changes
                    .entry(changed_file.file_id)
                    .and_modify(|(change, just_created)| {
                        // None -> Delete => keep
                        // Create -> Delete => collapse
                        //
                        match (change, just_created, changed_file.change_kind) {
                            // latter `Delete` wins
                            (change, _, Delete) => *change = Delete,
                            // merge `Create` with `Create` or `Modify`
                            (Create, _, Create | Modify) => {}
                            // collapse identical `Modify`es
                            (Modify, _, Modify) => {}
                            // equivalent to `Modify`
                            (change @ Delete, just_created, Create) => {
                                *change = Modify;
                                *just_created = true;
                            }
                            // shouldn't occur, but collapse into `Create`
                            (change @ Delete, just_created, Modify) => {
                                *change = Create;
                                *just_created = true;
                            }
                            // shouldn't occur, but collapse into `Modify`
                            (Modify, _, Create) => {}
                        }
                    })
                    .or_insert((
                        changed_file.change_kind,
                        matches!(changed_file.change_kind, Create),
                    ));
            }

            let changed_files: Vec<_> = file_changes
                .into_iter()
                .filter(|(_, (change_kind, just_created))| {
                    !matches!((change_kind, just_created), (vfs::ChangeKind::Delete, true))
                })
                .map(|(file_id, (change_kind, _))| vfs::ChangedFile {
                    file_id,
                    change_kind,
                })
                .collect();

            // let mut workspace_structure_change = None;
            // A file was added or deleted
            let mut has_structure_changes = false;
            let mut bytes = vec![];
            for file in &changed_files {
                let vfs_path = &vfs.file_path(file.file_id);
                // if let Some(path) = vfs_path.as_path() {
                //     let path = path.to_path_buf();
                //     if file.is_created_or_deleted() {
                //         has_structure_changes = true;
                //         workspace_structure_change =
                //             Some((path, self.crate_graph_file_dependencies.contains(vfs_path)));
                //     }
                // }

                // Clear native diagnostics when their file gets deleted
                // if !file.exists() {
                //     self.diagnostics.clear_native_for(file.file_id);
                // }

                let text = if file.exists() {
                    let bytes = vfs.file_contents(file.file_id).to_vec();

                    String::from_utf8(bytes).ok().and_then(|text| {
                        // FIXME: Consider doing normalization in the `vfs` instead? That allows
                        // getting rid of some locking
                        let (text, line_endings) = LineEndings::normalize(text);
                        Some((Arc::from(text), line_endings))
                    })
                } else {
                    None
                };
                // delay `line_endings_map` changes until we are done normalizing the text
                // this allows delaying the re-acquisition of the write lock
                bytes.push((file.file_id, text));
            }
            let vfs = &mut *RwLockUpgradableReadGuard::upgrade(guard);
            bytes.into_iter().for_each(|(file_id, text)| match text {
                None => change.change_file(file_id, None),
                Some((text, line_endings)) => {
                    change.change_file(file_id, Some(text));
                }
            });
            // if has_structure_changes {
            //     let roots = self.source_root_config.partition(vfs);
            //     change.set_roots(roots);
            // }
            (change, changed_files)
        };

        self.analysis_host.apply_change(change);

        true
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        log::debug!(
            "sourcepawn-lsp will use a maximum of {} threads.",
            self.pool.max_count()
        );
        self.initialize()?;
        while let Some(event) = self.next_event(&self.connection.receiver) {
            if matches!(
                &event,
                Event::Lsp(lsp_server::Message::Notification(lsp_server::Notification { method, .. }))
                if method == lsp_types::notification::Exit::METHOD
            ) {
                return Ok(());
            }
            self.handle_event(event)?;
        }
        Ok(())
    }
}

enum Event {
    Lsp(lsp_server::Message),
    Task(Task),
    // Vfs(vfs::loader::Message),
}

/// An immutable snapshot of the world's state at a point in time.
pub(crate) struct GlobalStateSnapshot {
    pub(crate) config: Arc<Config>,
    pub(crate) analysis: Analysis,
    vfs: Arc<RwLock<Vfs>>,
}

impl std::panic::UnwindSafe for GlobalStateSnapshot {}

impl GlobalStateSnapshot {
    fn vfs_read(&self) -> parking_lot::lock_api::RwLockReadGuard<'_, parking_lot::RawRwLock, Vfs> {
        self.vfs.read()
    }

    pub(crate) fn url_to_file_id(&self, url: &Url) -> anyhow::Result<FileId> {
        url_to_file_id(&self.vfs_read(), url)
    }

    pub(crate) fn file_id_to_url(&self, id: FileId) -> Url {
        file_id_to_url(&self.vfs_read(), id)
    }
}

pub(crate) fn file_id_to_url(vfs: &vfs::Vfs, id: FileId) -> Url {
    vfs.file_path(id)
}

pub(crate) fn url_to_file_id(vfs: &vfs::Vfs, uri: &Url) -> anyhow::Result<FileId> {
    let res = vfs
        .file_id(&uri)
        .ok_or_else(|| anyhow::format_err!("file not found: {uri}"))?;
    Ok(res)
}
