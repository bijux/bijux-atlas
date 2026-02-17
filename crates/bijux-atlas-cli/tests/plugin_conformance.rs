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
    for key in [
        "schema_version",
        "name",
        "version",
        "compatible_umbrella",
        "compatible_umbrella_min",
        "compatible_umbrella_max_exclusive",
        "build_hash",
    ] {
        assert!(payload.get(key).is_some(), "missing required field {key}");
    }
    assert_eq!(
        payload.get("name").and_then(Value::as_str),
        Some("bijux-atlas")
    );
}

#[test]
fn plugin_metadata_matches_snapshot_contract() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--bijux-plugin-metadata")
        .output()
        .expect("run plugin metadata command");
    assert!(output.status.success());
    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid JSON metadata");

    let snapshot_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs/PLUGIN_METADATA_SNAPSHOT.json");
    let snapshot_text = std::fs::read_to_string(snapshot_path).expect("read metadata snapshot");
    let snapshot_text = snapshot_text.replace("__CARGO_PKG_VERSION__", env!("CARGO_PKG_VERSION"));
    let expected: Value = serde_json::from_str(&snapshot_text).expect("parse metadata snapshot");
    assert_eq!(payload, expected);
}

#[test]
fn plugin_contract_doc_includes_required_sections() {
    let text = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/PLUGIN_CONTRACT.md"),
    )
    .expect("read plugin contract doc");
    for needle in [
        "--bijux-plugin-metadata",
        "--json",
        "--quiet",
        "--verbose",
        "--trace",
        "compatible_umbrella_min",
        "compatible_umbrella_max_exclusive",
        "build_hash",
        "PLUGIN_METADATA_SNAPSHOT.json",
    ] {
        assert!(
            text.contains(needle),
            "plugin contract doc missing `{needle}`"
        );
    }
}

#[test]
fn atlas_validate_command_supports_deep_mode() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["atlas", "validate", "--help"])
        .output()
        .expect("run atlas validate help");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("utf8 help");
    assert!(text.contains("--deep"));
    assert!(text.contains("--release"));
    assert!(text.contains("--species"));
    assert!(text.contains("--assembly"));
}

#[test]
fn umbrella_version_compatibility_is_enforced() {
    let bad = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["--json", "--umbrella-version", "0.2.1", "version"])
        .output()
        .expect("run with incompatible umbrella version");
    assert_eq!(bad.status.code(), Some(2));
    let stderr = String::from_utf8(bad.stderr).expect("stderr utf8");
    assert!(stderr.contains("\"code\":\"umbrella_incompatible\""));
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
