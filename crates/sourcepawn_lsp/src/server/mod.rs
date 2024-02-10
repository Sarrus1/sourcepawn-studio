use always_assert::always;
use base_db::{Change, FileExtension, SourceRootConfig};
use crossbeam::channel::{unbounded, Receiver, Sender};
use fxhash::FxHashMap;
use ide::{Analysis, AnalysisHost};

use itertools::Itertools;
use lsp_server::{Connection, ErrorCode, Message, RequestId};
use lsp_types::{
    notification::{Notification, ShowMessage},
    request::{Request, WorkspaceConfiguration},
    ConfigurationItem, ConfigurationParams, InitializeResult, MessageType, ServerInfo,
    ShowMessageParams, Url,
};
use parking_lot::{RwLock, RwLockUpgradableReadGuard, RwLockWriteGuard};
use paths::AbsPathBuf;
use serde::Serialize;
use std::{env, path::PathBuf, sync::Arc, time::Instant};
use stdx::thread::ThreadIntent;
use threadpool::ThreadPool;
use vfs::{FileId, Vfs, VfsPath};

use crate::{
    capabilities::{server_capabilities, ClientCapabilitiesExt},
    client::LspClient,
    config::{Config, ConfigData},
    diagnostics::{fetch_native_diagnostics, DiagnosticCollection},
    dispatch::{NotificationDispatcher, RequestDispatcher},
    from_json,
    line_index::LineEndings,
    lsp::{from_proto, to_proto::url_from_abs_path},
    lsp_ext,
    mem_docs::MemDocs,
    op_queue::OpQueue,
    server::progress::Progress,
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
mod reload;
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
    pub(crate) diagnostics: DiagnosticCollection,
    pub(crate) mem_docs: MemDocs,
    pub(crate) source_root_config: SourceRootConfig,

    connection: Arc<Connection>,
    client: LspClient,
    pub(crate) pool: ThreadPool,
    indexing: bool,
    amxxpawn_mode: bool,

    // status
    pub(crate) shutdown_requested: bool,
    pub(crate) last_reported_status: Option<lsp_ext::ServerStatusParams>,

    pub(crate) config: Arc<Config>,
    pub(crate) config_errors: Option<serde_json::Error>,

    pub(crate) analysis_host: AnalysisHost,

    // VFS
    pub(crate) loader: Handle<Box<dyn vfs::loader::Handle>, Receiver<vfs::loader::Message>>,
    pub(crate) vfs: Arc<RwLock<Vfs>>,
    pub(crate) vfs_config_version: u32,
    pub(crate) vfs_progress_config_version: u32,
    pub(crate) vfs_progress_n_total: usize,
    pub(crate) vfs_progress_n_done: usize,
    // // op queues
    // pub(crate) fetch_workspaces_queue:
    //     OpQueue<bool, Option<(Vec<anyhow::Result<ProjectWorkspace>>, bool)>>,
}

impl GlobalState {
    pub fn new(connection: Connection, amxxpawn_mode: bool) -> Self {
        let loader = {
            let (sender, receiver) = unbounded::<vfs::loader::Message>();
            let handle: vfs_notify::NotifyHandle =
                vfs::loader::Handle::spawn(Box::new(move |msg| sender.send(msg).unwrap()));
            let handle = Box::new(handle) as Box<dyn vfs::loader::Handle>;
            Handle { handle, receiver }
        };

        let client = LspClient::new(connection.sender.clone());
        let task_pool = {
            let (sender, receiver) = unbounded();
            let handle = TaskPool::new_with_threads(sender, num_cpus::get_physical());
            Handle { handle, receiver }
        };
        Self {
            client,
            pool: threadpool::Builder::new().build(),
            indexing: false,
            amxxpawn_mode,

            req_queue: ReqQueue::default(),
            sender: connection.sender.clone(),
            connection: Arc::new(connection),
            task_pool,

            mem_docs: MemDocs::default(),
            source_root_config: SourceRootConfig::default(),
            diagnostics: DiagnosticCollection::default(),

            shutdown_requested: false,
            last_reported_status: None,

            config: Arc::default(),
            config_errors: Default::default(),
            analysis_host: AnalysisHost::default(),

            loader,
            vfs: Arc::new(RwLock::new(Vfs::default())),
            vfs_config_version: 0,
            vfs_progress_config_version: 0,
            vfs_progress_n_total: 0,
            vfs_progress_n_done: 0,
        }
    }

