// SPDX-License-Identifier: Apache-2.0

use crate::cli::{PackagesCommand, PackagesCommandArgs};
use crate::{emit_payload, resolve_repo_root};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command as ProcessCommand;

const PYTHON_PACKAGE_ROOT: &str = "crates/bijux-atlas-python";

fn collect_package_entries(repo_root: &Path) -> Result<Vec<String>, String> {
    let package_root = repo_root.join(PYTHON_PACKAGE_ROOT);
    if !package_root.exists() {
        return Err(format!(
            "python package root is missing: {}",
            package_root.display()
        ));
    }
    let output = ProcessCommand::new("git")
        .args(["ls-files", "crates/bijux-atlas-python"])
        .current_dir(repo_root)
        .output()
        .map_err(|err| format!("failed to execute git ls-files: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let mut entries = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if !line.is_empty() {
            entries.push(line.to_string());
        }
    }
    entries.sort();
    Ok(entries)
}

fn run_packages_list(args: &PackagesCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let entries = collect_package_entries(&repo_root)?;
    let python_files = entries.iter().filter(|path| path.ends_with(".py")).count();
    let notebook_files = entries
        .iter()
        .filter(|path| path.ends_with(".ipynb"))
        .count();
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "packages",
        "action": "list",
        "packages": [{
            "id": "bijux-atlas-python",
            "path": PYTHON_PACKAGE_ROOT,
            "file_count": entries.len(),
            "python_file_count": python_files,
            "notebook_file_count": notebook_files
        }]
    });
    Ok((emit_payload(args.format, args.out.clone(), &payload)?, 0))
}

fn run_packages_verify(args: &PackagesCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let entries = collect_package_entries(&repo_root)?;
    let required_paths = [
        "crates/bijux-atlas-python/pyproject.toml",
        "crates/bijux-atlas-python/README.md",
        "crates/bijux-atlas-python/VERSIONING.md",
        "crates/bijux-atlas-python/compatibility.json",
    ];
    let mut violations = Vec::new();
    for required in required_paths {
        if !repo_root.join(required).exists() {
            violations.push(format!("missing required package file: {required}"));
        }
    }
    for path in entries {
        if path.contains("/__pycache__/") || path.ends_with(".pyc") {
            violations.push(format!("forbidden python cache artifact: {path}"));
        }
        if path.ends_with(".py")
            && !path.starts_with("crates/bijux-atlas-python/python/")
            && !path.starts_with("crates/bijux-atlas-python/tests/python/")
            && !path.starts_with("crates/bijux-atlas-python/examples/")
        {
            violations.push(format!(
                "python file is outside approved package source/test/example zones: {path}"
            ));
        }
        if path.ends_with(".ipynb") && !path.starts_with("crates/bijux-atlas-python/notebooks/") {
            violations.push(format!(
                "notebook file is outside approved package notebooks zone: {path}"
            ));
        }
        if path.starts_with("crates/bijux-atlas-python/tools/") {
            violations.push(format!(
                "repo automation tooling is forbidden under package path: {path}"
            ));
        }
    }
    let readme = fs::read_to_string(repo_root.join("crates/bijux-atlas-python/README.md"))
        .map_err(|err| format!("failed to read package README: {err}"))?;
    if !readme.contains("Bijux Atlas") {
        violations
            .push("package README must describe the Bijux Atlas server requirement".to_string());
    }
    let success = violations.is_empty();
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "packages",
        "action": "verify",
        "package": "bijux-atlas-python",
        "success": success,
        "violations": violations
    });
    Ok((
        emit_payload(args.format, args.out.clone(), &payload)?,
        if success { 0 } else { 1 },
    ))
}

pub(crate) fn run_packages_command(quiet: bool, command: PackagesCommand) -> i32 {
    let run = match command {
        PackagesCommand::List(args) => run_packages_list(&args),
        PackagesCommand::Verify(args) => run_packages_verify(&args),
    };
    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                let _ = writeln!(io::stdout(), "{rendered}");
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas packages failed: {err}");
            1
        }
    }
}
