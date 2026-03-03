// SPDX-License-Identifier: Apache-2.0

use super::*;

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
fn ops_install_plan_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "install", "--plan", "--format", "json"])
        .output()
        .expect("ops install plan");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let row = payload["rows"]
        .as_array()
        .and_then(|rows| rows.first())
        .expect("row");
    assert_eq!(row["plan_mode"].as_bool(), Some(true));
    assert!(row.get("install_plan").is_some());
    assert!(row.get("profile_intent").is_some());
    assert!(row["install_plan"]["resources"].as_array().is_some());
    assert!(row["install_plan"]["namespaces"].as_array().is_some());
    assert!(row["install_plan"]["namespace_isolated"]
        .as_bool()
        .is_some());
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
