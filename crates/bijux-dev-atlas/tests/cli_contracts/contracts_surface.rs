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
fn contracts_snapshot_defaults_to_artifacts_root() {
    let out = repo_root().join("artifacts/contracts/docker/registry-snapshot.json");
    if out.exists() {
        fs::remove_file(&out).expect("remove prior snapshot");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "snapshot", "--domain", "docker"])
        .output()
        .expect("contracts snapshot default path");
    assert!(output.status.success());
    let written = fs::read_to_string(out).expect("read snapshot");
    let payload: serde_json::Value = serde_json::from_str(&written).expect("json file");
    assert_eq!(payload["domain"].as_str(), Some("docker"));
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
    assert!(stderr.contains("effect mode requires"));
    assert!(stderr.contains("--allow-subprocess"));
    assert!(stderr.contains("--allow-network"));
}

#[test]
fn contracts_ops_ci_uses_default_artifacts_root() {
    let out = repo_root().join("artifacts/contracts/ops/local/static/local/ops.json");
    let inventory = repo_root().join("artifacts/contracts/ops/local/static/local/ops.inventory.json");
    let maturity = repo_root().join("artifacts/contracts/ops/local/static/local/ops.maturity.json");
    if out.exists() {
        fs::remove_file(&out).expect("remove prior report");
    }
    if inventory.exists() {
        fs::remove_file(&inventory).expect("remove prior inventory");
    }
    if maturity.exists() {
        fs::remove_file(&maturity).expect("remove prior maturity");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .env("CI", "true")
        .args(["contracts", "ops", "--format", "json"])
        .output()
        .expect("contracts ops ci");
    assert!(output.status.success());
    let written = fs::read_to_string(out).expect("read generated default report");
    let payload: serde_json::Value = serde_json::from_str(&written).expect("json file");
    assert_eq!(payload["domain"].as_str(), Some("ops"));
    let inventory_payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(inventory).expect("inventory file"))
            .expect("inventory json");
    assert_eq!(inventory_payload["domain"].as_str(), Some("ops"));
    let maturity_payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(maturity).expect("maturity file"))
            .expect("maturity json");
    assert_eq!(maturity_payload["domain"].as_str(), Some("ops"));
    assert!(maturity_payload["maturity"].is_object());
}

#[test]
fn contracts_profile_changes_default_artifacts_root_segment() {
    let out = repo_root().join("artifacts/contracts/docker/ci/static/local/docker.json");
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    if out.exists() {
        fs::remove_file(&out).expect("remove prior report");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "docker", "--profile", "ci", "--format", "json"])
        .output()
        .expect("contracts docker with profile");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out).expect("read generated report"))
            .expect("json report");
    assert_eq!(payload["domain"].as_str(), Some("docker"));
}

#[test]
fn contracts_docker_effect_requires_only_selected_effect_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            "docker",
            "--mode",
            "effect",
            "--filter-contract",
            "DOCKER-100",
        ])
        .output()
        .expect("contracts docker effect mode");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("--allow-subprocess"));
    assert!(!stderr.contains("--allow-network"));
}

#[test]
fn contracts_ci_forbids_skip_without_explicit_override() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .env("CI", "true")
        .args([
            "contracts",
            "ops",
            "--skip",
            "OPS-ROOT-*",
            "--artifacts-root",
            "artifacts/tests/contracts-ci-skip",
        ])
        .output()
        .expect("contracts ops ci skip");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("forbid --skip"));
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

