use std::{env, path::PathBuf, time::Instant};

use always_assert::always;
use crossbeam::channel::Receiver;
use lsp_server::Message;
use lsp_types::{
    notification::{Notification, ShowMessage},
    request::{Request, WorkspaceConfiguration},
    ConfigurationItem, ConfigurationParams, InitializeResult, MessageType, ServerInfo,
    ShowMessageParams, Url,
};
use paths::AbsPathBuf;
use stdx::thread::ThreadIntent;
use vfs::{FileId, VfsPath};

use crate::{
    capabilities::{server_capabilities, ClientCapabilitiesExt},
    config::Config,
    diagnostics::fetch_native_diagnostics,
    dispatch::{NotificationDispatcher, RequestDispatcher},
    from_json,
    global_state::file_id_to_url,
    lsp::from_proto,
    lsp_ext,
    progress::Progress,
    version::version,
    GlobalState,
};

#[derive(Debug)]
pub(crate) enum Event {
    Lsp(lsp_server::Message),
    Task(Task),
    Vfs(vfs::loader::Message),
    Flycheck(flycheck::Message),
}

#[derive(Debug)]
pub(crate) enum Task {
    Response(lsp_server::Response),
    Retry(lsp_server::Request),
    Diagnostics(Vec<(FileId, Vec<lsp_types::Diagnostic>)>),
    PrimeCaches(PrimeCachesProgress),
}

#[derive(Debug)]
pub(crate) enum PrimeCachesProgress {
    Begin,
    Report(ide::ParallelPrimeCachesProgress),
    End { cancelled: bool },
}

impl GlobalState {
    pub fn run(mut self) -> anyhow::Result<()> {
        log::debug!(
            "sourcepawn_lsp will use a maximum of {} threads.",
            self.pool.max_count()
        );
        self.update_status_or_notify();

        let mut ignored = self.initialize()?;

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

        self.reload_flycheck();

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
            if let Err(e) = config.update(config_data) {
                let not = lsp_server::Notification::new(
                    ShowMessage::METHOD.to_string(),
                    ShowMessageParams {
                        typ: MessageType::WARNING,
                        message: e.to_string(),
                    },
                );
                self.connection
                    .sender
                    .send(lsp_server::Message::Notification(not))
                    .unwrap();
            }
            ignored
        } else {
            Vec::new()
        };

        self.update_configuration(config);

        let _ = self.send_status(lsp_ext::ServerStatusParams {
            health: crate::lsp_ext::Health::Ok,
            quiescent: self.is_quiescent(),
            message: None,
        });
        log::debug!("Server is initialized.");

