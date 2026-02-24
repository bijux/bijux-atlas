use assert_cmd::Command;

#[test]
fn config_json_workflow_is_parseable() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["--json", "config"])
        .output()
        .expect("run config");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("config output json");
    assert!(payload.get("workspace_config").is_some());
    assert!(payload.get("cache_dir").is_some());
}

#[test]
fn openapi_generate_workflow_writes_contract_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("openapi.generated.json");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["--json", "openapi", "generate", "--out"])
        .arg(&out)
        .output()
        .expect("run openapi generate");
    assert!(output.status.success());

    let raw = std::fs::read(&out).expect("openapi file");
    let parsed: serde_json::Value = serde_json::from_slice(&raw).expect("openapi json");
    assert_eq!(parsed["openapi"], "3.0.3");
    assert!(parsed.get("paths").is_some());
}
