use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn list_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["list", "--format", "json"])
        .output()
        .expect("list json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("checks").and_then(|v| v.as_array()).is_some());
}

#[test]
fn explain_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["explain", "ops_surface_manifest", "--format", "json"])
        .output()
        .expect("explain json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(payload.get("id").and_then(|v| v.as_str()), Some("ops_surface_manifest"));
}

#[test]
fn doctor_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["doctor", "--format", "json"])
        .output()
        .expect("doctor json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(payload.get("status").and_then(|v| v.as_str()), Some("ok"));
    assert_eq!(payload.get("errors").and_then(|v| v.as_array()).map(|v| v.len()), Some(0));
}

#[test]
fn list_rejects_jsonl_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["list", "--format", "jsonl"])
        .output()
        .expect("list jsonl");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("jsonl output is not supported for list"));
}

#[test]
fn list_supports_out_file() {
    let out = repo_root().join("artifacts/tests/list_output.json");
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "list",
            "--format",
            "json",
            "--out",
            out.to_str().expect("out path"),
        ])
        .output()
        .expect("list out");
    assert!(output.status.success());
    let written = fs::read_to_string(out).expect("read out file");
    let payload: serde_json::Value = serde_json::from_str(&written).expect("json file");
    assert!(payload.get("checks").is_some());
}

#[test]
fn repo_root_discovery_works_from_nested_directory() {
    let nested = repo_root().join("crates/bijux-dev-atlas/src");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(nested)
        .arg("doctor")
        .output()
        .expect("doctor nested cwd");
    assert!(output.status.success());
}