        Ok(ignored)
    }

    // FIXME: Get rid of this
    fn send_status(&self, status: lsp_ext::ServerStatusParams) -> anyhow::Result<()> {
        self.client
            .send_notification::<lsp_ext::ServerStatusNotification>(status)?;
        Ok(())
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

    fn next_event(&self, inbox: &Receiver<lsp_server::Message>) -> Option<Event> {
        crossbeam::channel::select! {
            recv(inbox) -> msg =>
                msg.ok().map(Event::Lsp),

            recv(self.task_pool.receiver) -> task =>
                Some(Event::Task(task.unwrap())),

            recv(self.loader.receiver) -> task =>
                Some(Event::Vfs(task.unwrap())),

            recv(self.flycheck_receiver) -> task =>
                Some(Event::Flycheck(task.unwrap())),
        }
    }

    fn update_status_or_notify(&mut self) {
        let status = self.current_status();
        if self.last_reported_status.as_ref() != Some(&status) {
            self.last_reported_status = Some(status.clone());

            if self.config.server_status_notification() {
                self.send_notification::<lsp_ext::ServerStatusNotification>(status);
            } else if let (
                health @ (lsp_ext::Health::Warning | lsp_ext::Health::Error),
                Some(message),
            ) = (status.health, &status.message)
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

    fn prime_caches(&mut self, cause: String) {
        tracing::debug!(%cause, "will prime caches");
        let num_worker_threads = self.config.prime_caches_num_threads();
        // FIXME: This is a full clone of the VFS
        let vfs = self.vfs.read().get_url_map();
        self.task_pool
            .handle
            .spawn_with_sender(ThreadIntent::Worker, {
                let analysis = self.snapshot().analysis;
                move |sender| {
                    sender
                        .send(Task::PrimeCaches(PrimeCachesProgress::Begin))
                        .unwrap();
                    let res = analysis.parallel_prime_caches(
                        num_worker_threads,
                        |progress| {
                            let report = PrimeCachesProgress::Report(progress);
                            sender.send(Task::PrimeCaches(report)).unwrap();
                        },
                        |id| {
                            vfs.get(&id).and_then(|path| {
                                path.name_and_extension().map(|(name, ext)| {
                                    format!("{}.{}", name, ext.unwrap_or_default())
                                })
                            })
                        },
                    );
                    sender
                        .send(Task::PrimeCaches(PrimeCachesProgress::End {
                            cancelled: res.is_err(),
                        }))
                        .unwrap();
                }
            });
    }

    /// Registers and handles a request. This should only be called once per incoming request.
    fn on_new_request(&mut self, request_received: Instant, req: lsp_server::Request) {
        self.register_request(&req, request_received);
        self.on_request(req);
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

        use super::handlers::request as handlers;
        use lsp_types::request as lsp_request;

        dispatcher
            .on_latency_sensitive::<lsp_request::SemanticTokensFullRequest>(
                handlers::handle_semantic_tokens_full,
            )
            .on_latency_sensitive::<lsp_request::SemanticTokensFullDeltaRequest>(
                handlers::handle_semantic_tokens_full_delta,
            )
            .on_latency_sensitive::<lsp_request::SemanticTokensRangeRequest>(
                handlers::handle_semantic_tokens_range,
            )
            .on::<lsp_request::GotoDefinition>(handlers::handle_goto_definition)
            .on::<lsp_request::HoverRequest>(handlers::handle_hover)
            .on::<lsp_ext::SyntaxTree>(handlers::handle_syntax_tree)
            .on::<lsp_ext::ProjectsGraphviz>(handlers::handle_projects_graphviz)
            .on::<lsp_ext::PreprocessedDocument>(handlers::handle_preprocessed_document)
            .on::<lsp_ext::ItemTree>(handlers::handle_item_tree)
            .finish();
        log::debug!("Handled request id: {:?}", req_id);
    }

    pub(super) fn on_notification(&mut self, not: lsp_server::Notification) -> anyhow::Result<()> {
        use super::handlers::notification as handlers;
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
        .on_sync_mut::<notifs::WorkDoneProgressCancel>(handlers::handle_work_done_progress_cancel)?
        .finish();

        Ok(())
    }

    pub(crate) fn is_completed(&self, request: &lsp_server::Request) -> bool {
        self.req_queue.incoming.is_completed(&request.id)
    }

    fn update_diagnostics(&mut self) {
        // let db = self.analysis_host.raw_database();
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

    fn handle_flycheck_msg(&mut self, message: flycheck::Message) {
        match message {
            flycheck::Message::AddDiagnostic { id, diagnostic, .. } => {
                let diag = crate::diagnostics::to_proto::map_spcomp_diagnostic_to_lsp(&diagnostic);
                if let Some(file_id) = self
                    .vfs
                    .read()
                    .file_id(&VfsPath::from(diagnostic.path().to_owned()))
                {
                    self.diagnostics.add_check_diagnostic(id, file_id, diag)
                }
            }

            flycheck::Message::Progress { id, progress } => {
                let (state, message) = match progress {
                    flycheck::Progress::DidStart => {
                        self.diagnostics.clear_check(id);
                        (Progress::Begin, None)
                    }
                    flycheck::Progress::DidCheckCrate(target) => (Progress::Report, Some(target)),
                    flycheck::Progress::DidCancel => {
                        self.last_flycheck_error = None;
                        (Progress::End, None)
                    }
                    flycheck::Progress::DidFailToRestart(err) => {
                        self.last_flycheck_error =
                            Some(format!("spcomp check failed to start: {err}"));
                        return;
                    }
                    flycheck::Progress::DidFinish(result) => {
                        self.last_flycheck_error = result
                            .err()
                            .map(|err| format!("spcomp check failed to start: {err}"));
                        (Progress::End, None)
                    }
                };

                // When we're running multiple flychecks, we have to include a disambiguator in
                // the title, or the editor complains. Note that this is a user-facing string.
                let title = if self.flycheck.len() == 1 {
                    "spcomp check".to_string()
                } else {
                    format!("spcomp check (#{})", id + 1)
                };
                self.report_progress(
                    &title,
                    state,
                    message,
                    None,
                    Some(format!("sourcepawn-lsp/flycheck/{id}")),
                );
            }
        }
    }

    fn handle_task(&mut self, prime_caches_progress: &mut Vec<PrimeCachesProgress>, task: Task) {
        match task {
            Task::Response(response) => self.respond(response),
            Task::Retry(req) if !self.is_completed(&req) => self.on_request(req),
            Task::Retry(_) => (),
            Task::Diagnostics(diagnostics_per_file) => {
                for (file_id, diagnostics) in diagnostics_per_file {
                    self.diagnostics
                        .set_native_diagnostics(file_id, diagnostics)
                }
            }
            Task::PrimeCaches(progress) => match progress {
                PrimeCachesProgress::Begin => prime_caches_progress.push(progress),
                PrimeCachesProgress::Report(_) => {
                    match prime_caches_progress.last_mut() {
                        Some(last @ PrimeCachesProgress::Report(_)) => {
                            // Coalesce subsequent update events.
                            *last = progress;
                        }
                        _ => prime_caches_progress.push(progress),
                    }
                }
                PrimeCachesProgress::End { .. } => prime_caches_progress.push(progress),
            },
        }
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
            Event::Task(task) => {
                let mut prime_caches_progress = Vec::new();
                self.handle_task(&mut prime_caches_progress, task);
                // Coalesce multiple task events into one loop turn
                while let Ok(task) = self.task_pool.receiver.try_recv() {
                    self.handle_task(&mut prime_caches_progress, task);
                }
                for progress in prime_caches_progress {
                    let (state, message, fraction);
                    match progress {
                        PrimeCachesProgress::Begin => {
                            state = Progress::Begin;
                            message = None;
                            fraction = 0.0;
                        }
                        PrimeCachesProgress::Report(report) => {
                            state = Progress::Report;

                            message = match &report.projects_currently_indexing[..] {
                                [crate_name] => Some(format!(
                                    "{}/{} ({crate_name})",
                                    report.projects_done, report.projects_total
                                )),
                                [crate_name, rest @ ..] => Some(format!(
                                    "{}/{} ({} + {} more)",
                                    report.projects_done,
                                    report.projects_total,
                                    crate_name,
                                    rest.len()
                                )),
                                _ => None,
                            };

                            fraction =
                                Progress::fraction(report.projects_done, report.projects_total);
                        }
                        PrimeCachesProgress::End { cancelled } => {
                            state = Progress::End;
                            message = None;
                            fraction = 1.0;

                            self.prime_caches_queue.op_completed(());
                            if cancelled {
                                self.prime_caches_queue
                                    .request_op("restart after cancellation".to_string(), ());
                            }
                        }
                    };

                    self.report_progress("Indexing", state, message, Some(fraction), None);
                }
            }
            Event::Vfs(message) => {
                self.handle_vfs_msg(message);
                // Coalesce many VFS event into a single loop turn
                while let Ok(message) = self.loader.receiver.try_recv() {
                    self.handle_vfs_msg(message);
                }
            }
            Event::Flycheck(message) => {
                self.handle_flycheck_msg(message);
                // Coalesce many flycheck updates into a single loop turn
                while let Ok(message) = self.flycheck_receiver.try_recv() {
                    self.handle_flycheck_msg(message);
                }
            }
        }
        let state_changed = self.process_changes();
        let memdocs_added_or_removed = self.mem_docs.take_changes();

        if self.is_quiescent() {
            let became_quiescent = !(was_quiescent);

            if became_quiescent && self.config.prefill_caches() {
                self.prime_caches_queue
                    .request_op("became quiescent".to_string(), ());
            }

            let client_refresh = !was_quiescent || state_changed;
            if client_refresh {
                // Refresh semantic tokens if the client supports it.
                if self.config.semantic_tokens_refresh() {
                    self.semantic_tokens_cache.lock().clear();
                    self.send_request::<lsp_types::request::SemanticTokensRefresh>((), |_, _| ());
                }

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
            }

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

        if let Some((cause, ())) = self.prime_caches_queue.should_start_op() {
            self.prime_caches(cause);
        }

        self.update_status_or_notify();

        Ok(())
    }
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
