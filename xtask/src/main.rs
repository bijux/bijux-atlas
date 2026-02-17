use std::env;
use std::path::Path;
use std::process::{Command, ExitCode};

fn run(root: &Path, cmd: &str) -> Result<(), String> {
    let status = Command::new("sh")
        .arg("-lc")
        .arg(cmd)
        .current_dir(root)
        .status()
        .map_err(|e| format!("failed to run `{cmd}`: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed: {cmd}"))
    }
}

fn main() -> ExitCode {
    let arg = env::args().nth(1).unwrap_or_else(|| "help".to_string());
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");

    let result = match arg.as_str() {
        "format-contracts" => run(root, "./scripts/contracts/format_contracts.py"),
        "generate-contracts" => run(root, "./scripts/contracts/generate_contract_artifacts.py"),
        "check-contracts" => run(root, "./scripts/contracts/check_all.sh"),
        "help" | "--help" | "-h" => {
            eprintln!("xtask commands:");
            eprintln!("  format-contracts");
            eprintln!("  generate-contracts");
            eprintln!("  check-contracts");
            Ok(())
        }
        _ => Err(format!(
            "unknown xtask command: {arg} (try `cargo run --manifest-path xtask/Cargo.toml -- help`)"
        )),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}
