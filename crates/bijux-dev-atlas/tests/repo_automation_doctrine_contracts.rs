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
fn make_wrapper_modules_must_not_contain_shell_parsing_or_script_calls() {
    let root = repo_root();
    let wrapper_targets = [
        ("make/root.mk", "checks-all:"),
        ("make/root.mk", "release-plan:"),
        ("make/root.mk", "openapi-generate:"),
        ("make/contracts.mk", "contracts-all:"),
        ("make/docs.mk", "docs-build:"),
        ("make/docs.mk", "docs-serve:"),
        ("make/ops.mk", "ops-validate:"),
    ];
    let forbidden_tokens = ["jq ", "rg ", "sed ", "awk ", "python ", "python3 ", "node "];
    let mut violations = Vec::new();
    for (module_rel, target_header) in wrapper_targets {
        let module = root.join(module_rel);
        let text = fs::read_to_string(&module).expect("read make module");
        let mut in_target = false;
        for line in text.lines() {
            if line.starts_with(target_header) {
                in_target = true;
                continue;
            }
            if in_target && !line.starts_with('\t') {
                break;
            }
            if !in_target || !line.starts_with('\t') {
                continue;
            }
            let trimmed = line.trim();
            for token in forbidden_tokens {
                if trimmed.contains(token) {
                    violations.push(format!(
                        "{} target `{}` contains forbidden token `{token}`",
                        module_rel,
                        target_header.trim_end_matches(':')
                    ));
                }
            }
            if trimmed.contains("tools/") || trimmed.contains("scripts/") {
                violations.push(format!(
                    "{} target `{}` references forbidden tools/scripts path",
                    module_rel,
                    target_header.trim_end_matches(':')
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "make wrapper modules must remain delegation-only:\n{}",
        violations.join("\n")
    );
}

#[test]
fn required_make_wrappers_must_delegate_to_dev_atlas() {
    let root = repo_root();
    let root_mk = fs::read_to_string(root.join("make/root.mk")).expect("read make/root.mk");
    let contracts_mk =
        fs::read_to_string(root.join("make/contracts.mk")).expect("read make/contracts.mk");
    let docs_mk = fs::read_to_string(root.join("make/docs.mk")).expect("read make/docs.mk");
    let ops_mk = fs::read_to_string(root.join("make/ops.mk")).expect("read make/ops.mk");

    assert!(root_mk.contains("checks-all:"), "missing checks-all target");
    assert!(
        root_mk.contains("release-plan:") && root_mk.contains("$(DEV_ATLAS) release plan"),
        "release-plan target must delegate to bijux-dev-atlas release plan"
    );
    assert!(
        root_mk.contains("openapi-generate:") && root_mk.contains("$(DEV_ATLAS) api contract"),
        "openapi-generate target must delegate through bijux-dev-atlas"
    );
    assert!(
        contracts_mk.contains("contracts-all:")
            && contracts_mk.contains("$(DEV_ATLAS)")
            && contracts_mk.contains("contract run --mode all"),
        "contracts-all target must delegate to bijux-dev-atlas contract run --mode all"
    );
    assert!(
        docs_mk.contains("docs-build:") && docs_mk.contains("$(DEV_ATLAS) docs build"),
        "docs-build target must delegate to bijux-dev-atlas docs build"
    );
    assert!(
        docs_mk.contains("docs-serve:") && docs_mk.contains("$(DEV_ATLAS) docs serve"),
        "docs-serve target must delegate to bijux-dev-atlas docs serve"
    );
    assert!(
        ops_mk.contains("ops-validate:") && ops_mk.contains("$(DEV_ATLAS) ops validate"),
        "ops-validate target must delegate to bijux-dev-atlas ops validate"
    );
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
