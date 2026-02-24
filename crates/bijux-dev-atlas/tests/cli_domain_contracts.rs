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
fn ops_stack_status_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "stack", "status", "--format", "json"])
        .output()
        .expect("ops stack status");
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
fn ops_load_run_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "load", "run", "--format", "json"])
        .output()
        .expect("ops load run");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let rows = payload
        .get("rows")
        .and_then(|v| v.as_array())
        .expect("rows");
    assert_eq!(
        rows[0].get("action").and_then(|v| v.as_str()),
        Some("load-run")
    );
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
fn docs_validate_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "validate", "--format", "json"])
        .output()
        .expect("docs validate json");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert!(payload.get("errors").and_then(|v| v.as_array()).is_some());
}

#[test]
fn docs_check_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "check", "--format", "json"])
        .output()
        .expect("docs check");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("docs check requires --allow-subprocess"));
}

#[test]
fn docs_serve_requires_allow_network() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "serve", "--allow-subprocess", "--format", "json"])
        .output()
        .expect("docs serve");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("docs serve requires --allow-network"));
}

#[test]
fn docs_clean_requires_allow_write() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "clean", "--format", "json"])
        .output()
        .expect("docs clean");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("docs clean requires --allow-write"));
}

#[test]
fn docs_inventory_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "inventory", "--format", "json"])
        .output()
        .expect("docs inventory json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("pages").and_then(|v| v.as_array()).is_some());
    assert!(payload.get("nav").and_then(|v| v.as_array()).is_some());
}

#[test]
fn docs_verify_contracts_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "verify-contracts", "--format", "json"])
        .output()
        .expect("docs verify-contracts json");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert!(payload.get("summary").is_some());
}

#[test]
fn docs_inventory_respects_include_drafts_flag() {
    let fixture_root = repo_root().join("crates/bijux-dev-atlas/tests/fixtures/docs-mini");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "docs",
            "inventory",
            "--repo-root",
            fixture_root.to_str().expect("fixture root"),
            "--include-drafts",
            "--format",
            "json",
        ])
        .output()
        .expect("docs inventory include drafts");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let pages = payload
        .get("pages")
        .and_then(|v| v.as_array())
        .expect("pages");
    assert!(pages.iter().any(|row| {
        row.get("path")
            .and_then(|v| v.as_str())
            .is_some_and(|p| p == "_drafts/draft.md")
    }));
}

#[test]
fn docs_validate_strict_escalates_warnings_to_errors() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "validate", "--strict", "--format", "json"])
        .output()
        .expect("docs validate strict");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert_eq!(
        payload
            .get("options")
            .and_then(|v| v.get("strict"))
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert!(payload.get("error_code").is_some());
}

#[test]
fn docs_doctor_fixture_json_matches_golden() {
    let fixture_root = repo_root().join("crates/bijux-dev-atlas/tests/fixtures/docs-mini");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "docs",
            "doctor",
            "--repo-root",
            fixture_root.to_str().expect("fixture root"),
            "--run-id",
            "docs_fixture",
            "--format",
            "json",
        ])
        .output()
        .expect("docs doctor fixture");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    payload["duration_ms"] = serde_json::json!(0);
    let actual = serde_json::to_string_pretty(&payload).expect("json");
    let golden_path =
        repo_root().join("crates/bijux-dev-atlas/tests/goldens/docs_doctor_fixture.json");
    let golden = fs::read_to_string(golden_path).expect("golden");
    assert_eq!(actual.trim(), golden.trim());
}

#[test]
fn configs_inventory_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["configs", "inventory", "--format", "json"])
        .output()
        .expect("configs inventory json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn configs_list_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["configs", "list", "--format", "json"])
        .output()
        .expect("configs list json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn configs_inventory_writes_artifact_when_allow_write_enabled() {
    let artifact_root = repo_root().join("artifacts/tests/configs_inventory_write");
    if artifact_root.exists() {
        let _ = std::fs::remove_dir_all(&artifact_root);
    }
    std::fs::create_dir_all(&artifact_root).expect("mkdir");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "configs",
            "inventory",
            "--allow-write",
            "--artifacts-root",
            artifact_root.to_str().expect("artifact root"),
            "--run-id",
            "configs_inventory_test",
            "--format",
            "json",
        ])
        .output()
        .expect("configs inventory write");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let path = payload
        .get("artifacts")
        .and_then(|v| v.get("inventory"))
        .and_then(|v| v.as_str())
        .expect("inventory artifact path");
    assert!(
        std::path::Path::new(path).exists(),
        "artifact file must exist"
    );
}

#[test]
fn configs_print_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["configs", "print", "--format", "json"])
        .output()
        .expect("configs print");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn configs_fmt_check_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["configs", "fmt", "--check", "--format", "json"])
        .output()
        .expect("configs fmt");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert_eq!(payload.get("mode").and_then(|v| v.as_str()), Some("check"));
}

#[test]
fn configs_doctor_fixture_json_matches_golden() {
    let fixture_root = repo_root().join("crates/bijux-dev-atlas/tests/fixtures/configs-mini");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "configs",
            "doctor",
            "--repo-root",
            fixture_root.to_str().expect("fixture root"),
            "--run-id",
            "configs_fixture",
            "--format",
            "json",
        ])
        .output()
        .expect("configs doctor fixture");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    payload["duration_ms"] = serde_json::json!(0);
    let actual = serde_json::to_string_pretty(&payload).expect("json");
    let golden_path =
        repo_root().join("crates/bijux-dev-atlas/tests/goldens/configs_doctor_fixture.json");
    let golden = fs::read_to_string(golden_path).expect("golden");
    assert_eq!(actual.trim(), golden.trim());
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
fn ops_list_tools_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "list-tools", "--format", "json"])
        .output()
        .expect("ops list-tools");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("subprocess is denied"));
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
fn ops_render_kind_default_does_not_write_without_explicit_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "render", "--target", "kind", "--format", "json"])
        .output()
        .expect("ops render kind default");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let row = payload["rows"]
        .as_array()
        .and_then(|rows| rows.first())
        .expect("row");
    assert_eq!(row["write_enabled"].as_bool(), Some(false));
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
