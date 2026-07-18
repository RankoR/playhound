#![forbid(unsafe_code)]
//! Command-line entry point for repository maintenance tasks.

use std::{env, fs, path::PathBuf, process::ExitCode};

use playhound_xtask::{FixtureKind, sanitize};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("xtask: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or_else(usage)?;
    if command != "sanitize" {
        return Err(usage());
    }
    let kind: FixtureKind = args
        .next()
        .ok_or_else(usage)?
        .parse()
        .map_err(|message: &'static str| message.to_owned())?;
    let input = PathBuf::from(args.next().ok_or_else(usage)?);
    let output = PathBuf::from(args.next().ok_or_else(usage)?);
    if args.next().is_some() {
        return Err(usage());
    }

    let bytes =
        fs::read(&input).map_err(|error| format!("cannot read {}: {error}", input.display()))?;
    let value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid JSON in {}: {error}", input.display()))?;
    let sanitized = sanitize(kind, value)?;
    let bytes = serde_json::to_vec_pretty(&sanitized)
        .map_err(|error| format!("cannot encode sanitized fixture: {error}"))?;
    fs::write(&output, bytes)
        .map_err(|error| format!("cannot write {}: {error}", output.display()))?;
    Ok(())
}

fn usage() -> String {
    "usage: cargo run -p playhound-xtask -- sanitize <app|search|list|reviews|suggestions> <input.json> <output.json>".into()
}
