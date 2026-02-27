// SPDX-License-Identifier: Apache-2.0

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
fn contracts_ops_list_includes_tests_by_default() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "ops", "--list", "--format", "json"])
        .output()
        .expect("contracts ops list");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let contracts = payload["contracts"].as_array().expect("contracts array");
    assert!(!contracts.is_empty());
    let tests = contracts[0]["tests"].as_array().expect("tests array");
    assert!(!tests.is_empty());
}

#[test]
fn contracts_ops_list_contains_curated_contract_ids() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "ops", "--list", "--format", "json"])
        .output()
        .expect("contracts ops list");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let ids = payload["contracts"]
        .as_array()
        .expect("contracts array")
        .iter()
        .filter_map(|row| row["id"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for expected in [
        "OPS-ROOT-001",
        "OPS-INV-001",
        "OPS-SCHEMA-001",
        "OPS-STACK-001",
    ] {
        assert!(
            ids.contains(expected),
            "missing curated contract id {expected}"
        );
    }
}

#[test]
fn contracts_ops_supports_junit_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "ops", "--format", "junit"])
        .output()
        .expect("contracts ops junit");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(text.contains("<testsuite"));
    assert!(text.contains("contracts.ops"));
}

#[test]
fn contracts_all_lists_all_domains() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "all", "--list", "--format", "json"])
        .output()
        .expect("contracts all list");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let domains = payload["contracts"]
        .as_array()
        .expect("contracts array")
        .iter()
        .filter_map(|row| row["domain"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(domains.contains("docker"));
    assert!(domains.contains("make"));
    assert!(domains.contains("ops"));
}

#[test]
fn contracts_make_runs_and_reports_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "make", "--format", "json"])
        .output()
        .expect("contracts make");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(payload["domain"].as_str(), Some("make"));
    assert_eq!(payload["summary"]["fail"].as_u64(), Some(0));
}

#[test]
fn contracts_snapshot_writes_ops_registry_file() {
    let out = repo_root().join("artifacts/tests/contracts-ops-snapshot.json");
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            "snapshot",
            "--domain",
            "ops",
            "--out",
            out.to_str().expect("out path"),
        ])
        .output()
        .expect("contracts snapshot");
    assert!(output.status.success());
    let written = fs::read_to_string(out).expect("read out file");
    let payload: serde_json::Value = serde_json::from_str(&written).expect("json file");
    assert_eq!(payload["domain"].as_str(), Some("ops"));
    assert!(payload["contracts"].is_array());
}

#[test]
fn contracts_ops_supports_filter_contract_alias() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            "ops",
            "--format",
            "json",
            "--filter-contract",
            "OPS-ROOT-017",
        ])
        .output()
        .expect("contracts ops filter-contract");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let contracts = payload["contracts"].as_array().expect("contracts array");
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0]["id"].as_str(), Some("OPS-ROOT-017"));
}

#[test]
fn contracts_ops_explain_includes_mapped_gate() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            "ops",
            "--explain",
            "OPS-ROOT-017",
            "--format",
            "json",
        ])
        .output()
        .expect("contracts ops explain");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload["mapped_gate"].as_str().is_some());
}

#[test]
fn contracts_ops_explain_test_reports_effects_and_io() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            "ops",
            "--explain-test",
            "ops.root_surface.required_commands_exist",
            "--format",
            "json",
        ])
        .output()
        .expect("contracts ops explain-test");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(
        payload["test_id"].as_str(),
        Some("ops.root_surface.required_commands_exist")
    );
    assert!(payload["inputs_read"].as_array().is_some());
    assert!(payload["outputs_written"].as_array().is_some());
    assert!(payload["effects_required"].as_array().is_some());
}

#[test]
fn contracts_docker_supports_json_and_junit_sidecar_outputs() {
    let json_out = repo_root().join("artifacts/tests/contracts-docker-report.json");
    let junit_out = repo_root().join("artifacts/tests/contracts-docker-report.xml");
    if let Some(parent) = json_out.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            "docker",
            "--format",
            "human",
            "--json-out",
            json_out.to_str().expect("json out"),
            "--junit-out",
            junit_out.to_str().expect("junit out"),
        ])
        .output()
        .expect("contracts docker sidecar outputs");
    assert!(output.status.success());
    let human = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(human.contains("Contracts: docker"));
    let json_payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(json_out).expect("json out"))
            .expect("json report");
    assert_eq!(json_payload["domain"].as_str(), Some("docker"));
    let junit_text = fs::read_to_string(junit_out).expect("junit out");
    assert!(junit_text.contains("<testsuite"));
}

#[test]
fn contracts_ops_effect_mode_requires_explicit_allow_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "ops", "--mode", "effect", "--format", "json"])
        .output()
        .expect("contracts ops effect mode");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("effect mode requires both --allow-subprocess and --allow-network"));
}

#[test]
fn contracts_ops_ci_requires_artifacts_root() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .env("CI", "true")
        .args(["contracts", "ops", "--format", "json"])
        .output()
        .expect("contracts ops ci");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("CI contracts runs require --artifacts-root"));
}

