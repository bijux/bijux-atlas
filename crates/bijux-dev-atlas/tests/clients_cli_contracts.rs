// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

static CLIENT_DOCS_TEST_LOCK: Mutex<()> = Mutex::new(());

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn clients_docs_generate_recreates_index_and_reference() {
    let _guard = CLIENT_DOCS_TEST_LOCK.lock().expect("lock");
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["clients", "docs-generate", "--client", "atlas-client", "--format", "json"])
        .output()
        .expect("clients docs-generate");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload["action"], "docs-generate");
    assert!(root.join("crates/bijux-atlas-client-python/docs/index.md").exists());
    assert!(root.join("crates/bijux-atlas-client-python/docs/api-reference.md").exists());
    assert!(root.join("crates/bijux-atlas-client-python/docs/version-compatibility-matrix.md").exists());
}

#[test]
fn clients_docs_verify_passes_for_generated_files() {
    let _guard = CLIENT_DOCS_TEST_LOCK.lock().expect("lock");
    let root = repo_root();
    let generated = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["clients", "docs-generate", "--client", "atlas-client", "--format", "json"])
        .output()
        .expect("clients docs-generate");
    assert!(generated.status.success(), "{}", String::from_utf8_lossy(&generated.stderr));
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["clients", "docs-verify", "--client", "atlas-client", "--format", "json"])
        .output()
        .expect("clients docs-verify");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload["success"], true);
}

#[test]
fn clients_examples_verify_passes_without_local_script_references() {
    let _guard = CLIENT_DOCS_TEST_LOCK.lock().expect("lock");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["clients", "examples-verify", "--client", "atlas-client", "--format", "json"])
        .output()
        .expect("clients examples-verify");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload["success"], true);
}

#[test]
fn clients_examples_run_writes_evidence_bundle() {
    let _guard = CLIENT_DOCS_TEST_LOCK.lock().expect("lock");
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["clients", "examples-run", "--client", "atlas-client", "--format", "json"])
        .output()
        .expect("clients examples-run");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    let evidence = payload["evidence"].as_str().expect("evidence path");
    assert!(PathBuf::from(evidence).exists(), "evidence bundle must exist");
}

#[test]
fn clients_schema_verify_passes_for_docs_model() {
    let _guard = CLIENT_DOCS_TEST_LOCK.lock().expect("lock");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["clients", "schema-verify", "--client", "atlas-client", "--format", "json"])
        .output()
        .expect("clients schema-verify");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload["success"], true);
}

#[test]
fn clients_compat_matrix_verify_passes() {
    let _guard = CLIENT_DOCS_TEST_LOCK.lock().expect("lock");
    let root = repo_root();
    let generated = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["clients", "docs-generate", "--client", "atlas-client", "--format", "json"])
        .output()
        .expect("clients docs-generate");
    assert!(generated.status.success(), "{}", String::from_utf8_lossy(&generated.stderr));
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(root)
        .args([
            "clients",
            "compat-matrix",
            "verify",
            "--client",
            "atlas-client",
            "--format",
            "json",
        ])
        .output()
        .expect("clients compat-matrix verify");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload["success"], true);
}

#[test]
fn clients_docs_generation_is_deterministic() {
    let _guard = CLIENT_DOCS_TEST_LOCK.lock().expect("lock");
    let root = repo_root();
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
            .current_dir(&root)
            .args(["clients", "docs-generate", "--client", "atlas-client", "--format", "json"])
            .output()
            .expect("clients docs-generate")
    };
    let first = run();
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    let index_one = std::fs::read_to_string(root.join("crates/bijux-atlas-client-python/docs/index.md"))
        .expect("read index one");
    let api_one = std::fs::read_to_string(root.join("crates/bijux-atlas-client-python/docs/api-reference.md"))
        .expect("read api one");
    let matrix_one = std::fs::read_to_string(
        root.join("crates/bijux-atlas-client-python/docs/version-compatibility-matrix.md"),
    )
    .expect("read matrix one");

    let second = run();
    assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
    let index_two = std::fs::read_to_string(root.join("crates/bijux-atlas-client-python/docs/index.md"))
        .expect("read index two");
    let api_two = std::fs::read_to_string(root.join("crates/bijux-atlas-client-python/docs/api-reference.md"))
        .expect("read api two");
    let matrix_two = std::fs::read_to_string(
        root.join("crates/bijux-atlas-client-python/docs/version-compatibility-matrix.md"),
    )
    .expect("read matrix two");

    assert_eq!(index_one, index_two);
    assert_eq!(api_one, api_two);
    assert_eq!(matrix_one, matrix_two);
}

#[test]
fn clients_verify_text_uses_nextest_style_summary() {
    let _guard = CLIENT_DOCS_TEST_LOCK.lock().expect("lock");
    let root = repo_root();
    let generated = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["clients", "docs-generate", "--client", "atlas-client", "--format", "json"])
        .output()
        .expect("clients docs-generate");
    assert!(generated.status.success(), "{}", String::from_utf8_lossy(&generated.stderr));

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(root)
        .args(["clients", "verify", "--client", "atlas-client"])
        .output()
        .expect("clients verify text");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let text = String::from_utf8(output.stdout).expect("utf8");
    assert!(text.contains("PASS"), "verify text should include PASS rows");
    assert!(text.contains("summary: total="), "verify text should include summary line");
}

#[test]
fn clients_verify_writes_json_and_markdown_evidence() {
    let _guard = CLIENT_DOCS_TEST_LOCK.lock().expect("lock");
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["clients", "verify", "--client", "atlas-client", "--format", "json"])
        .output()
        .expect("clients verify");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    let evidence_json = payload["evidence"]["json"].as_str().expect("evidence json path");
    let evidence_md = payload["evidence"]["markdown"]
        .as_str()
        .expect("evidence markdown path");
    assert!(PathBuf::from(evidence_json).exists(), "json evidence must exist");
    assert!(PathBuf::from(evidence_md).exists(), "markdown evidence must exist");
}

#[test]
fn clients_python_test_reports_missing_lockfile() {
    let _guard = CLIENT_DOCS_TEST_LOCK.lock().expect("lock");
    let root = repo_root();
    let lock = root.join("crates/bijux-atlas-client-python/requirements.lock");
    assert!(lock.exists(), "requirements.lock must exist");
    let backup = root.join("crates/bijux-atlas-client-python/requirements.lock.bak-test");
    std::fs::rename(&lock, &backup).expect("rename lock");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["clients", "python", "test", "--client", "atlas-client", "--format", "json"])
        .output()
        .expect("clients python test");
    std::fs::rename(&backup, &lock).expect("restore lock");
    assert!(
        !output.status.success(),
        "command should fail without deterministic lockfile"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing deterministic lockfile"),
        "stderr should mention missing deterministic lockfile: {stderr}"
    );
}
