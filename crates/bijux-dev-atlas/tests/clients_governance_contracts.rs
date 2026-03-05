// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn client_tools_directory_is_forbidden() {
    let root = repo_root();
    assert!(
        !root.join("clients/atlas-client/tools").exists(),
        "clients/atlas-client/tools must be removed; use dev-atlas clients commands"
    );
}

#[test]
fn client_docs_must_not_reference_local_script_paths() {
    let root = repo_root();
    let docs_dir = root.join("clients/atlas-client/docs");
    let mut violations = Vec::new();
    for entry in walkdir::WalkDir::new(docs_dir) {
        let entry = entry.expect("walk docs");
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let text = fs::read_to_string(entry.path()).expect("read docs file");
        if text.contains("tools/generate_docs.py") || text.contains("clients/atlas-client/tools/") {
            violations.push(entry.path().display().to_string());
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
        .args(["ls-files", "clients/atlas-client/**/*.py", "clients/atlas-client/**/*.ipynb"])
        .current_dir(&root)
        .output()
        .expect("git ls-files");
    assert!(output.status.success(), "git ls-files failed");
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    let mut violations = Vec::new();
    for path in stdout.lines() {
        let allowed = path.starts_with("clients/atlas-client/atlas_client/")
            || path.starts_with("clients/atlas-client/tests/")
            || path.starts_with("clients/atlas-client/examples/")
            || path.starts_with("clients/atlas-client/notebooks/");
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
        .args(["ls-files", "clients/**/__pycache__/*", "clients/**/*.pyc"])
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
    let pyproject = fs::read_to_string(root.join("clients/atlas-client/pyproject.toml"))
        .expect("read pyproject");
    for needle in [
        "[project]",
        "name = \"atlas-client\"",
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
    let api_ref = fs::read_to_string(root.join("clients/atlas-client/docs/api-reference.md"))
        .expect("read api-reference.md");
    assert!(
        api_ref.contains("/v1/"),
        "client API reference must include runtime endpoint paths"
    );
}

#[test]
fn client_examples_must_target_tutorial_dataset_scenario() {
    let root = repo_root();
    let examples = root.join("clients/atlas-client/examples");
    let mut checked = 0usize;
    let mut violations = Vec::new();
    for entry in walkdir::WalkDir::new(examples) {
        let entry = entry.expect("walk examples");
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|v| v.to_str()) != Some("py") {
            continue;
        }
        let text = fs::read_to_string(entry.path()).expect("read example");
        checked += 1;
        let uses_dataset = text.contains("dataset=\"genes\"") || text.contains("dataset='genes'");
        if !uses_dataset {
            violations.push(entry.path().display().to_string());
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
    let tests_dir = root.join("clients/atlas-client/tests");
    let mut violations = Vec::new();
    for entry in walkdir::WalkDir::new(tests_dir) {
        let entry = entry.expect("walk tests");
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|v| v.to_str()) != Some("py") {
            continue;
        }
        let text = fs::read_to_string(entry.path()).expect("read test file");
        let lower = text.to_ascii_lowercase();
        let has_external_call = lower.contains("requests.get(\"http")
            || lower.contains("requests.post(\"http")
            || lower.contains("urllib.request.urlopen(\"http")
            || lower.contains("httpx.get(\"http")
            || lower.contains("httpx.post(\"http");
        if has_external_call {
            violations.push(entry.path().display().to_string());
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
    let integration = fs::read_to_string(root.join("clients/atlas-client/tests/test_integration.py"))
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
        text.contains("clients/atlas-client-usage-examples"),
        "usage examples page must point to clients/atlas-client-usage-examples"
    );
}
