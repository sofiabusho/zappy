//! Zappy server binary entrypoint.
//!
//! Parses the command line (S01 / RQ17 / AQ01). On a valid line it starts the
//! multiplexed TCP event loop (S02/S03 / RQ16 / RQ19 / AQ02 / AQ14), which
//! accepts clients, sends `WELCOME\n`, and completes the team handshake.
//! Gameplay arrives in later tickets.

use std::process::ExitCode;

use zappy_server::cli::{self, CliError};
use zappy_server::net;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match cli::parse(&args) {
        Ok(config) => {
            println!(
                "server: port={} world={}x{} teams={} clients_per_team={} t={}",
                config.port,
                config.width,
                config.height,
                config.teams.join(","),
                config.clients_per_team,
                config.t,
            );
            match net::serve(&config) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("server: {err}");
                    ExitCode::FAILURE
                }
            }
        }
        Err(err) => {
            // Bare `./server` shows usage only; other errors add a diagnostic
            // line on stderr so the reason is clear without hiding the usage.
            if !matches!(err, CliError::NoArgs) {
                eprintln!("{err}");
            }
            print!("{}", cli::USAGE);
            ExitCode::FAILURE
        }
    }
}
