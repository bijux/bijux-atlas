// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn walk_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(cursor) = stack.pop() {
        let entries =
            fs::read_dir(&cursor).map_err(|err| format!("read {} failed: {err}", cursor.display()))?;
        for entry in entries {
            let entry = entry.map_err(|err| format!("read entry failed: {err}"))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

#[test]
fn client_tools_directory_is_forbidden() {
    let root = repo_root();
    assert!(
        !root.join("packages/bijux-atlas-python/tools").exists(),
        "packages/bijux-atlas-python/tools must be removed; use dev-atlas clients commands"
    );
}

#[test]
fn client_docs_must_not_reference_local_script_paths() {
    let root = repo_root();
    let docs_dir = root.join("packages/bijux-atlas-python/docs");
    let mut violations = Vec::new();
    for path in walk_files(&docs_dir).expect("walk docs") {
        if path.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read docs file");
        if text.contains("tools/generate_docs.py")
            || text.contains("packages/bijux-atlas-python/tools/")
        {
            violations.push(path.display().to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "client docs must not reference local script paths:\n{}",
        violations.join("\n")
    );
}

#[test]
fn client_python_paths_must_stay_in_product_zones_only() {
    let root = repo_root();
    let output = std::process::Command::new("git")
        .args([
            "ls-files",
            "packages/bijux-atlas-python/**/**/*.py",
            "packages/bijux-atlas-python/**/**/*.ipynb",
        ])
        .current_dir(&root)
        .output()
        .expect("git ls-files");
    assert!(output.status.success(), "git ls-files failed");
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    let mut violations = Vec::new();
    for path in stdout.lines() {
        let allowed = path.starts_with("packages/bijux-atlas-python/src/bijux_atlas/")
            || path.starts_with("packages/bijux-atlas-python/tests/")
            || path.starts_with("packages/bijux-atlas-python/examples/")
            || path.starts_with("packages/bijux-atlas-python/notebooks/");
        if !allowed {
            violations.push(path.to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "client python/notebook paths must stay in product zones:\n{}",
        violations.join("\n")
    );
}

#[test]
fn client_pycache_and_pyc_must_not_be_tracked() {
    let root = repo_root();
    let output = std::process::Command::new("git")
        .args([
            "ls-files",
            "packages/bijux-atlas-python/**/__pycache__/*",
            "packages/bijux-atlas-python/**/*.pyc",
        ])
        .current_dir(&root)
        .output()
        .expect("git ls-files");
    assert!(output.status.success(), "git ls-files failed");
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    assert!(
        stdout.trim().is_empty(),
        "tracked client __pycache__ or .pyc files are forbidden:\n{}",
        stdout
    );
}

#[test]
fn client_pyproject_metadata_must_be_complete() {
    let root = repo_root();
    let pyproject = fs::read_to_string(root.join("packages/bijux-atlas-python/pyproject.toml"))
        .expect("read pyproject");
    for needle in [
        "[project]",
        "name = \"bijux-atlas\"",
        "version = ",
        "description = ",
        "requires-python = ",
        "[project.urls]",
    ] {
        assert!(
            pyproject.contains(needle),
            "pyproject missing required metadata token `{needle}`"
        );
    }
}

#[test]
fn client_docs_must_reference_runtime_endpoints() {
    let root = repo_root();
    let api_ref =
        fs::read_to_string(root.join("packages/bijux-atlas-python/docs/api-reference.md"))
            .expect("read api-reference.md");
    assert!(
        api_ref.contains("/v1/"),
        "client API reference must include runtime endpoint paths"
    );
}

#[test]
fn client_examples_must_target_tutorial_dataset_scenario() {
    let root = repo_root();
    let examples = root.join("packages/bijux-atlas-python/examples");
    let mut checked = 0usize;
    let mut violations = Vec::new();
    for path in walk_files(&examples).expect("walk examples") {
        if path.extension().and_then(|v| v.to_str()) != Some("py") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read example");
        checked += 1;
        let uses_dataset = text.contains("dataset=\"genes\"") || text.contains("dataset='genes'");
        if !uses_dataset {
            violations.push(path.display().to_string());
        }
    }
    assert!(checked > 0, "expected python examples to exist");
    assert!(
        violations.is_empty(),
        "examples must target tutorial dataset scenario (`genes`):\n{}",
        violations.join("\n")
    );
}

#[test]
fn client_tests_must_not_require_external_network() {
    let root = repo_root();
    let tests_dir = root.join("packages/bijux-atlas-python/tests");
    let mut violations = Vec::new();
    for path in walk_files(&tests_dir).expect("walk tests") {
        if path.extension().and_then(|v| v.to_str()) != Some("py") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read test file");
        let lower = text.to_ascii_lowercase();
        let has_external_call = lower.contains("requests.get(\"http")
            || lower.contains("requests.post(\"http")
            || lower.contains("urllib.request.urlopen(\"http")
            || lower.contains("httpx.get(\"http")
            || lower.contains("httpx.post(\"http");
        if has_external_call {
            violations.push(path.display().to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "client tests must stay offline unless explicitly tagged:\n{}",
        violations.join("\n")
    );
}

#[test]
fn client_tests_must_support_offline_mock_server_mode() {
    let root = repo_root();
    let integration =
        fs::read_to_string(root.join("packages/bijux-atlas-python/tests/test_integration.py"))
            .expect("read integration test");
    assert!(
        integration.contains("HTTPServer") && integration.contains("127.0.0.1"),
        "integration tests must include local mock server mode"
    );
}

#[test]
fn usage_examples_must_be_routed_through_docs_examples_page() {
    let root = repo_root();
    let page = root.join("docs/reference/examples/atlas-client-usage-examples.md");
    assert!(page.exists(), "usage examples page must exist");
    let text = fs::read_to_string(&page).expect("read examples page");
    assert!(
        text.contains("packages/bijux-atlas-python/examples/usage"),
        "usage examples page must point to packages/bijux-atlas-python/examples/usage"
    );
}

#[test]
fn client_tests_must_be_tagged_unit_integration_or_perf() {
    let root = repo_root();
    let tests_dir = root.join("packages/bijux-atlas-python/tests");
    let mut violations = Vec::new();
    for path in walk_files(&tests_dir).expect("walk tests") {
        if path.extension().and_then(|v| v.to_str()) != Some("py") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read test file");
        let has_scope_tag = text.contains("# test_scope: unit")
            || text.contains("# test_scope: integration")
            || text.contains("# test_scope: perf");
        if !has_scope_tag {
            violations.push(format!("{}: missing # test_scope tag", path.display()));
        }
    }
    assert!(
        violations.is_empty(),
        "client tests must be tagged unit/integration/perf:\n{}",
        violations.join("\n")
    );
}

#[test]
fn perf_tests_must_be_opt_in_and_integration_tests_must_require_env() {
    let root = repo_root();
    let tests_dir = root.join("packages/bijux-atlas-python/tests");
    let mut perf_violations = Vec::new();
    let mut integration_violations = Vec::new();
    for path in walk_files(&tests_dir).expect("walk tests") {
        if path.extension().and_then(|v| v.to_str()) != Some("py") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read test file");
        let filename = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        if text.contains("# test_scope: perf")
            && !text.contains("BIJUX_ATLAS_RUN_PERF")
            && !text.contains("skipUnless")
        {
            perf_violations.push(format!(
                "{}: perf tests must be guarded by BIJUX_ATLAS_RUN_PERF or skipUnless",
                path.display()
            ));
        }
        if text.contains("# test_scope: integration") && !text.contains("BIJUX_ATLAS_URL") {
            integration_violations.push(format!(
                "{}: integration tests must require BIJUX_ATLAS_URL",
                path.display()
            ));
        }
        if filename.contains("integration") && !text.contains("# test_scope: integration") {
            integration_violations.push(format!(
                "{}: integration-named file must carry integration scope tag",
                path.display()
            ));
        }
    }
    assert!(
        perf_violations.is_empty(),
        "perf test opt-in violations:\n{}",
        perf_violations.join("\n")
    );
    assert!(
        integration_violations.is_empty(),
        "integration test env gate violations:\n{}",
        integration_violations.join("\n")
    );
}

#[test]
fn examples_index_must_exist_and_reference_all_example_scripts() {
    let root = repo_root();
    let index_path = root.join("packages/bijux-atlas-python/examples/INDEX.md");
    assert!(index_path.exists(), "examples index must exist");
    let index = fs::read_to_string(&index_path).expect("read examples index");

    let examples = root.join("packages/bijux-atlas-python/examples");
    let mut scripts = Vec::new();
    for path in walk_files(&examples).expect("walk examples") {
        if path.extension().and_then(|v| v.to_str()) != Some("py") {
            continue;
        }
        let rel = path
            .strip_prefix(&examples)
            .expect("relative path")
            .to_string_lossy()
            .replace('\\', "/");
        scripts.push(rel);
    }
    scripts.sort();
    for script in scripts {
        assert!(
            index.contains(&format!("`{script}`")),
            "examples/INDEX.md must reference `{script}`"
        );
    }
}

#[test]
fn examples_must_include_header_and_stay_within_complexity_budget() {
    let root = repo_root();
    let examples = root.join("packages/bijux-atlas-python/examples");
    let mut violations = Vec::new();
    for path in walk_files(&examples).expect("walk examples") {
        if path.extension().and_then(|v| v.to_str()) != Some("py") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read example");
        let line_count = text.lines().count();
        if !text.contains("# Purpose:") || !text.contains("# Expected output:") {
            violations.push(format!("{}: missing required header", path.display()));
        }
        if line_count > 200 {
            violations.push(format!("{}: exceeds 200 LOC budget ({line_count})", path.display()));
        }
    }
    assert!(
        violations.is_empty(),
        "example quality violations:\n{}",
        violations.join("\n")
    );
}

#[test]
fn examples_must_use_documented_runtime_endpoint_variable() {
    let root = repo_root();
    let examples = root.join("packages/bijux-atlas-python/examples");
    let mut violations = Vec::new();
    for path in walk_files(&examples).expect("walk examples") {
        if path.extension().and_then(|v| v.to_str()) != Some("py") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read example");
        if !text.contains("BIJUX_ATLAS_URL") {
            violations.push(format!(
                "{}: examples must use BIJUX_ATLAS_URL (no hardcoded runtime endpoint)",
                path.display()
            ));
        }
    }

    let usage_readme =
        fs::read_to_string(root.join("packages/bijux-atlas-python/examples/usage/README.md"))
            .expect("read usage readme");
    assert!(
        usage_readme.contains("BIJUX_ATLAS_URL"),
        "usage README must document BIJUX_ATLAS_URL"
    );

    assert!(
        violations.is_empty(),
        "endpoint configuration violations:\n{}",
        violations.join("\n")
    );
}

#[test]
fn examples_must_not_use_external_network_clients_directly() {
    let root = repo_root();
    let examples = root.join("packages/bijux-atlas-python/examples");
    let mut violations = Vec::new();
    for path in walk_files(&examples).expect("walk examples") {
        if path.extension().and_then(|v| v.to_str()) != Some("py") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read example");
        let lower = text.to_ascii_lowercase();
        let has_external_client = lower.contains("import requests")
            || lower.contains("import httpx")
            || lower.contains("urllib.request")
            || lower.contains("socket.socket(");
        if has_external_client {
            violations.push(path.display().to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "examples must use atlas_client and avoid direct external network clients:\n{}",
        violations.join("\n")
    );
}

#[test]
fn client_test_policy_docs_must_exist() {
    let root = repo_root();
    assert!(
        root.join("docs/api/client-python/client-test-policy.md")
            .exists(),
        "client test policy doc must exist"
    );
    assert!(
        root.join("docs/api/client-python/client-test-troubleshooting.md")
            .exists(),
        "client test troubleshooting doc must exist"
    );
}

#[test]
fn make_tests_all_must_support_optional_client_python_lane() {
    let root = repo_root();
    let makefile = fs::read_to_string(root.join("make/root.mk")).expect("read make/root.mk");
    assert!(
        makefile.contains("tests run --mode all")
            && makefile.contains("INCLUDE_CLIENT_PYTHON")
            && makefile.contains("--include-client-python"),
        "make tests-all must optionally include client python tests via dev-atlas"
    );
}