#[test]
fn contracts_ops_json_report_matches_schema() {
    let artifacts_root = repo_root().join("artifacts/tests/contracts-json-schema");
    fs::create_dir_all(&artifacts_root).expect("mkdir artifacts");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            "ops",
            "--mode",
            "static",
            "--filter-contract",
            "OPS-ROOT-001",
            "--format",
            "json",
            "--artifacts-root",
            artifacts_root.to_str().expect("artifacts root"),
        ])
        .output()
        .expect("contracts ops json report");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(payload["schema_version"].as_u64(), Some(1));
    assert_eq!(payload["domain"].as_str(), Some("ops"));
    assert_eq!(payload["mode"].as_str(), Some("static"));
    assert!(payload["summary"]["contracts"].as_u64().is_some());
    assert!(payload["summary"]["tests"].as_u64().is_some());
    let contracts = payload["contracts"].as_array().expect("contracts array");
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0]["id"].as_str(), Some("OPS-ROOT-001"));
    assert!(matches!(
        contracts[0]["status"].as_str(),
        Some("PASS" | "FAIL" | "SKIP" | "ERROR")
    ));
    let tests = payload["tests"].as_array().expect("tests array");
    assert!(!tests.is_empty());
    for case in tests {
        assert!(case["contract_id"].as_str().is_some());
        assert!(case["contract_title"].as_str().is_some());
        assert!(case["test_id"].as_str().is_some());
        assert!(case["test_title"].as_str().is_some());
        assert!(matches!(
            case["kind"].as_str(),
            Some("pure" | "subprocess" | "network")
        ));
        assert!(matches!(
            case["status"].as_str(),
            Some("PASS" | "FAIL" | "SKIP" | "ERROR")
        ));
        assert!(case["note"].is_string() || case["note"].is_null());
        assert!(case["violations"].as_array().is_some());
    }
}

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

#[test]
fn docker_build_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docker", "build", "--format", "json"])
        .output()
        .expect("docker build");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("docker build requires --allow-subprocess"));
}

#[test]
fn docker_check_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docker", "check", "--allow-subprocess", "--format", "json"])
        .output()
        .expect("docker check");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn docker_smoke_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docker", "smoke", "--format", "json"])
        .output()
        .expect("docker smoke");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("docker smoke requires --allow-subprocess"));
}

#[test]
fn docker_scan_requires_allow_network() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docker", "scan", "--allow-subprocess", "--format", "json"])
        .output()
        .expect("docker scan");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("docker scan requires --allow-network"));
}

#[test]
fn docker_policy_check_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docker", "policy", "check", "--format", "json"])
        .output()
        .expect("docker policy check");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert_eq!(
        payload.get("action").and_then(|v| v.as_str()),
        Some("policy_check")
    );
}

#[test]
fn docker_lock_requires_allow_write() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docker", "lock", "--format", "json"])
        .output()
        .expect("docker lock");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("docker lock requires --allow-write"));
}

#[test]
fn build_bin_requires_effect_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["build", "bin", "--format", "json"])
        .output()
        .expect("build bin");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("build bin requires --allow-subprocess"));
}

#[test]
#[ignore = "slow"]
fn build_bin_writes_manifest_when_effects_enabled() {
    let repo = repo_root();
    let manifest = repo.join("artifacts/dist/bin/manifest.json");
    let _ = fs::remove_file(&manifest);
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&repo)
        .args([
            "build",
            "bin",
            "--allow-subprocess",
            "--allow-write",
            "--format",
            "json",
            "--run-id",
            "build_bin_contract",
        ])
        .output()
        .expect("build bin");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(payload.get("action").and_then(|v| v.as_str()), Some("bin"));
    assert!(
        manifest.exists(),
        "manifest should exist: {}",
        manifest.display()
    );
    let manifest_payload: serde_json::Value =
        serde_json::from_slice(&fs::read(manifest).expect("read manifest")).expect("manifest json");
    assert_eq!(
        manifest_payload.get("kind").and_then(|v| v.as_str()),
        Some("build_bin_manifest")
    );
}

#[test]
fn build_clean_requires_allow_write() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["build", "clean", "--format", "json"])
        .output()
        .expect("build clean");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("build clean requires --allow-write"));
}

#[test]
fn build_dist_requires_effect_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["build", "dist", "--format", "json"])
        .output()
        .expect("build dist");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("build dist requires --allow-subprocess"));
}

#[test]
fn build_doctor_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["build", "doctor", "--format", "json"])
        .output()
        .expect("build doctor");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert_eq!(
        payload.get("action").and_then(|v| v.as_str()),
        Some("doctor")
    );
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn build_plan_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["build", "plan", "--format", "json"])
        .output()
        .expect("build plan");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid json");
    assert_eq!(payload.get("action").and_then(|v| v.as_str()), Some("plan"));
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn build_verify_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["build", "verify", "--format", "json"])
        .output()
        .expect("build verify");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("build verify requires --allow-subprocess"));
}

#[test]
fn build_meta_requires_allow_write() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["build", "meta", "--format", "json"])
        .output()
        .expect("build meta");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("build meta requires --allow-write"));
}
