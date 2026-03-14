#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    bijux_cli::api::runtime::run_cli_from_env()
}
