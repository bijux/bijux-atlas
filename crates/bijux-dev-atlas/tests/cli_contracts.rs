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
        .args(["explain", "checks_ops_surface_manifest", "--format", "json"])
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
    assert_eq!(
        payload
            .get("registry_errors")
            .and_then(|v| v.as_array())
            .map(|v| v.len()),
        Some(0)
    );
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
    assert_eq!(payload.get("schema_version").and_then(|v| v.as_u64()), Some(1));
    assert!(payload.get("errors").and_then(|v| v.as_array()).is_some());
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
    let pages = payload.get("pages").and_then(|v| v.as_array()).expect("pages");
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
    let bytes = if output.stdout.is_empty() { &output.stderr } else { &output.stdout };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert_eq!(
        payload.get("options").and_then(|v| v.get("strict")).and_then(|v| v.as_bool()),
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
    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let mut payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    payload["duration_ms"] = serde_json::json!(0);
    let actual = serde_json::to_string_pretty(&payload).expect("json");
    let golden_path = repo_root().join("crates/bijux-dev-atlas/tests/goldens/docs_doctor_fixture.json");
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
    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let mut payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    payload["duration_ms"] = serde_json::json!(0);
    let actual = serde_json::to_string_pretty(&payload).expect("json");
    let golden_path = repo_root().join("crates/bijux-dev-atlas/tests/goldens/configs_doctor_fixture.json");
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
        assert!(output.status.success());
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
fn check_list_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["check", "list", "--format", "json"])
        .output()
        .expect("check list json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("checks").and_then(|v| v.as_array()).is_some());
}

#[test]
fn check_doctor_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["check", "doctor", "--format", "json"])
        .output()
        .expect("check doctor json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(payload.get("status").and_then(|v| v.as_str()), Some("ok"));
}

#[test]
fn check_run_supports_out_file() {
    let out = repo_root().join("artifacts/tests/check_run_output.json");
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
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
    assert!(output.status.success());
    let written = fs::read_to_string(out).expect("read out file");
    let payload: serde_json::Value = serde_json::from_str(&written).expect("json file");
    assert!(payload.get("run_id").is_some());
    assert!(payload.get("capabilities").is_some());
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
