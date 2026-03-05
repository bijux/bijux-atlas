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
        !root.join("crates/bijux-atlas-client-python/tools").exists(),
        "crates/bijux-atlas-client-python/tools must be removed; use dev-atlas clients commands"
    );
}

#[test]
fn client_docs_must_not_reference_local_script_paths() {
    let root = repo_root();
    let docs_dir = root.join("crates/bijux-atlas-client-python/docs");
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
        if text.contains("tools/generate_docs.py") || text.contains("crates/bijux-atlas-client-python/tools/") {
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
        .args(["ls-files", "crates/bijux-atlas-client-python/**/**/*.py", "crates/bijux-atlas-client-python/**/**/*.ipynb"])
        .current_dir(&root)
        .output()
        .expect("git ls-files");
    assert!(output.status.success(), "git ls-files failed");
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    let mut violations = Vec::new();
    for path in stdout.lines() {
        let allowed = path.starts_with("crates/bijux-atlas-client-python/python/atlas_client/")
            || path.starts_with("crates/bijux-atlas-client-python/tests/")
            || path.starts_with("crates/bijux-atlas-client-python/examples/")
            || path.starts_with("crates/bijux-atlas-client-python/notebooks/");
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
            "crates/bijux-atlas-client-python/**/__pycache__/*",
            "crates/bijux-atlas-client-python/**/*.pyc",
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
    let pyproject = fs::read_to_string(root.join("crates/bijux-atlas-client-python/pyproject.toml"))
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
    let api_ref = fs::read_to_string(root.join("crates/bijux-atlas-client-python/docs/api-reference.md"))
        .expect("read api-reference.md");
    assert!(
        api_ref.contains("/v1/"),
        "client API reference must include runtime endpoint paths"
    );
}

#[test]
fn client_examples_must_target_tutorial_dataset_scenario() {
    let root = repo_root();
    let examples = root.join("crates/bijux-atlas-client-python/examples");
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
    let tests_dir = root.join("crates/bijux-atlas-client-python/tests");
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
    let integration = fs::read_to_string(root.join("crates/bijux-atlas-client-python/tests/test_integration.py"))
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
        text.contains("crates/bijux-atlas-client-python/examples/usage"),
        "usage examples page must point to crates/bijux-atlas-client-python/examples/usage"
    );
}

#[test]
fn examples_index_must_exist_and_reference_all_example_scripts() {
    let root = repo_root();
    let index_path = root.join("crates/bijux-atlas-client-python/examples/INDEX.md");
    assert!(index_path.exists(), "examples index must exist");
    let index = fs::read_to_string(&index_path).expect("read examples index");

    let examples = root.join("crates/bijux-atlas-client-python/examples");
    let mut scripts = Vec::new();
    for entry in walkdir::WalkDir::new(&examples) {
        let entry = entry.expect("walk examples");
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|v| v.to_str()) != Some("py") {
            continue;
        }
        let rel = entry
            .path()
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
    let examples = root.join("crates/bijux-atlas-client-python/examples");
    let mut violations = Vec::new();
    for entry in walkdir::WalkDir::new(&examples) {
        let entry = entry.expect("walk examples");
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|v| v.to_str()) != Some("py") {
            continue;
        }
        let text = fs::read_to_string(entry.path()).expect("read example");
        let line_count = text.lines().count();
        if !text.contains("# Purpose:") || !text.contains("# Expected output:") {
            violations.push(format!("{}: missing required header", entry.path().display()));
        }
        if line_count > 200 {
            violations.push(format!(
                "{}: exceeds 200 LOC budget ({line_count})",
                entry.path().display()
            ));
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
    let examples = root.join("crates/bijux-atlas-client-python/examples");
    let mut violations = Vec::new();
    for entry in walkdir::WalkDir::new(&examples) {
        let entry = entry.expect("walk examples");
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|v| v.to_str()) != Some("py") {
            continue;
        }
        let text = fs::read_to_string(entry.path()).expect("read example");
        if !text.contains("ATLAS_BASE_URL") {
            violations.push(format!(
                "{}: examples must use ATLAS_BASE_URL (no hardcoded runtime endpoint)",
                entry.path().display()
            ));
        }
    }

    let usage_readme = fs::read_to_string(
        root.join("crates/bijux-atlas-client-python/examples/usage/README.md"),
    )
    .expect("read usage readme");
    assert!(
        usage_readme.contains("ATLAS_BASE_URL"),
        "usage README must document ATLAS_BASE_URL"
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
    let examples = root.join("crates/bijux-atlas-client-python/examples");
    let mut violations = Vec::new();
    for entry in walkdir::WalkDir::new(&examples) {
        let entry = entry.expect("walk examples");
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|v| v.to_str()) != Some("py") {
            continue;
        }
        let text = fs::read_to_string(entry.path()).expect("read example");
        let lower = text.to_ascii_lowercase();
        let has_external_client = lower.contains("import requests")
            || lower.contains("import httpx")
            || lower.contains("urllib.request")
            || lower.contains("socket.socket(");
        if has_external_client {
            violations.push(entry.path().display().to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "examples must use atlas_client and avoid direct external network clients:\n{}",
        violations.join("\n")
    );
}
