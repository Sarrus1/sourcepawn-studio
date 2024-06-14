use clap::ArgAction;
use clap::Parser;
use log::LevelFilter;
use lsp_server::Connection;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;
use std::time::SystemTime;

use sourcepawn_studio::GlobalState;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// An implementation of the Language Server Protocol for SourcePawn
#[derive(Debug, Parser, Clone)]
#[clap(version)]
pub struct Opts {
    /// Enable AMXXPawn mode
    #[clap(short, long)]
    amxxpawn_mode: bool,

    /// Increase message verbosity (-vv for max verbosity)
    #[clap(short, long, action = ArgAction::Count)]
    verbosity: u8,

    /// No output printed to stderr
    #[clap(short, long)]
    quiet: bool,

    /// Disable telemetry
    #[clap(short, long)]
    disable_telemetry: bool,

    /// Write the logging output to FILE
    #[clap(long, name = "FILE", value_parser)]
    log_file: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let opts = Opts::parse();
    setup_logger(opts.clone());

    let _guard = if !opts.disable_telemetry {
        log::info!("Telemetry is enabled. To disable it, use the --disable-telemetry flag.");
        Some(sentry::init(("https://621f3ac25899467a92414f0cabd31346@o4505249792262144.ingest.sentry.io/4505249800519680", sentry::ClientOptions {
            release: sentry::release_name!(),
            attach_stacktrace: true,
            server_name: Some("sourcepawn-studio".into()),
            ..Default::default()
        })))
    } else {
        log::info!("Telemetry is disabled.");
        None
    };

    log::info!("Starting sourcepawn-studio version {}", VERSION);
    env::set_var("RUST_BACKTRACE", "full");
    env::set_var("RUST_LIB_BACKTRACE", "full");
    let (connection, threads) = Connection::stdio();
    GlobalState::new(connection, opts.amxxpawn_mode).run()?;
    threads.join()?;
    // Use a print to avoid a panic since the output channel might have been closed
    eprintln!("Shutting down sourcepawn-studio");

    Ok(())
}

fn setup_logger(opts: Opts) {
    let verbosity_level = if !opts.quiet {
        match opts.verbosity {
            0 => LevelFilter::Error,
            1 => LevelFilter::Warn,
            2 => LevelFilter::Info,
            3 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    } else {
        LevelFilter::Off
    };

    let logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                humantime::format_rfc3339_millis(SystemTime::now()),
                record.level(),
                message
            ))
        })
        .level(LevelFilter::Error)
        .level_for("base_db", verbosity_level)
        .level_for("flychek", verbosity_level)
        .level_for("hir", verbosity_level)
        .level_for("hir_def", verbosity_level)
        .level_for("ide", verbosity_level)
        .level_for("ide_db", verbosity_level)
        .level_for("ide_diagnostics", verbosity_level)
        .level_for("paths", verbosity_level)
        .level_for("preprocessor", verbosity_level)
        .level_for("profile", verbosity_level)
        .level_for("sourcepawn_studio", verbosity_level)
        .level_for("stdx", verbosity_level)
        .level_for("syntax", verbosity_level)
        .level_for("test_utils", verbosity_level)
        .level_for("vfs", verbosity_level)
        .level_for("vfs_notify", verbosity_level)
        .chain(io::stderr());

    let logger = match opts.log_file {
        Some(log_file) => logger.chain(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(log_file)
                .expect("failed to open log file"),
        ),
        None => logger,
    };

    logger.apply().expect("failed to initialize logger");
}
