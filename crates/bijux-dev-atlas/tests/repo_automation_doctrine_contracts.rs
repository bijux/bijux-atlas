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
fn clients_root_directory_is_forbidden() {
    let root = repo_root();
    assert!(
        !root.join("clients").exists(),
        "`clients/` is forbidden; client products must live under crates/"
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
            if trimmed.contains("bash ./")
                || trimmed.contains("bash tutorials/")
                || trimmed.contains("bash ops/")
            {
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

#[test]
fn allowed_nonrust_policy_must_define_python_and_shell_boundaries() {
    let root = repo_root();
    let path = root.join("configs/governance/allowed-nonrust.json");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let json: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()));

    let python_allowed = json["python"]["allowed_zones"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let shell_allowed = json["shell"]["allowed_zones"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        python_allowed
            .iter()
            .any(|item| item.as_str() == Some("crates/**/python/**/*.py")),
        "allowed-nonrust policy must explicitly allow python in client SDK package zones"
    );
    assert!(
        python_allowed
            .iter()
            .any(|item| item.as_str() == Some("crates/**/tests/**/*.py")),
        "allowed-nonrust policy must explicitly allow python in client SDK test zones"
    );
    assert!(
        python_allowed
            .iter()
            .any(|item| item.as_str() == Some("crates/**/examples/**/*.py")),
        "allowed-nonrust policy must explicitly allow python in client SDK examples zones"
    );
    assert!(
        python_allowed
            .iter()
            .any(|item| item.as_str() == Some("crates/**/notebooks/**/*.ipynb")),
        "allowed-nonrust policy must explicitly allow client notebooks when needed"
    );
    assert!(
        shell_allowed.is_empty(),
        "allowed-nonrust policy must keep shell allowlist empty for repository automation"
    );
}

#[test]
fn repository_python_files_must_stay_in_allowed_crate_zones() {
    let root = repo_root();
    let output = std::process::Command::new("git")
        .args(["ls-files", "**/*.py"])
        .current_dir(&root)
        .output()
        .expect("git ls-files");
    assert!(output.status.success(), "git ls-files failed");
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    let mut violations = Vec::new();
    for path in stdout.lines() {
        let allowed = path.starts_with("crates/bijux-atlas-client-python/python/")
            || path.starts_with("crates/bijux-atlas-client-python/tests/")
            || path.starts_with("crates/bijux-atlas-client-python/examples/")
            || path == "ops/cli/perf/cli_ux_benchmark.py";
        if !allowed {
            violations.push(path.to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "python files outside approved crate zones are forbidden:\n{}",
        violations.join("\n")
    );
}

#[test]
fn repository_notebooks_must_stay_in_allowed_crate_zones() {
    let root = repo_root();
    let output = std::process::Command::new("git")
        .args(["ls-files", "**/*.ipynb"])
        .current_dir(&root)
        .output()
        .expect("git ls-files");
    assert!(output.status.success(), "git ls-files failed");
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    let mut violations = Vec::new();
    for path in stdout.lines() {
        if !path.starts_with("crates/bijux-atlas-client-python/notebooks/") {
            violations.push(path.to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "notebooks outside approved crate zones are forbidden:\n{}",
        violations.join("\n")
    );
}

#[test]
fn tracked_pycache_and_pyc_are_forbidden_repo_wide() {
    let root = repo_root();
    let output = std::process::Command::new("git")
        .args(["ls-files", "**/__pycache__/*", "**/*.pyc"])
        .current_dir(&root)
        .output()
        .expect("git ls-files");
    assert!(output.status.success(), "git ls-files failed");
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    assert!(
        stdout.trim().is_empty(),
        "tracked __pycache__ and .pyc files are forbidden:\n{}",
        stdout
    );
}

#[test]
fn legacy_clients_root_references_are_forbidden_in_docs_make_and_workflows() {
    let root = repo_root();
    let scan_roots = [
        root.join("docs"),
        root.join(".github/workflows"),
        root.join("Makefile"),
    ];
    let mut violations = Vec::new();
    let has_legacy_client_root_reference = |text: &str| {
        text.contains("cd clients/atlas-client")
            || text.contains("cd ./clients/atlas-client")
            || text.contains("`clients/atlas-client`")
            || text.contains("`clients/atlas-client/")
            || text.contains("clients/atlas-client/tests/")
            || text.contains("clients/atlas-client/examples/")
            || text.contains("clients/atlas-client/docs/")
            || text.contains("clients/atlas-client-usage-examples/")
    };
    for scan_root in scan_roots {
        if scan_root.is_file() {
            let text = std::fs::read_to_string(&scan_root).expect("read scan file");
            if has_legacy_client_root_reference(&text) {
                violations.push(scan_root.display().to_string());
            }
            continue;
        }
        if !scan_root.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(scan_root) {
            let entry = entry.expect("walk scan root");
            if !entry.file_type().is_file() {
                continue;
            }
            let text = std::fs::read_to_string(entry.path()).unwrap_or_default();
            if has_legacy_client_root_reference(&text) {
                violations.push(entry.path().display().to_string());
            }
        }
    }
    assert!(
        violations.is_empty(),
        "legacy client path references are forbidden:\n{}",
        violations.join("\n")
    );
}

#[test]
fn tutorials_python_and_script_paths_must_be_marked_forbidden_after_migration() {
    let root = repo_root();
    let path = root.join("configs/governance/allowed-nonrust.json");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let json: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()));
    let forbidden = json["python"]["forbidden_after_migration_complete"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    assert!(
        forbidden
            .iter()
            .any(|item| item.as_str() == Some("tutorials/**/*.py")),
        "tutorial python files must be marked forbidden after migration completion"
    );
    assert!(
        forbidden
            .iter()
            .any(|item| item.as_str() == Some("tutorials/scripts/**")),
        "tutorial script directories must be marked forbidden after migration completion"
    );
}

#[test]
fn tutorials_legacy_script_and_test_directories_must_not_exist() {
    let root = repo_root();
    assert!(
        !root.join("tutorials/scripts").exists(),
        "tutorials/scripts must be removed after dev-atlas parity"
    );
    assert!(
        !root.join("tutorials/tests").exists(),
        "tutorials/tests must be removed after Rust contract parity"
    );
}

#[test]
fn tutorials_tree_must_not_include_python_or_shell_sources() {
    let root = repo_root();
    let tutorials = root.join("tutorials");
    let mut violations = Vec::new();
    if tutorials.exists() {
        for entry in walkdir::WalkDir::new(&tutorials) {
            let entry = entry.expect("walk tutorials");
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let ext = path.extension().and_then(|v| v.to_str()).unwrap_or_default();
            if ext == "py" || ext == "sh" {
                violations.push(
                    path.strip_prefix(&root)
                        .expect("relative path")
                        .display()
                        .to_string(),
                );
            }
        }
    }
    assert!(
        violations.is_empty(),
        "tutorials directory must not include Python or shell sources:\n{}",
        violations.join("\n")
    );
}
