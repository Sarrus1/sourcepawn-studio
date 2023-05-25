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

use sourcepawn_lsp::Server;

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

    /// Write the logging output to FILE
    #[clap(long, name = "FILE", value_parser)]
    log_file: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let opts = Opts::parse();
    setup_logger(opts.clone());
    log::info!("Starting SourcePawn server version {}", VERSION);
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LIB_BACKTRACE", "0");
    let (connection, io_threads) = Connection::stdio();
    Server::new(connection, opts.amxxpawn_mode).run()?;
    io_threads.join()?;
    log::info!("Shutting down SourcePawn server");

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
        .level(verbosity_level)
        .filter(|metadata| metadata.target().contains("sourcepawn_lsp"))
        .chain(io::stderr());

    let logger = match opts.log_file {
        Some(log_file) => logger.chain(
            OpenOptions::new()
                .write(true)
                .create(true)
                .open(log_file)
                .expect("failed to open log file"),
        ),
        None => logger,
    };

    logger.apply().expect("failed to initialize logger");
}