    pub(crate) fn snapshot(&self) -> GlobalStateSnapshot {
        GlobalStateSnapshot {
            config: Arc::clone(&self.config),
            analysis: self.analysis_host.analysis(),
            mem_docs: self.mem_docs.clone(),
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

    /// Synchronously initialize the lsp server according to the LSP spec.
    fn initialize(&mut self) -> anyhow::Result<Vec<lsp_server::Message>> {
        log::debug!("Initializing server...");
        let (id, initialize_params) = self.connection.initialize_start()?;
        let lsp_types::InitializeParams {
            root_uri,
            capabilities,
            workspace_folders,
            initialization_options,
            client_info,
            ..
        } = from_json::<lsp_types::InitializeParams>("InitializeParams", &initialize_params)?;

        let root_path = match root_uri
            .clone()
            .and_then(|it| it.to_file_path().ok())
            .map(patch_path_prefix)
            .and_then(|it| AbsPathBuf::try_from(it).ok())
        {
            Some(it) => it,
            None => {
                let cwd = env::current_dir()?;
                AbsPathBuf::assert(cwd)
            }
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
            .unwrap_or_else(|| vec![root_path.clone().into()]);
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

        let ignored = if config.caps().has_pull_configuration_support() {
            let (config_data, ignored) = self.pull_config_sync(root_uri);
            // FIXME: Report error to the user.
            config.update(config_data);
            ignored
        } else {
            Vec::new()
        };

        self.update_configuration(config);

        let _ = self.send_status(lsp_ext::ServerStatusParams {
            health: crate::lsp_ext::Health::Ok,
            quiescent: !self.indexing,
            message: None,
        });
        log::debug!("Server is initialized.");

        Ok(ignored)
    }

    /// Synchronously pull the configuration from the client and return it, along with any ignored messages.
    fn pull_config_sync(
        &self,
        scope_uri: Option<Url>,
    ) -> (serde_json::Value, Vec<lsp_server::Message>) {
        let request_id = lsp_server::RequestId::from("initial_config_pull".to_string());
        let config_item = ConfigurationItem {
            scope_uri,
            section: Some("SourcePawnLanguageServer".to_string()),
        };
        let params = ConfigurationParams {
            items: vec![config_item],
        };
        let request = lsp_server::Request::new(
            request_id.clone(),
            WorkspaceConfiguration::METHOD.to_string(),
            params,
        );

        self.sender
            .send(Message::Request(request))
            .expect("Failed to send request");

        let mut config: Option<Vec<serde_json::Value>> = None;
        let mut ignored = Vec::new();
        while config.is_none() {
            match self.connection.receiver.recv() {
                Ok(Message::Response(response)) if response.id == request_id => {
                    if let Some(result) = response.result {
                        config = serde_json::from_value(result).ok();
                    }
                }
                Ok(e) => ignored.push(e),
                Err(e) => panic!("Error receiving message: {:?}", e),
            }
        }

        // Reverse the stack of ignored events to pop them in the correct order.
        ignored.reverse();

        (
            config
                .expect("Failed to receive configuration")
                .first()
                .expect("Empty configuration")
                .clone(),
            ignored,
        )
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

    pub(crate) fn send_notification<N: lsp_types::notification::Notification>(
        &self,
        params: N::Params,
    ) {
        let not = lsp_server::Notification::new(N::METHOD.to_string(), params);
        self.send(not.into());
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
    fn on_request(&mut self, req: lsp_server::Request) {
        log::debug!("received request: {:?}", req);
        let req_id = req.id.clone();
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
            .on::<lsp_ext::ProjectsGraphviz>(handlers::handle_projects_graphviz)
            .finish();
        log::debug!("Handled request id: {:?}", req_id);
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
        .on_sync_mut::<notifs::DidCloseTextDocument>(handlers::handle_did_close_text_document)?
        .on_sync_mut::<notifs::DidSaveTextDocument>(handlers::handle_did_save_text_document)?
        .on_sync_mut::<notifs::DidChangeConfiguration>(handlers::handle_did_change_configuration)?
        .on_sync_mut::<notifs::DidChangeWatchedFiles>(handlers::handle_did_change_watched_files)? // TODO: Implement this.
        .finish();

        Ok(())
    }

    fn next_event(&self, inbox: &Receiver<lsp_server::Message>) -> Option<Event> {
        crossbeam::channel::select! {
            recv(inbox) -> msg =>
                msg.ok().map(Event::Lsp),

            recv(self.task_pool.receiver) -> task =>
                Some(Event::Task(task.unwrap())),

            recv(self.loader.receiver) -> task =>
                Some(Event::Vfs(task.unwrap())),
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
    fn on_new_request(&mut self, request_received: Instant, req: lsp_server::Request) {
        self.register_request(&req, request_received);
        self.on_request(req);
    }

    pub(crate) fn is_completed(&self, request: &lsp_server::Request) -> bool {
        self.req_queue.incoming.is_completed(&request.id)
    }

    fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        log::debug!("handle_event: {:?}", event);

        let was_quiescent = self.is_quiescent();

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
                Task::Diagnostics(diagnostics_per_file) => {
                    for (file_id, diagnostics) in diagnostics_per_file {
                        self.diagnostics
                            .set_native_diagnostics(file_id, diagnostics)
                    }
                }
            },
            Event::Vfs(message) => {
                self.handle_vfs_msg(message);
                // Coalesce many VFS event into a single loop turn
                while let Ok(message) = self.loader.receiver.try_recv() {
                    self.handle_vfs_msg(message);
                }
            }
        }
        let state_changed = self.process_changes();
        let memdocs_added_or_removed = self.mem_docs.take_changes();

        if self.is_quiescent() {
            let became_quiescent = !(was_quiescent);

            // let client_refresh = !was_quiescent || state_changed;
            // if client_refresh {
            // Refresh semantic tokens if the client supports it.
            // if self.config.semantic_tokens_refresh() {
            //     self.semantic_tokens_cache.lock().clear();
            //     self.send_request::<lsp_types::request::SemanticTokensRefresh>((), |_, _| ());
            // }

            // Refresh code lens if the client supports it.
            // if self.config.code_lens_refresh() {
            //     self.send_request::<lsp_types::request::CodeLensRefresh>((), |_, _| ());
            // }

            // Refresh inlay hints if the client supports it.
            // if (self.send_hint_refresh_query || self.proc_macro_changed)
            //     && self.config.inlay_hints_refresh()
            // {
            //     self.send_request::<lsp_types::request::InlayHintRefreshRequest>((), |_, _| ());
            //     self.send_hint_refresh_query = false;
            // }
            // }

            let update_diagnostics = (!was_quiescent || state_changed || memdocs_added_or_removed)
                && self.config.publish_diagnostics();
            if update_diagnostics {
                self.update_diagnostics()
            }
        }

        if let Some(diagnostic_changes) = self.diagnostics.take_changes() {
            for file_id in diagnostic_changes {
                let uri = file_id_to_url(&self.vfs.read(), file_id);

                let mut diagnostics = self
                    .diagnostics
                    .diagnostics_for(file_id)
                    .cloned()
                    .collect::<Vec<_>>();

                // VSCode assumes diagnostic messages to be non-empty strings, so we need to patch
                // empty diagnostics. Neither the docs of VSCode nor the LSP spec say whether
                // diagnostic messages are actually allowed to be empty or not and patching this
                // in the VSCode client does not work as the assertion happens in the protocol
                // conversion. So this hack is here to stay, and will be considered a hack
                // until the LSP decides to state that empty messages are allowed.

                // See https://github.com/rust-lang/rust-analyzer/issues/11404
                // See https://github.com/rust-lang/rust-analyzer/issues/13130
                let patch_empty = |message: &mut String| {
                    if message.is_empty() {
                        *message = " ".to_string();
                    }
                };

                for d in &mut diagnostics {
                    patch_empty(&mut d.message);
                    if let Some(dri) = &mut d.related_information {
                        for dri in dri {
                            patch_empty(&mut dri.message);
                        }
                    }
                }

                let version = from_proto::vfs_path(&uri)
                    .map(|path| self.mem_docs.get(&path).map(|it| it.version))
                    .unwrap_or_default();

                self.send_notification::<lsp_types::notification::PublishDiagnostics>(
                    lsp_types::PublishDiagnosticsParams {
                        uri,
                        diagnostics,
                        version,
                    },
                );
            }
        }

        self.update_status_or_notify();

        Ok(())
    }

    fn handle_vfs_msg(&mut self, message: vfs::loader::Message) {
        match message {
            vfs::loader::Message::Loaded { files } => {
                let vfs = &mut self.vfs.write();
                for (path, contents) in files {
                    let path = VfsPath::from(path);
                    if !self.mem_docs.contains(&path) {
                        vfs.set_file_contents(path, contents);
                    }
                }
            }
            vfs::loader::Message::Progress {
                n_total,
                n_done,
                config_version,
            } => {
                always!(config_version <= self.vfs_config_version);

                self.vfs_progress_config_version = config_version;
                self.vfs_progress_n_total = n_total;
                self.vfs_progress_n_done = n_done;

                let state = if n_done == 0 {
                    Progress::Begin
                } else if n_done < n_total {
                    Progress::Report
                } else {
                    assert_eq!(n_done, n_total);
                    Progress::End
                };
                self.report_progress(
                    "Roots Scanned",
                    state,
                    Some(format!("{n_done}/{n_total}")),
                    Some(Progress::fraction(n_done, n_total)),
                    None,
                );
            }
        }
    }

    fn update_diagnostics(&mut self) {
        let db = self.analysis_host.raw_database();
        let subscriptions = self
            .mem_docs
            .iter()
            .map(|path| self.vfs.read().file_id(path).unwrap())
            // .filter(|&file_id| {
            //     let source_root = db.file_source_root(file_id);
            //     // Only publish diagnostics for files in the workspace, not from crates.io deps
            //     // or the sysroot.
            //     // While theoretically these should never have errors, we have quite a few false
            //     // positives particularly in the stdlib, and those diagnostics would stay around
            //     // forever if we emitted them here.
            //     !db.source_root(source_root).is_library
            // })
            .collect::<Vec<_>>();
        tracing::trace!("updating notifications for {:?}", subscriptions);

        // Diagnostics are triggered by the user typing
        // so we run them on a latency sensitive thread.
        self.task_pool
            .handle
            .spawn(ThreadIntent::LatencySensitive, {
                let snapshot = self.snapshot();
                move || Task::Diagnostics(fetch_native_diagnostics(snapshot, subscriptions))
            });
    }

    fn update_status_or_notify(&mut self) {
        let status = self.current_status();
        if self.last_reported_status.as_ref() != Some(&status) {
            self.last_reported_status = Some(status.clone());

            // TODO: Handle this config eventually.
            // if self.config.server_status_notification() {
            //     self.send_notification::<lsp_ext::ServerStatusNotification>(status);
            // } else
            if let (health @ (lsp_ext::Health::Warning | lsp_ext::Health::Error), Some(message)) =
                (status.health, &status.message)
            {
                // let open_log_button = tracing::enabled!(tracing::Level::ERROR)
                //     && (self.fetch_build_data_error().is_err()
                //         || self.fetch_workspace_error().is_err());
                let open_log_button = false;
                self.show_message(
                    match health {
                        lsp_ext::Health::Ok => lsp_types::MessageType::INFO,
                        lsp_ext::Health::Warning => lsp_types::MessageType::WARNING,
                        lsp_ext::Health::Error => lsp_types::MessageType::ERROR,
                    },
                    message.clone(),
                    open_log_button,
                );
            }
        }
    }

    pub(crate) fn process_changes(&mut self) -> bool {
        let mut file_changes = FxHashMap::default();
        let (change, _changed_files) = {
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

            // A file was added or deleted
            // let mut workspace_structure_change = None;
            let mut has_structure_changes = false;
            let mut bytes = vec![];
            for file in &changed_files {
                let vfs_path = &vfs.file_path(file.file_id);
                if let Some(path) = vfs_path.as_path() {
                    let _path = path.to_path_buf();
                    if file.is_created_or_deleted() {
                        has_structure_changes = true;
                        // workspace_structure_change =
                        //     Some((path, self.crate_graph_file_dependencies.contains(vfs_path)));
                    }
                }

                // Clear native diagnostics when their file gets deleted
                if !file.exists() {
                    self.diagnostics.clear_native_for(file.file_id);
                }

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
                Some((text, _line_endings)) => {
                    change.change_file(file_id, Some(text));
                }
            });
            if has_structure_changes {
                let roots = self.source_root_config.partition(vfs);
                change.set_roots(roots);
            }
            (change, changed_files)
        };

        self.analysis_host.apply_change(change);

        let mut files = self
            .vfs
            .read()
            .iter()
            .flat_map(|(id, path)| {
                let (_, ext) = path.name_and_extension()?;
                FileExtension::try_from(ext?).ok().map(|ext| (id, ext))
            })
            .collect_vec();
        files.sort(); // FIXME: Maybe we can avoid sorting here? This was done to make the query deterministic.
        self.analysis_host.set_known_files(files);

        true
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        log::debug!(
            "sourcepawn_lsp will use a maximum of {} threads.",
            self.pool.max_count()
        );
        self.update_status_or_notify();

        let mut ignored = self.initialize()?;

        // self.fetch_workspaces_queue
        //     .request_op("startup".to_string(), false);
        // if let Some((cause, force_crate_graph_reload)) =
        //     self.fetch_workspaces_queue.should_start_op()
        // {
        //     self.fetch_workspaces(cause, force_crate_graph_reload);
        // }

        while let Some(event) = ignored.pop() {
            let event = Event::Lsp(event);
            if matches!(
                &event,
                Event::Lsp(lsp_server::Message::Notification(lsp_server::Notification { method, .. }))
                if method == lsp_types::notification::Exit::METHOD
            ) {
                return Ok(());
            }
            self.handle_event(event)?;
        }

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

#[derive(Debug)]
enum Event {
    Lsp(lsp_server::Message),
    Task(Task),
    Vfs(vfs::loader::Message),
}

/// An immutable snapshot of the world's state at a point in time.
pub(crate) struct GlobalStateSnapshot {
    pub(crate) config: Arc<Config>,
    pub(crate) analysis: Analysis,
    pub(crate) mem_docs: MemDocs,
    vfs: Arc<RwLock<vfs::Vfs>>,
}

impl std::panic::UnwindSafe for GlobalStateSnapshot {}

impl GlobalStateSnapshot {
    fn vfs_read(&self) -> parking_lot::lock_api::RwLockReadGuard<'_, parking_lot::RawRwLock, Vfs> {
        self.vfs.read()
    }

    pub(crate) fn url_to_file_id(&self, uri: &Url) -> anyhow::Result<FileId> {
        url_to_file_id(&self.vfs_read(), uri)
    }

    pub(crate) fn file_id_to_url(&self, id: FileId) -> Url {
        file_id_to_url(&self.vfs_read(), id)
    }

    pub(crate) fn url_file_version(&self, uri: &Url) -> Option<i32> {
        let path = from_proto::vfs_path(uri).ok()?;
        self.mem_docs.get(&path)?.version.into()
    }
}

pub(crate) fn file_id_to_url(vfs: &vfs::Vfs, id: FileId) -> Url {
    let path = vfs.file_path(id);
    let path = path.as_path().unwrap();
    url_from_abs_path(path)
}

pub(crate) fn url_to_file_id(vfs: &vfs::Vfs, url: &Url) -> anyhow::Result<FileId> {
    let path = from_proto::vfs_path(url)?;
    let res = vfs
        .file_id(&path)
        .ok_or_else(|| anyhow::format_err!("file not found: {path}"))?;
    Ok(res)
}

fn patch_path_prefix(path: PathBuf) -> PathBuf {
    use std::path::{Component, Prefix};
    if cfg!(windows) {
        // VSCode might report paths with the file drive in lowercase, but this can mess
        // with env vars set by tools and build scripts executed by r-a such that it invalidates
        // cargo's compilations unnecessarily. https://github.com/rust-lang/rust-analyzer/issues/14683
        // So we just uppercase the drive letter here unconditionally.
        // (doing it conditionally is a pain because std::path::Prefix always reports uppercase letters on windows)
        let mut comps = path.components();
        match comps.next() {
            Some(Component::Prefix(prefix)) => {
                let prefix = match prefix.kind() {
                    Prefix::Disk(d) => {
                        format!("{}:", d.to_ascii_uppercase() as char)
                    }
                    Prefix::VerbatimDisk(d) => {
                        format!(r"\\?\{}:", d.to_ascii_uppercase() as char)
                    }
                    _ => return path,
                };
                let mut path = PathBuf::new();
                path.push(prefix);
                path.extend(comps);
                path
            }
            _ => path,
        }
    } else {
        path
    }
}
