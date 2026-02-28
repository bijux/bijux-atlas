#[test]
fn list_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["check", "list", "--format", "json"])
        .output()
        .expect("list json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let checks = payload
        .get("checks")
        .and_then(|v| v.as_array())
        .expect("checks array");
    assert!(!checks.is_empty());
    assert!(checks[0]
        .get("budget_ms")
        .and_then(|v| v.as_u64())
        .is_some());
}

#[test]
fn capabilities_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["capabilities", "--format", "json"])
        .output()
        .expect("capabilities json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert_eq!(
        payload
            .get("defaults")
            .and_then(|v| v.get("subprocess"))
            .and_then(|v| v.as_bool()),
        Some(false)
    );
}

#[test]
fn version_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .arg("version")
        .args(["--format", "json"])
        .output()
        .expect("version json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert_eq!(
        payload.get("name").and_then(|v| v.as_str()),
        Some("bijux-dev-atlas")
    );
    assert!(payload.get("version").and_then(|v| v.as_str()).is_some());
}

#[test]
fn help_inventory_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .arg("help")
        .args(["--format", "json"])
        .output()
        .expect("help inventory json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let commands = payload
        .get("commands")
        .and_then(|v| v.as_array())
        .expect("commands array");
    assert!(commands
        .iter()
        .any(|row| row.get("name").and_then(|v| v.as_str()) == Some("check")));
    assert!(commands
        .iter()
        .any(|row| row.get("name").and_then(|v| v.as_str()) == Some("ops")));
    assert!(commands
        .iter()
        .any(|row| row.get("name").and_then(|v| v.as_str()) == Some("policies")));
}

#[test]
fn configs_list_uses_registry_groups() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["configs", "list", "--format", "json"])
        .output()
        .expect("configs list json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(payload.get("kind").and_then(|v| v.as_str()), Some("configs"));
    let groups = payload
        .get("groups")
        .and_then(|v| v.as_array())
        .expect("groups array");
    assert!(!groups.is_empty());
    assert!(groups.iter().any(|row| {
        row.get("group").and_then(|v| v.as_str()) == Some("inventory")
            && row
                .get("tool_entrypoints")
                .and_then(|v| v.as_array())
                .is_some_and(|entries| !entries.is_empty())
    }));
    assert_eq!(
        payload
            .get("contract_surface")
            .and_then(|v| v.get("contract_count"))
            .and_then(|v| v.as_u64()),
        Some(38)
    );
    assert!(payload
        .get("contract_surface")
        .and_then(|v| v.get("registry_sha256"))
        .and_then(|v| v.as_str())
        .is_some());
}

#[test]
fn configs_explain_reports_registry_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "configs",
            "explain",
            "configs/rust/rustfmt.toml",
            "--format",
            "json",
        ])
        .output()
        .expect("configs explain json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(payload.get("kind").and_then(|v| v.as_str()), Some("configs_explain"));
    assert_eq!(payload.get("group").and_then(|v| v.as_str()), Some("rust"));
    assert_eq!(
        payload.get("owner").and_then(|v| v.as_str()),
        Some("rust-foundation")
    );
    assert!(payload
        .get("consumers")
        .and_then(|v| v.as_array())
        .is_some_and(|items| !items.is_empty()));
    assert!(payload.get("schema").is_some());
}

#[test]
fn plugin_metadata_matches_umbrella_contract_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .arg("--bijux-plugin-metadata")
        .output()
        .expect("plugin metadata");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid plugin metadata json");
    assert_eq!(payload["schema_version"].as_str(), Some("v1"));
    assert_eq!(payload["name"].as_str(), Some("bijux-dev-atlas"));
    assert!(payload["version"].as_str().is_some());
    assert!(payload["compatible_umbrella"].as_str().is_some());
    assert!(payload["compatible_umbrella_min"].as_str().is_some());
    assert!(payload["compatible_umbrella_max_exclusive"]
        .as_str()
        .is_some());
}

#[test]
fn explain_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "check",
            "explain",
            "checks_ops_surface_manifest",
            "--format",
            "json",
        ])
        .output()
        .expect("explain json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(
        payload.get("id").and_then(|v| v.as_str()),
        Some("checks_ops_surface_manifest")
    );
}

#[test]
fn slow_doctor_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["check", "doctor", "--format", "json"])
        .output()
        .expect("doctor json");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert!(payload.get("status").and_then(|v| v.as_str()).is_some());
    assert!(payload
        .get("registry_errors")
        .and_then(|v| v.as_array())
        .is_some());
}

#[test]
fn print_policies_outputs_stable_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .arg("--print-policies")
        .output()
        .expect("print policies");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_str()),
        Some("1")
    );
    assert!(payload.get("repo").is_some());
    assert!(payload.get("ops").is_some());
}

