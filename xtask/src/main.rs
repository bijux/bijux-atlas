use std::env;
use std::fs;
use std::io::Write;
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

fn scan_rust_relaxations(root: &Path, out_path: &Path) -> Result<(), String> {
    let mut findings: Vec<String> = Vec::new();
    let scan_roots = [root.join("crates"), root.join("xtask")];

    fn walk(dir: &Path, out: &mut Vec<String>) -> Result<(), String> {
        for entry in fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))? {
            let entry = entry.map_err(|e| format!("read_dir entry {}: {e}", dir.display()))?;
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out)?;
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            if path
                .components()
                .any(|c| c.as_os_str().to_string_lossy() == "generated")
            {
                // Generated files are rewritten by contract generators; enforce relaxations
                // only in authored sources to keep exception tagging stable.
                continue;
            }
            let content =
                fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
            for (idx, line) in content.lines().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim();
                let exception_id = line
                    .split_whitespace()
                    .find(|p| p.starts_with("ATLAS-EXC-"))
                    .map(|s| s.trim_matches(|c: char| c == ',' || c == ';').to_string());
                if trimmed.contains("#[cfg(test)]") || trimmed.contains("#[cfg_attr(test") {
                    let exc = exception_id
                        .as_ref()
                        .map(|v| format!("\"{v}\""))
                        .unwrap_or_else(|| "null".to_string());
                    out.push(format!(
                        "{{\"source\":\"rust-ast\",\"pattern_id\":\"cfg_test_attribute\",\"requires_exception\":false,\"severity\":\"info\",\"file\":\"{}\",\"line\":{},\"exception_id\":{}}}",
                        path.display(),
                        line_no,
                        exc
                    ));
                }
                if trimmed.starts_with("#[allow(")
                    || trimmed.starts_with("#![allow(")
                    || (trimmed.starts_with("#[cfg_attr(") && trimmed.contains("allow("))
                {
                    let exc = exception_id
                        .as_ref()
                        .map(|v| format!("\"{v}\""))
                        .unwrap_or_else(|| "null".to_string());
                    out.push(format!(
                        "{{\"source\":\"rust-ast\",\"pattern_id\":\"allow_attribute\",\"requires_exception\":true,\"severity\":\"error\",\"file\":\"{}\",\"line\":{},\"exception_id\":{}}}",
                        path.display(),
                        line_no,
                        exc
                    ));
                }
            }
        }
        Ok(())
    }

    for dir in scan_roots {
        if dir.exists() {
            walk(&dir, &mut findings)?;
        }
    }
    findings.sort();
    let parent = out_path
        .parent()
        .ok_or_else(|| format!("invalid output path: {}", out_path.display()))?;
    fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    let mut f =
        fs::File::create(out_path).map_err(|e| format!("create {}: {e}", out_path.display()))?;
    writeln!(f, "{{").map_err(|e| e.to_string())?;
    writeln!(f, "  \"schema_version\": 1,").map_err(|e| e.to_string())?;
    writeln!(f, "  \"findings\": [").map_err(|e| e.to_string())?;
    for (idx, row) in findings.iter().enumerate() {
        let suffix = if idx + 1 == findings.len() { "" } else { "," };
        writeln!(f, "    {row}{suffix}").map_err(|e| e.to_string())?;
    }
    writeln!(f, "  ]").map_err(|e| e.to_string())?;
    writeln!(f, "}}").map_err(|e| e.to_string())?;
    println!("{}", out_path.display());
    Ok(())
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let arg = args.next().unwrap_or_else(|| "help".to_string());
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");

    let result = match arg.as_str() {
        "format-contracts" => run(root, "./scripts/contracts/format_contracts.py"),
        "generate-contracts" => run(root, "./scripts/contracts/generate_contract_artifacts.py"),
        "check-contracts" => run(root, "./scripts/contracts/check_all.sh"),
        "scan-relaxations" => {
            let out = args
                .next()
                .unwrap_or_else(|| "artifacts/policy/relaxations-rust.json".to_string());
            scan_rust_relaxations(root, &root.join(out))
        }
        "help" | "--help" | "-h" => {
            eprintln!("xtask commands:");
            eprintln!("  format-contracts");
            eprintln!("  generate-contracts");
            eprintln!("  check-contracts");
            eprintln!("  scan-relaxations [out-path]");
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
