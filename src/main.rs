#![forbid(unsafe_code)]
//! Command-line interface for the PlayHound library.

mod cli;

#[tokio::main]
async fn main() -> std::process::ExitCode {
    cli::main_entry().await
}
