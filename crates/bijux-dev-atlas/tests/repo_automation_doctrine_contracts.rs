// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn tools_root_directory_is_forbidden() {
    let root = repo_root();
    assert!(
        !root.join("tools").exists(),
        "`tools/` is forbidden; migrate automation into bijux-dev-atlas commands"
    );
}

#[test]
fn scripts_root_directory_is_forbidden() {
    let root = repo_root();
    assert!(
        !root.join("scripts").exists(),
        "`scripts/` is forbidden; migrate automation into bijux-dev-atlas commands"
    );
}

#[test]
fn root_shell_and_python_files_are_forbidden() {
    let root = repo_root();
    let mut violations = Vec::new();
    for entry in fs::read_dir(&root).expect("read repo root") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        match path.extension().and_then(|v| v.to_str()) {
            Some("sh") | Some("py") => {
                violations.push(
                    path.strip_prefix(&root)
                        .expect("relative path")
                        .display()
                        .to_string(),
                );
            }
            _ => {}
        }
    }
    assert!(
        violations.is_empty(),
        "root-level shell/python files are forbidden: {}",
        violations.join(", ")
    );
}

#[test]
fn workflows_must_not_use_bash_c_pipeline() {
    let root = repo_root();
    let workflows = root.join(".github/workflows");
    let mut violations = Vec::new();
    for entry in fs::read_dir(&workflows).expect("read workflows") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|v| v.to_str()) != Some("yml") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read workflow file");
        if text.contains("bash -c") {
            violations.push(
                path.strip_prefix(&root)
                    .expect("relative path")
                    .display()
                    .to_string(),
            );
        }
    }
    assert!(
        violations.is_empty(),
        "`bash -c` pipelines are forbidden in workflows: {}",
        violations.join(", ")
    );
}

#[test]
fn workflows_must_not_execute_repo_bash_scripts() {
    let root = repo_root();
    let workflows = root.join(".github/workflows");
    let mut violations = Vec::new();
    for entry in fs::read_dir(&workflows).expect("read workflows") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|v| v.to_str()) != Some("yml") {
            continue;
        }
        let rel = path
            .strip_prefix(&root)
            .expect("relative path")
            .display()
            .to_string();
        let text = fs::read_to_string(&path).expect("read workflow file");
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.contains("bash ./") || trimmed.contains("bash tutorials/") || trimmed.contains("bash ops/") {
                violations.push(format!("{rel}: {trimmed}"));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "workflow steps must use bijux-dev-atlas commands instead of repo bash scripts:\n{}",
        violations.join("\n")
    );
}

fn read_lines(path: &PathBuf) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect()
}

#[test]
fn root_makefile_must_not_contain_shell_parsing_logic() {
    let root = repo_root();
    let makefile = root.join("Makefile");
    let text = fs::read_to_string(&makefile).expect("read root Makefile");
    for forbidden in ["jq ", "rg ", "sed ", "awk "] {
        assert!(
            !text.contains(forbidden),
            "root Makefile must stay delegation-only and must not contain `{forbidden}`"
        );
    }
}

#[test]
fn root_makefile_must_stay_include_only() {
    let root = repo_root();
    let makefile = root.join("Makefile");
    let text = fs::read_to_string(&makefile).expect("read root Makefile");
    let mut violations = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("include ") {
            continue;
        }
        violations.push(format!("line {}: {trimmed}", index + 1));
    }
    assert!(
        violations.is_empty(),
        "root Makefile must remain include-only:\n{}",
        violations.join("\n")
    );
}

#[test]
fn workflow_python_usage_must_be_allowlisted() {
    let root = repo_root();
    let workflows = root.join(".github/workflows");
    let allowlist_path = root.join("configs/governance/workflow-python-allowlist.txt");
    let allowlist = read_lines(&allowlist_path);
    let mut violations = Vec::new();
    for entry in fs::read_dir(&workflows).expect("read workflows") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|v| v.to_str()) != Some("yml") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read workflow file");
        if !(text.contains("python ") || text.contains("python3 ")) {
            continue;
        }
        let rel = path
            .strip_prefix(&root)
            .expect("relative path")
            .display()
            .to_string();
        if !allowlist.iter().any(|item| item == &rel) {
            violations.push(rel);
        }
    }
    assert!(
        violations.is_empty(),
        "workflows with python commands must be explicitly allowlisted in {}:\n{}",
        allowlist_path.display(),
        violations.join("\n")
    );
}

#[test]
fn executable_files_outside_dev_atlas_must_be_allowlisted() {
    let root = repo_root();
    let allowlist_path = root.join("configs/governance/repo-executable-allowlist.txt");
    let allowlist = read_lines(&allowlist_path);
    let output = std::process::Command::new("git")
        .args(["ls-files", "--stage"])
        .current_dir(&root)
        .output()
        .expect("run git ls-files --stage");
    assert!(output.status.success(), "git ls-files --stage failed");
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    let mut violations = Vec::new();
    for line in stdout.lines() {
        let mut parts = line.split_whitespace();
        let mode = parts.next().unwrap_or_default();
        let _hash = parts.next();
        let _stage = parts.next();
        let path = parts.next().unwrap_or_default();
        if mode != "100755" {
            continue;
        }
        if path.starts_with("crates/bijux-dev-atlas/") {
            continue;
        }
        if !allowlist.iter().any(|item| item == path) {
            violations.push(path.to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "new executable files outside crates/bijux-dev-atlas require explicit allowlist updates in {}:\n{}",
        allowlist_path.display(),
        violations.join("\n")
    );
}

#[test]
fn bin_utilities_outside_dev_atlas_must_be_allowlisted() {
    let root = repo_root();
    let allowlist_path = root.join("configs/governance/repo-bin-allowlist.txt");
    let allowlist = read_lines(&allowlist_path);
    let output = std::process::Command::new("git")
        .args(["ls-files"])
        .current_dir(&root)
        .output()
        .expect("run git ls-files");
    assert!(output.status.success(), "git ls-files failed");
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    let mut violations = Vec::new();
    for path in stdout.lines() {
        if !path.contains("/bin/") {
            continue;
        }
        if path.starts_with("crates/bijux-dev-atlas/") {
            continue;
        }
        if !allowlist.iter().any(|item| item == path) {
            violations.push(path.to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "bin utilities outside crates/bijux-dev-atlas must be explicitly allowlisted in {}:\n{}",
        allowlist_path.display(),
        violations.join("\n")
    );
}
