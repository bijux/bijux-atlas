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

fn update_goldens_enabled() -> bool {
    std::env::var("UPDATE_GOLDENS").ok().as_deref() == Some("1")
}

fn run_render(profile: &str, run_id: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "render",
            "--target",
            "kind",
            "--profile",
            profile,
            "--run-id",
            run_id,
            "--allow-write",
            "--format",
            "json",
        ])
        .output()
        .expect("ops render");
    assert!(
        output.status.success(),
        "render failed for profile `{profile}`:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let path = repo_root()
        .join("artifacts/ops")
        .join(run_id)
        .join(format!("render/{profile}/kind/render.yaml"));
    fs::read_to_string(path).expect("read rendered manifest")
}

fn run_install_plan(profile: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "install-plan",
            "--profile",
            profile,
            "--format",
            "json",
        ])
        .output()
        .expect("ops install-plan");
    assert!(
        output.status.success(),
        "install-plan failed for profile `{profile}`:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("parse json");
    serde_json::to_string_pretty(&payload).expect("serialize")
}

fn assert_or_update_golden(path: &PathBuf, actual: &str) {
    if update_goldens_enabled() {
        fs::write(path, actual).expect("write golden");
    }
    let expected = fs::read_to_string(path).expect("read golden");
    assert_eq!(actual, expected);
}

#[test]
fn ops_render_ci_matches_golden() {
    let actual = run_render("ci", "ops_render_ci_snapshot");
    let golden = repo_root().join("crates/bijux-dev-atlas/tests/goldens/ops_render_ci.yaml");
    assert_or_update_golden(&golden, &actual);
}

#[test]
fn ops_render_kind_matches_golden() {
    let actual = run_render("kind", "ops_render_kind_snapshot");
    let golden = repo_root().join("crates/bijux-dev-atlas/tests/goldens/ops_render_kind.yaml");
    assert_or_update_golden(&golden, &actual);
}

#[test]
fn ops_render_perf_matches_golden() {
    let actual = run_render("perf", "ops_render_perf_snapshot");
    let golden = repo_root().join("crates/bijux-dev-atlas/tests/goldens/ops_render_perf.yaml");
    assert_or_update_golden(&golden, &actual);
}

#[test]
fn ops_install_plan_ci_matches_golden() {
    let actual = run_install_plan("ci");
    let golden = repo_root().join("crates/bijux-dev-atlas/tests/goldens/ops_install_plan_ci.json");
    assert_or_update_golden(&golden, &actual);
}

#[test]
fn ops_install_plan_kind_matches_golden() {
    let actual = run_install_plan("kind");
    let golden =
        repo_root().join("crates/bijux-dev-atlas/tests/goldens/ops_install_plan_kind.json");
    assert_or_update_golden(&golden, &actual);
}

#[test]
fn ops_install_plan_perf_matches_golden() {
    let actual = run_install_plan("perf");
    let golden =
        repo_root().join("crates/bijux-dev-atlas/tests/goldens/ops_install_plan_perf.json");
    assert_or_update_golden(&golden, &actual);
}
