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
fn ops_list_profiles_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "list-profiles", "--format", "json"])
        .output()
        .expect("ops list-profiles json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let rows = payload
        .get("rows")
        .and_then(|v| v.as_array())
        .expect("rows array");
    assert!(!rows.is_empty());
}

#[test]
fn ops_inventory_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "inventory", "--format", "json"])
        .output()
        .expect("ops inventory");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert!(payload.get("status").and_then(|v| v.as_str()).is_some());
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn ops_list_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "list", "--format", "json"])
        .output()
        .expect("ops list");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn validate_meta_command_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["validate", "--help"])
        .output()
        .expect("validate help");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(text.contains("--profile"));
    assert!(text.contains("--format"));
}

#[test]
fn ops_explain_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "explain", "render", "--format", "json"])
        .output()
        .expect("ops explain");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let rows = payload
        .get("rows")
        .and_then(|v| v.as_array())
        .expect("rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].get("action").and_then(|v| v.as_str()),
        Some("render")
    );
}

#[test]
fn ops_cleanup_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "cleanup", "--format", "json"])
        .output()
        .expect("ops cleanup");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
}

#[test]
fn ops_stack_status_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "stack", "status", "--format", "json"])
        .output()
        .expect("ops stack status");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("status k8s requires --allow-subprocess"));
}

#[test]
fn ops_stack_plan_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "stack", "plan", "--format", "json"])
        .output()
        .expect("ops stack plan");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn ops_k8s_test_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "k8s", "test", "--format", "json"])
        .output()
        .expect("ops k8s test");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("conformance requires --allow-subprocess"));
}

#[test]
fn ops_k8s_apply_requires_explicit_apply_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "k8s",
            "apply",
            "--allow-subprocess",
            "--allow-write",
            "--run-id",
            "ops_render_kind_golden",
            "--format",
            "json",
        ])
        .output()
        .expect("ops k8s apply");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("requires explicit --apply"));
}

#[test]
fn ops_k8s_dry_run_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "k8s",
            "dry-run",
            "--run-id",
            "ops_render_kind_golden",
            "--format",
            "json",
        ])
        .output()
        .expect("ops k8s dry-run");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("requires --allow-subprocess"));
}

#[test]
fn ops_k8s_logs_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "k8s", "logs", "--format", "json"])
        .output()
        .expect("ops k8s logs");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("k8s logs requires --allow-subprocess"));
}

#[test]
fn ops_k8s_port_forward_requires_allow_network() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "k8s",
            "port-forward",
            "--allow-subprocess",
            "--format",
            "json",
        ])
        .output()
        .expect("ops k8s port-forward");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("k8s port-forward requires --allow-network"));
}

#[test]
fn ops_load_run_requires_effect_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "load", "run", "mixed", "--format", "json"])
        .output()
        .expect("ops load run");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("load run requires --allow-subprocess"));
}

#[test]
fn ops_load_plan_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "load", "plan", "mixed", "--format", "json"])
        .output()
        .expect("ops load plan");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let rows = payload
        .get("rows")
        .and_then(|v| v.as_array())
        .expect("rows");
    assert_eq!(rows[0].get("suite").and_then(|v| v.as_str()), Some("mixed"));
}

#[test]
fn ops_docs_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "docs", "--format", "json"])
        .output()
        .expect("ops docs");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert!(payload.get("run_id").is_some());
    assert!(payload.get("results").and_then(|v| v.as_array()).is_some());
}

#[test]
fn ops_conformance_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "conformance", "--format", "json"])
        .output()
        .expect("ops conformance");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("conformance requires --allow-subprocess"));
}

#[test]
fn ops_report_requires_allow_write() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "report", "--format", "json"])
        .output()
        .expect("ops report");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("report requires --allow-write"));
}

#[test]
fn ops_report_writes_structured_report_under_artifacts() {
    let run_id = "ops_report_contract";
    let target = repo_root()
        .join("artifacts/reports/dev-atlas/ops")
        .join(format!("{run_id}.json"));
    let _ = fs::remove_file(&target);
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "report",
            "--allow-write",
            "--run-id",
            run_id,
            "--format",
            "json",
        ])
        .output()
        .expect("ops report write");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
    assert!(target.exists(), "expected report file {}", target.display());
    let written = fs::read_to_string(target).expect("read report");
    let report: serde_json::Value = serde_json::from_str(&written).expect("report json");
    assert_eq!(
        report.get("kind").and_then(|v| v.as_str()),
        Some("ops_report")
    );
    assert_eq!(report.get("run_id").and_then(|v| v.as_str()), Some(run_id));
}

#[test]
fn ops_generate_pins_index_check_fails_when_artifact_missing() {
    let run_id = "ops_pins_index_check_missing";
    let target = repo_root()
        .join("artifacts/atlas-dev/ops")
        .join(run_id)
        .join("generate/pins.index.json");
    let _ = fs::remove_file(&target);
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "generate",
            "pins-index",
            "--check",
            "--run-id",
            run_id,
            "--format",
            "json",
        ])
        .output()
        .expect("pins-index check");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("pins-index check failed: missing"));
}

#[test]
fn ops_generate_pins_index_check_passes_after_generation() {
    let run_id = "ops_pins_index_check_ok";
    let generate = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "generate",
            "pins-index",
            "--run-id",
            run_id,
            "--format",
            "json",
        ])
        .output()
        .expect("pins-index generate");
    assert!(generate.status.success());
    let check = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "generate",
            "pins-index",
            "--check",
            "--run-id",
            run_id,
            "--format",
            "json",
        ])
        .output()
        .expect("pins-index check");
    assert!(
        check.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&check.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&check.stdout).expect("valid json");
    assert_eq!(
        payload
            .get("summary")
            .and_then(|v| v.get("errors"))
            .and_then(|v| v.as_u64()),
        Some(0)
    );
}