#[test]
fn print_boundaries_outputs_stable_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .arg("--print-boundaries")
        .output()
        .expect("print boundaries");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    let effects = payload
        .get("effects")
        .and_then(|v| v.as_array())
        .expect("effects array");
    assert!(effects
        .iter()
        .any(|row| row.get("id").and_then(|v| v.as_str()) == Some("network")));
}

#[test]
fn list_rejects_jsonl_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["check", "list", "--format", "jsonl"])
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
            "check",
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
        .args(["check", "list", "--format", "json"])
        .output()
        .expect("doctor nested cwd");
    assert!(output.status.success());
}

#[test]
fn check_list_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["check", "list", "--format", "json"])
        .output()
        .expect("check list json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let checks = payload
        .get("checks")
        .and_then(|v| v.as_array())
        .expect("checks array");
    assert!(!checks.is_empty());
    assert!(checks[0]
        .get("budget_ms")
        .and_then(|v| v.as_u64())
        .is_some());
}

#[test]
fn check_list_supports_json_alias() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["check", "list", "--json"])
        .output()
        .expect("check list --json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("checks").and_then(|v| v.as_array()).is_some());
}

#[test]
fn slow_check_doctor_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["check", "doctor", "--format", "json"])
        .output()
        .expect("check doctor json");
    assert!(
        !output.stdout.is_empty(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("status").and_then(|v| v.as_str()).is_some());
    assert!(payload
        .get("ops_doctor")
        .and_then(|v| v.as_object())
        .is_some());
    assert!(payload
        .get("docs_doctor")
        .and_then(|v| v.as_object())
        .is_some());
    assert!(payload
        .get("configs_doctor")
        .and_then(|v| v.as_object())
        .is_some());
    assert!(payload
        .get("control_plane_doctor")
        .and_then(|v| v.as_object())
        .is_some());
}

#[test]
fn check_registry_doctor_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["check", "registry", "doctor", "--format", "json"])
        .output()
        .expect("check registry doctor json");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_i64()),
        Some(1)
    );
    assert!(payload.get("status").and_then(|v| v.as_str()).is_some());
    assert!(payload.get("errors").and_then(|v| v.as_array()).is_some());
}

#[test]
fn check_run_supports_out_file() {
    let out = repo_root().join("artifacts/tests/check_run_output.json");
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    let _output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "check",
            "run",
            "--suite",
            "ci",
            "--format",
            "json",
            "--out",
            out.to_str().expect("out path"),
        ])
        .output()
        .expect("check run out");
    let written = fs::read_to_string(out).expect("read out file");
    let payload: serde_json::Value = serde_json::from_str(&written).expect("json file");
    assert!(payload.get("run_id").is_some());
    assert!(payload.get("capabilities").is_some());
    assert!(payload.get("results").and_then(|v| v.as_array()).is_some());
}

#[test]
fn check_list_accepts_ci_fast_suite_alias() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["check", "list", "--suite", "ci-fast", "--format", "json"])
        .output()
        .expect("check list ci-fast");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("checks").and_then(|v| v.as_array()).is_some());
}

#[test]
fn check_list_accepts_local_and_deep_suites() {
    for suite in ["local", "deep"] {
        let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
            .current_dir(repo_root())
            .args(["check", "list", "--suite", suite, "--format", "json"])
            .output()
            .expect("check list suite");
        assert!(output.status.success(), "suite `{suite}` failed");
    }
}

#[test]
fn workflows_validate_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["workflows", "validate", "--format", "json"])
        .output()
        .expect("workflows validate json");
    assert!(output.status.success());
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert!(payload.get("run_id").is_some());
    assert!(payload.get("command").and_then(|v| v.as_str()).is_some());
    assert!(payload.get("results").and_then(|v| v.as_array()).is_some());
}

#[test]
fn gates_list_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["gates", "list", "--format", "json"])
        .output()
        .expect("gates list json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("checks").and_then(|v| v.as_array()).is_some());
}

#[test]
fn policies_validate_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["policies", "validate", "--format", "json"])
        .output()
        .expect("policies validate json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_i64()),
        Some(1)
    );
    assert_eq!(payload.get("status").and_then(|v| v.as_str()), Some("ok"));
    assert!(payload
        .get("capabilities")
        .and_then(|v| v.as_object())
        .is_some());
}

#[test]
fn policies_list_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["policies", "list", "--format", "json"])
        .output()
        .expect("policies list");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn policies_explain_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["policies", "explain", "repo", "--format", "json"])
        .output()
        .expect("policies explain");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload.get("id").and_then(|v| v.as_str()), Some("repo"));
    assert!(payload.get("fields").is_some());
}

#[test]
fn policies_report_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["policies", "report", "--format", "json"])
        .output()
        .expect("policies report");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(
        payload.get("report_kind").and_then(|v| v.as_str()),
        Some("control_plane_policies")
    );
}
