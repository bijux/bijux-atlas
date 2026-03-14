#![forbid(unsafe_code)]

use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::process::{Command, ExitCode};

use bijux_cli::api::runtime::run_cli_from_env;

fn decode_argv() -> Result<Vec<String>, OsString> {
    env::args_os()
        .map(|value| value.into_string())
        .collect::<Result<Vec<_>, _>>()
}

fn dev_atlas_forwarded_args(argv: &[String]) -> Option<Vec<String>> {
    if argv.len() >= 4 && argv[1] == "help" && argv[2] == "dev" && argv[3] == "atlas" {
        let mut forwarded = argv[4..].to_vec();
        forwarded.push("--help".to_string());
        return Some(forwarded);
    }

    if argv.len() >= 3 && argv[1] == "dev" && argv[2] == "atlas" {
        return Some(argv[3..].to_vec());
    }

    None
}

fn emit_process_output(output: &std::process::Output) {
    if !output.stdout.is_empty() {
        let _ = io::stdout().write_all(&output.stdout);
    }
    if !output.stderr.is_empty() {
        let _ = io::stderr().write_all(&output.stderr);
    }
}

fn normalize_process_exit_code(code: i32) -> u8 {
    if code <= 0 {
        return u8::from(code != 0);
    }
    if code > i32::from(u8::MAX) {
        return u8::MAX;
    }
    code as u8
}

fn run_dev_atlas(forwarded_args: &[String]) -> ExitCode {
    match Command::new("bijux-dev-atlas")
        .args(forwarded_args)
        .output()
    {
        Ok(output) => {
            emit_process_output(&output);
            ExitCode::from(normalize_process_exit_code(
                output.status.code().unwrap_or(1),
            ))
        }
        Err(error) => {
            let _ = writeln!(
                io::stderr(),
                "failed to run `bijux dev atlas` via `bijux-dev-atlas`: {error}"
            );
            let _ = writeln!(
                io::stderr(),
                "install with `cargo install bijux-dev-atlas` or `pip install bijux-dev-atlas`"
            );
            ExitCode::from(1)
        }
    }
}

fn main() -> ExitCode {
    let argv = match decode_argv() {
        Ok(argv) => argv,
        Err(_) => {
            let _ = writeln!(io::stderr(), "invalid UTF-8 argument in argv");
            return ExitCode::from(2);
        }
    };

    let Some(forwarded_args) = dev_atlas_forwarded_args(&argv) else {
        return run_cli_from_env();
    };

    run_dev_atlas(&forwarded_args)
}
