//! Flycheck provides the functionality needed to run `spcomp` in a background thread and provide
//! LSP diagnostics based on the output of the command.

#![warn(
    rust_2018_idioms,
    unused_lifetimes,
    semicolon_in_expressions_from_macros
)]

use std::{
    ffi::OsString,
    fmt, io,
    path::PathBuf,
    process::{ChildStderr, ChildStdout, Command, Stdio},
    time::Duration,
};

use command_group::{CommandGroup, GroupChild};
use crossbeam::channel::{never, select, unbounded, Receiver, Sender};
use paths::AbsPathBuf;
use rand::Rng;
use spcomp::build_args;
use stdx::process::streaming_output;

mod spcomp;

pub use spcomp::{SpCompDiagnostic, SpCompSeverity};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum InvocationStrategy {
    Once,
    #[default]
    PerWorkspace,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum InvocationLocation {
    Root(AbsPathBuf),
    #[default]
    Workspace,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlycheckConfig {
    command: String,
    args: Vec<String>,
    include_directories: Vec<AbsPathBuf>,
}

impl FlycheckConfig {
    pub fn new(command: String, args: Vec<String>, include_directories: Vec<AbsPathBuf>) -> Self {
        FlycheckConfig {
            command,
            args,
            include_directories,
        }
    }
}

/// Flycheck wraps the shared state and communication machinery used for
/// running `spcomp` and providing diagnostics based on the output.
/// The spawned thread is shut down when this struct is dropped.
#[derive(Debug)]
pub struct FlycheckHandle {
    // XXX: drop order is significant
    sender: Sender<StateChange>,
    _thread: stdx::thread::JoinHandle,
    id: u32,
}

impl FlycheckHandle {
    pub fn spawn(
        id: u32,
        sender: Box<dyn Fn(Message) + Send>,
        config: FlycheckConfig,
        project_root: AbsPathBuf,
        tempdir: AbsPathBuf,
    ) -> FlycheckHandle {
        let actor = FlycheckActor::new(id, sender, config, project_root, tempdir);
        let (sender, receiver) = unbounded::<StateChange>();
        let thread = stdx::thread::Builder::new(stdx::thread::ThreadIntent::Worker)
            .name("Flycheck".to_owned())
            .spawn(move || actor.run(receiver))
            .expect("failed to spawn thread");
        FlycheckHandle {
            id,
            sender,
            _thread: thread,
        }
    }

    /// Schedule a re-start of the spcomp worker.
    pub fn restart(&self) {
        self.sender.send(StateChange::Restart).unwrap();
    }

    /// Stop this spcomp worker.
    pub fn cancel(&self) {
        self.sender.send(StateChange::Cancel).unwrap();
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

pub enum Message {
    /// Request adding a diagnostic with fixes included to a file
    AddDiagnostic {
        id: u32,
        workspace_root: AbsPathBuf,
        diagnostic: SpCompDiagnostic,
    },

    /// Request check progress notification to client
    Progress {
        /// Flycheck instance ID
        id: u32,
        progress: Progress,
    },
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::AddDiagnostic {
                id,
                workspace_root,
                diagnostic,
            } => f
                .debug_struct("AddDiagnostic")
                .field("id", id)
                .field("workspace_root", workspace_root)
                .field("code", &diagnostic.code())
                .finish(),
            Message::Progress { id, progress } => f
                .debug_struct("Progress")
                .field("id", id)
                .field("progress", progress)
                .finish(),
        }
    }
}

#[derive(Debug)]
pub enum Progress {
    DidStart,
    // FIXME: Implement this
    DidCheckCrate(String),
    DidFinish(io::Result<()>),
    DidCancel,
    DidFailToRestart(String),
}

enum StateChange {
    Restart,
    Cancel,
}

/// A [`FlycheckActor`] is a single check instance of a workspace.
struct FlycheckActor {
    /// The project root FileId of this flycheck instance.
    id: u32,
    sender: Box<dyn Fn(Message) + Send>,
    config: FlycheckConfig,
    tempdir: AbsPathBuf,
    /// Either the workspace root of the workspace we are flychecking,
    /// or the project root of the project.
    root: AbsPathBuf,
    /// CargoHandle exists to wrap around the communication needed to be able to
    /// run `spcomp`  without blocking. Currently the Rust standard library
    /// doesn't provide a way to read sub-process output without blocking, so we
    /// have to wrap sub-processes output handling in a thread and pass messages
    /// back over a channel.
    command_handle: Option<CommandHandle>,
}

enum Event {
    RequestStateChange(StateChange),
    SpCompEvent(Option<SpCompDiagnostic>),
}

impl FlycheckActor {
    fn new(
        id: u32,
        sender: Box<dyn Fn(Message) + Send>,
        config: FlycheckConfig,
        workspace_root: AbsPathBuf,
        tempdir: AbsPathBuf,
    ) -> FlycheckActor {
        FlycheckActor {
            id,
            tempdir,
            sender,
            config,
            root: workspace_root,
            command_handle: None,
        }
    }

    fn report_progress(&self, progress: Progress) {
        self.send(Message::Progress {
            id: self.id,
            progress,
        });
    }

    fn next_event(&self, inbox: &Receiver<StateChange>) -> Option<Event> {
        let check_chan = self
            .command_handle
            .as_ref()
            .map(|spcomp: &CommandHandle| &spcomp.receiver);
        if let Ok(msg) = inbox.try_recv() {
            // give restarts a preference so check outputs don't block a restart or stop
            return Some(Event::RequestStateChange(msg));
        }
        select! {
            recv(inbox) -> msg => msg.ok().map(Event::RequestStateChange),
            recv(check_chan.unwrap_or(&never())) -> msg => Some(Event::SpCompEvent(msg.ok())),
        }
    }

    fn run(mut self, inbox: Receiver<StateChange>) {
        'event: while let Some(event) = self.next_event(&inbox) {
            match event {
                Event::RequestStateChange(StateChange::Cancel) => {
                    tracing::debug!(flycheck_id = self.id, "flycheck cancelled");
                    self.cancel_check_process();
                }
                Event::RequestStateChange(StateChange::Restart) => {
                    // Cancel the previously spawned process
                    self.cancel_check_process();
                    while let Ok(restart) = inbox.recv_timeout(Duration::from_millis(50)) {
                        // restart chained with a stop, so just cancel
                        if let StateChange::Cancel = restart {
                            continue 'event;
                        }
                    }

                    let command = self.check_command();
                    let formatted_command = format!("{:?}", command);

                    tracing::debug!(?command, "will restart flycheck");
                    match CommandHandle::spawn(command) {
                        Ok(command_handle) => {
                            tracing::debug!(command = formatted_command, "did  restart flycheck");
                            self.command_handle = Some(command_handle);
                            self.report_progress(Progress::DidStart);
                        }
                        Err(error) => {
                            self.report_progress(Progress::DidFailToRestart(format!(
                                "Failed to run the following command: {} error={}",
                                formatted_command, error
                            )));
                        }
                    }
                }
                Event::SpCompEvent(None) => {
                    tracing::debug!(flycheck_id = self.id, "flycheck finished");

                    // Watcher finished
                    let command_handle = self.command_handle.take().unwrap();
                    let formatted_handle = format!("{:?}", command_handle);

                    let res = command_handle.join();
                    if res.is_err() {
                        tracing::error!(
                            "Flycheck failed to run the following command: {}",
                            formatted_handle
                        );
                    }
                    self.report_progress(Progress::DidFinish(res));
                }
                Event::SpCompEvent(Some(diagnostic)) => self.send(Message::AddDiagnostic {
                    id: self.id,
                    workspace_root: self.root.clone(),
                    diagnostic,
                }),
            }
        }
        // If we rerun the thread, we need to discard the previous check results first
        self.cancel_check_process();
    }

    fn cancel_check_process(&mut self) {
        if let Some(command_handle) = self.command_handle.take() {
            tracing::debug!(
                command = ?command_handle,
                "did  cancel flycheck"
            );
            command_handle.cancel();
            self.report_progress(Progress::DidCancel);
        }
    }

    fn check_command(&self) -> Command {
        let args = build_args(
            &self.root,
            &self.output_path(),
            &self.config.include_directories,
            &self.config.args,
        );
        #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
        let program = "arch";
        #[cfg(not(all(target_arch = "aarch64", target_os = "macos")))]
        let program = &self.config.command;

        let mut command = Command::new(program);

        #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
        command.arg("-x86_64").arg(&self.config.command);

        command.args(args);

        command
    }

    fn send(&self, check_task: Message) {
        (self.sender)(check_task);
    }

    fn output_path(&self) -> AbsPathBuf {
        let mut rng = rand::thread_rng();
        self.tempdir.join(format!("{}.smx", rng.gen::<u16>()))
    }
}

struct JodGroupChild(GroupChild);

impl Drop for JodGroupChild {
    fn drop(&mut self) {
        _ = self.0.kill();
        _ = self.0.wait();
    }
}

/// A handle to a spcomp process used for fly-checking.
struct CommandHandle {
    /// The handle to the actual spcomp process. As we cannot cancel directly from with
    /// a read syscall dropping and therefore terminating the process is our best option.
    child: JodGroupChild,
    thread: stdx::thread::JoinHandle<io::Result<(bool, String)>>,
    receiver: Receiver<SpCompDiagnostic>,
    program: OsString,
    arguments: Vec<OsString>,
    current_dir: Option<PathBuf>,
}

impl fmt::Debug for CommandHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandHandle")
            .field("program", &self.program)
            .field("arguments", &self.arguments)
            .field("current_dir", &self.current_dir)
            .finish()
    }
}