#[test]
fn ops_explain_profile_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "explain-profile", "kind", "--format", "json"])
        .output()
        .expect("ops explain-profile json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let first = payload
        .get("rows")
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())
        .expect("first row");
    assert_eq!(first.get("name").and_then(|v| v.as_str()), Some("kind"));
}

#[test]
fn ops_list_actions_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "list-actions", "--format", "json"])
        .output()
        .expect("ops list-actions json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let rows = payload
        .get("rows")
        .and_then(|v| v.as_array())
        .expect("rows array");
    assert!(!rows.is_empty());
}

#[test]
fn ops_status_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "status", "--format", "json"])
        .output()
        .expect("ops status json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let rows = payload
        .get("rows")
        .and_then(|v| v.as_array())
        .expect("rows array");
    assert_eq!(rows.len(), 1);
}

#[test]
fn ops_doctor_and_validate_do_not_require_subprocess_flag() {
    for args in [
        ["ops", "doctor", "--format", "json"],
        ["ops", "validate", "--format", "json"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
            .current_dir(repo_root())
            .args(args)
            .output()
            .expect("ops command");
        let bytes = if output.stdout.is_empty() {
            &output.stderr
        } else {
            &output.stdout
        };
        let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
        assert!(payload.get("schema_version").is_some());
    }
}

#[test]
fn ops_list_tools_supports_json_without_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "list-tools", "--format", "json"])
        .output()
        .expect("ops list-tools");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn ops_tools_verify_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "tools", "verify", "--format", "json"])
        .output()
        .expect("ops tools verify");
    assert!(!output.status.success());
}

#[test]
fn ops_tools_doctor_runs_without_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "tools", "doctor", "--format", "json"])
        .output()
        .expect("ops tools doctor");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn ops_render_kind_check_supports_json_format_without_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops", "render", "--target", "kind", "--check", "--format", "json",
        ])
        .output()
        .expect("ops render kind check");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    let row = payload
        .get("rows")
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())
        .expect("row");
    let actual = serde_json::json!({
        "target": row.get("target").and_then(|v| v.as_str()).unwrap_or(""),
        "write_enabled": row.get("write_enabled").and_then(|v| v.as_bool()).unwrap_or(false),
        "check_only": row.get("check_only").and_then(|v| v.as_bool()).unwrap_or(false),
        "stdout_mode": row.get("stdout_mode").and_then(|v| v.as_bool()).unwrap_or(false),
    });
    let golden_path =
        repo_root().join("crates/bijux-dev-atlas/tests/goldens/ops_render_kind_contract.json");
    let golden_text = fs::read_to_string(golden_path).expect("golden");
    let golden: serde_json::Value = serde_json::from_str(&golden_text).expect("golden json");
    assert_eq!(actual, golden);
}

#[test]
fn ops_render_helm_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops", "render", "--target", "helm", "--check", "--format", "json",
        ])
        .output()
        .expect("ops render helm");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("helm render requires --allow-subprocess"));
}

#[test]
fn ops_render_write_requires_allow_write() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops", "render", "--target", "kind", "--write", "--format", "json",
        ])
        .output()
        .expect("ops render kind write");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("ops render --write requires --allow-write"));
}

#[test]
fn ops_render_kind_default_requires_allow_write() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "render", "--target", "kind", "--format", "json"])
        .output()
        .expect("ops render kind default");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("ops render write requires --allow-write"));
}

#[test]
fn ops_render_kind_writes_with_allow_write() {
    let run_id = "ops_render_kind_write_contract";
    let render_path = repo_root()
        .join("artifacts/ops")
        .join(run_id)
        .join("render/developer/kind/render.yaml");
    let _ = fs::remove_file(&render_path);
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "render",
            "--target",
            "kind",
            "--run-id",
            run_id,
            "--allow-write",
            "--format",
            "json",
        ])
        .output()
        .expect("ops render kind write");
    assert!(output.status.success());
    assert!(
        render_path.exists(),
        "expected render at {}",
        render_path.display()
    );
}

#[test]
fn ops_render_kustomize_is_forbidden() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "render",
            "--target",
            "kustomize",
            "--check",
            "--format",
            "json",
        ])
        .output()
        .expect("ops render kustomize");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("kustomize render is not enabled"));
}

#[test]
fn ops_install_apply_requires_allow_write() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "install",
            "--apply",
            "--allow-subprocess",
            "--format",
            "json",
        ])
        .output()
        .expect("ops install apply");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("install apply/kind requires --allow-write"));
}

#[test]
fn ops_status_pods_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "status", "--target", "pods", "--format", "json"])
        .output()
        .expect("ops status pods");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("status pods requires --allow-subprocess"));
}

#[test]
fn ops_pins_update_requires_allow_write() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "pins",
            "update",
            "--i-know-what-im-doing",
            "--allow-subprocess",
            "--format",
            "json",
        ])
        .output()
        .expect("ops pins update");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("pins update requires --allow-write"));
}

#[test]
fn ops_generate_surface_list_check_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "generate",
            "surface-list",
            "--check",
            "--format",
            "json",
        ])
        .output()
        .expect("ops generate surface-list check");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(
        payload
            .get("summary")
            .and_then(|v| v.get("errors"))
            .and_then(|v| v.as_u64()),
        Some(0)
    );
}
