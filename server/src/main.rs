//! Zappy server binary entrypoint.
//!
//! S01 scope: parse and validate the command line (RQ17 / AQ01). On bad or
//! missing arguments it prints [`cli::USAGE`] and exits non-zero. On a valid
//! line it echoes the resolved configuration and exits; the TCP event loop is
//! added in S02 and later.

use std::process::ExitCode;

use zappy_server::cli::{self, CliError};

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
            // Networking / game loop start in S02+.
            ExitCode::SUCCESS
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
