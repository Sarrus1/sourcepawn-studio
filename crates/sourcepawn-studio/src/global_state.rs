use base_db::{Change, FileExtension, SourceRootConfig};
use crossbeam::channel::{unbounded, Receiver, Sender};
use flycheck::FlycheckHandle;
use fxhash::FxHashMap;
use ide::{Analysis, AnalysisHost, Cancellable};

use itertools::Itertools;
use lsp_server::{Connection, ErrorCode, RequestId};
use lsp_types::{SemanticTokens, Url};
use nohash_hasher::IntMap;
use parking_lot::{
    MappedRwLockReadGuard, Mutex, RwLock, RwLockReadGuard, RwLockUpgradableReadGuard,
    RwLockWriteGuard,
};
use serde::Serialize;
use std::{sync::Arc, time::Instant};
use tempfile::TempDir;
use threadpool::ThreadPool;
use vfs::{FileId, Vfs};

use crate::{
    client::LspClient,
    config::{Config, ConfigError},
    diagnostics::DiagnosticCollection,
    line_index::{LineEndings, LineIndex},
    lsp::{self, from_proto, to_proto::url_from_abs_path},
    main_loop::Task,
    mem_docs::MemDocs,
    op_queue::OpQueue,
    task_pool::TaskPool,
};

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
    pub(crate) req_queue: ReqQueue,
    pub(crate) sender: Sender<lsp_server::Message>,

    pub(crate) task_pool: Handle<TaskPool<Task>, Receiver<Task>>,
    pub(crate) diagnostics: DiagnosticCollection,
    pub(crate) mem_docs: MemDocs,
    pub(crate) source_root_config: SourceRootConfig,
    pub(crate) semantic_tokens_cache: Arc<Mutex<FxHashMap<Url, SemanticTokens>>>,

    pub(crate) connection: Arc<Connection>,
    pub(crate) client: LspClient,
    pub(crate) pool: ThreadPool,
    pub(crate) amxxpawn_mode: bool,

    // status
    pub(crate) shutdown_requested: bool,
    pub(crate) last_reported_status: Option<lsp::ext::ServerStatusParams>,

    pub config: Arc<Config>,
    pub(crate) config_errors: Option<ConfigError>,

    pub(crate) analysis_host: AnalysisHost,

    // Flycheck
    pub(crate) flycheck: Arc<FxHashMap<FileId, FlycheckHandle>>,
    pub(crate) flycheck_tempdir: TempDir,
    pub(crate) flycheck_sender: Sender<flycheck::Message>,
    pub(crate) flycheck_receiver: Receiver<flycheck::Message>,
    pub(crate) last_flycheck_error: Option<String>,

    // VFS
    pub(crate) loader: Handle<Box<dyn vfs::loader::Handle>, Receiver<vfs::loader::Message>>,
    pub(crate) vfs: Arc<RwLock<(vfs::Vfs, IntMap<FileId, LineEndings>)>>,
    pub(crate) vfs_config_version: u32,
    pub(crate) vfs_progress_config_version: u32,
    pub(crate) vfs_progress_n_total: usize,
    pub(crate) vfs_progress_n_done: usize,

    // op queues
    pub(crate) prime_caches_queue: OpQueue,
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

        let (flycheck_sender, flycheck_receiver) = unbounded();
        Self {
            client,
            pool: threadpool::Builder::new().build(),
            amxxpawn_mode,

            req_queue: ReqQueue::default(),
            sender: connection.sender.clone(),
            connection: Arc::new(connection),
            task_pool,

            mem_docs: MemDocs::default(),
            source_root_config: SourceRootConfig::default(),
            semantic_tokens_cache: Arc::new(Mutex::new(FxHashMap::default())),
            diagnostics: DiagnosticCollection::default(),

            shutdown_requested: false,
            last_reported_status: None,

            config: Arc::default(),
            config_errors: Default::default(),
            analysis_host: AnalysisHost::default(),

            flycheck: Arc::new(FxHashMap::default()),
            flycheck_tempdir: TempDir::new().expect("failed to create temp dir"),
            flycheck_sender,
            flycheck_receiver,
            last_flycheck_error: None,

            loader,
            vfs: Arc::new(RwLock::new((vfs::Vfs::default(), IntMap::default()))),
            vfs_config_version: 0,
            vfs_progress_config_version: 0,
            vfs_progress_n_total: 0,
            vfs_progress_n_done: 0,

            prime_caches_queue: Default::default(),
        }
    }

    pub(crate) fn snapshot(&self) -> GlobalStateSnapshot {
        GlobalStateSnapshot {
            config: Arc::clone(&self.config),
            analysis: self.analysis_host.analysis(),
            mem_docs: self.mem_docs.clone(),
            semantic_tokens_cache: Arc::clone(&self.semantic_tokens_cache),
            flycheck: self.flycheck.clone(),
            vfs: Arc::clone(&self.vfs),
        }
    }

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

    pub(crate) fn process_changes(&mut self) -> bool {
        let mut file_changes = FxHashMap::default();
        let (change, _changed_files) = {
            let mut change = Change::new();
            let mut guard = self.vfs.write();
            let changed_files = guard.0.take_changes();
            if changed_files.is_empty() {
                return false;
            }

            // downgrade to read lock to allow more readers while we are normalizing text
            let guard = RwLockWriteGuard::downgrade_to_upgradable(guard);
            let vfs: &Vfs = &guard.0;
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

                    String::from_utf8(bytes).ok().map(|text| {
                        // FIXME: Consider doing normalization in the `vfs` instead? That allows
                        // getting rid of some locking
                        let (text, line_endings) = LineEndings::normalize(text);
                        (Arc::from(text), line_endings)
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
                let roots = self.source_root_config.partition(&vfs.0);
                change.set_roots(roots);
            }
            (change, changed_files)
        };

        self.analysis_host.apply_change(change);

        let mut files = self
            .vfs
            .read()
            .0
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
}

/// An immutable snapshot of the world's state at a point in time.
pub(crate) struct GlobalStateSnapshot {
    pub(crate) config: Arc<Config>,
    pub(crate) analysis: Analysis,
    #[allow(unused)]
    pub(crate) mem_docs: MemDocs,
    pub(crate) semantic_tokens_cache: Arc<Mutex<FxHashMap<Url, SemanticTokens>>>,
    pub(crate) flycheck: Arc<FxHashMap<FileId, FlycheckHandle>>,
    vfs: Arc<RwLock<(vfs::Vfs, IntMap<FileId, LineEndings>)>>,
}

impl std::panic::UnwindSafe for GlobalStateSnapshot {}

impl GlobalStateSnapshot {
    fn vfs_read(&self) -> MappedRwLockReadGuard<'_, vfs::Vfs> {
        RwLockReadGuard::map(self.vfs.read(), |(it, _)| it)
    }

    pub(crate) fn url_to_file_id(&self, uri: &Url) -> anyhow::Result<FileId> {
        url_to_file_id(&self.vfs_read(), uri)
    }

    pub(crate) fn file_id_to_url(&self, id: FileId) -> Url {
        file_id_to_url(&self.vfs_read(), id)
    }

    pub(crate) fn file_line_index(&self, file_id: FileId) -> Cancellable<LineIndex> {
        let endings = self.vfs.read().1[&file_id];
        let index = self.analysis.file_line_index(file_id)?;
        let res = LineIndex {
            index,
            endings,
            encoding: self.config.position_encoding(),
        };
        Ok(res)
    }

    #[allow(unused)]
    pub(crate) fn url_file_version(&self, uri: &Url) -> Option<i32> {
        let path = from_proto::vfs_path(uri).ok()?;
        self.mem_docs.get(&path)?.version.into()
    }

    pub(crate) fn vfs_memory_usage(&self) -> usize {
        self.vfs_read().memory_usage()
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
