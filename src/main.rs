use std::env;
use std::error::Error;

use lsp_server::Connection;
use sourcepawn_lsp::Server;

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    // Note that  we must have our logging only write out to stderr.
    eprintln!("Starting SourcePawn server");
    env::set_var("RUST_BACKTRACE", "1");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();
    Server::new(connection, env::current_dir()?).run()?;
    io_threads.join()?;

    // Shut down gracefully.
    eprintln!("Shutting down SourcePawn server");
    Ok(())
}
