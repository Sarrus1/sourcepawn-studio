use std::env;
use std::error::Error;

use clap::Parser;
use lsp_server::Connection;
use sourcepawn_lsp::Server;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// An implementation of the Language Server Protocol for SourcePawn
#[derive(Debug, Parser)]
#[clap(version)]
pub struct Opts {
    /// Enable AMXXPawn mode
    #[clap(short, long)]
    amxxpawn_mode: bool,
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let opts = Opts::parse();
    // Note that  we must have our logging only write out to stderr.
    eprintln!("Starting SourcePawn server version {}", VERSION);
    env::set_var("RUST_BACKTRACE", "1");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();
    Server::new(connection, opts.amxxpawn_mode).run()?;
    io_threads.join()?;

    // Shut down gracefully.
    eprintln!("Shutting down SourcePawn server");

    Ok(())
}
