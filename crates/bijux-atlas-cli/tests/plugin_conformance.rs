use serde_json::Value;
use std::process::Command;

#[test]
fn plugin_metadata_handshake_has_required_fields() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--bijux-plugin-metadata")
        .output()
        .expect("run plugin metadata command");
    assert!(output.status.success());

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid JSON metadata");
    for key in ["name", "version", "compatible_umbrella", "build_hash"] {
        assert!(payload.get(key).is_some(), "missing required field {key}");
    }
    assert_eq!(
        payload.get("name").and_then(Value::as_str),
        Some("bijux-atlas")
    );
}

#[test]
fn help_contains_standard_plugin_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(output.status.success());

    let text = String::from_utf8(output.stdout).expect("utf8 help");
    for needle in [
        "--json",
        "--quiet",
        "--verbose",
        "--trace",
        "--bijux-plugin-metadata",
    ] {
        assert!(text.contains(needle), "help missing {needle}");
    }
}

#[test]
fn atlas_namespace_help_is_stable() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["atlas", "--help"])
        .output()
        .expect("run atlas help");
    assert!(output.status.success());

    let text = String::from_utf8(output.stdout).expect("utf8 help");
    for needle in ["ingest", "serve", "catalog", "dataset", "openapi"] {
        assert!(text.contains(needle), "atlas help missing {needle}");
    }
}

#[test]
fn unknown_arguments_exit_with_usage_code() {
    let status = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--not-a-real-flag")
        .status()
        .expect("run with bad flag");
    assert_eq!(status.code(), Some(2));
}

#[test]
fn json_error_contract_is_stable() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["--json", "--not-a-real-flag"])
        .output()
        .expect("run with bad flag");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    assert!(stderr.contains("\"code\":\"usage_error\""));
    assert!(stderr.contains("\"message\":\"invalid command line arguments\""));
}