impl CommandHandle {
    fn spawn(mut command: Command) -> std::io::Result<CommandHandle> {
        command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
        let mut child = command.group_spawn().map(JodGroupChild)?;

        let program = command.get_program().into();
        let arguments = command
            .get_args()
            .map(|arg| arg.into())
            .collect::<Vec<OsString>>();
        let current_dir = command.get_current_dir().map(|arg| arg.to_path_buf());

        let stdout = child.0.inner().stdout.take().unwrap();
        let stderr = child.0.inner().stderr.take().unwrap();

        let (sender, receiver) = unbounded();
        let actor = SpCompActor::new(sender, stdout, stderr);
        let thread = stdx::thread::Builder::new(stdx::thread::ThreadIntent::Worker)
            .name("CargoHandle".to_owned())
            .spawn(move || actor.run())
            .expect("failed to spawn thread");
        Ok(CommandHandle {
            program,
            arguments,
            current_dir,
            child,
            thread,
            receiver,
        })
    }

    fn cancel(mut self) {
        let _ = self.child.0.kill();
        let _ = self.child.0.wait();
    }

    fn join(mut self) -> io::Result<()> {
        let _ = self.child.0.kill();
        let exit_status = self.child.0.wait()?;
        let (read_at_least_one_message, error) = self.thread.join()?;
        if read_at_least_one_message || exit_status.success() {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, format!(
                "Cargo watcher failed, the command produced no valid metadata (exit code: {exit_status:?}):\n{error}"
            )))
        }
    }
}

struct SpCompActor {
    sender: Sender<SpCompDiagnostic>,
    stdout: ChildStdout,
    stderr: ChildStderr,
}

impl SpCompActor {
    fn new(
        sender: Sender<SpCompDiagnostic>,
        stdout: ChildStdout,
        stderr: ChildStderr,
    ) -> SpCompActor {
        SpCompActor {
            sender,
            stdout,
            stderr,
        }
    }

    fn run(self) -> io::Result<(bool, String)> {
        let mut stdout_errors = String::new();
        let mut stderr_errors = String::new();
        let mut read_at_least_one_stdout_message = false;
        let mut read_at_least_one_stderr_message = false;
        let process_line = |line: &str, error: &mut String| {
            if let Some(diag) = SpCompDiagnostic::try_from_line(line) {
                self.sender.send(diag).unwrap();
                return true;
            }
            error.push_str(line);
            error.push('\n');
            false
        };
        let output = streaming_output(
            self.stdout,
            self.stderr,
            &mut |line| {
                if process_line(line, &mut stdout_errors) {
                    read_at_least_one_stdout_message = true;
                }
            },
            &mut |line| {
                if process_line(line, &mut stderr_errors) {
                    read_at_least_one_stderr_message = true;
                }
            },
        );

        let read_at_least_one_message =
            read_at_least_one_stdout_message || read_at_least_one_stderr_message;
        let mut error = stdout_errors;
        error.push_str(&stderr_errors);
        match output {
            Ok(_) => Ok((read_at_least_one_message, error)),
            Err(e) => Err(io::Error::new(e.kind(), format!("{e:?}: {error}"))),
        }
    }
}
