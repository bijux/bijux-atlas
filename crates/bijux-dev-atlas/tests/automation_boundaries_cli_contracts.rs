// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn fixture_root() -> PathBuf {
    repo_root().join("crates/bijux-dev-atlas/tests/fixtures/automation-boundary-violations")
}

#[test]
fn checks_automation_boundaries_reports_fixture_violations() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "checks",
            "automation-boundaries",
            "--repo-root",
            fixture_root().to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .output()
        .expect("run checks automation-boundaries");

    assert!(
        !output.status.success(),
        "fixture must fail boundaries; stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let raw = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(raw).expect("parse json");
    let checks = payload["checks"].as_array().expect("checks array");
    let check_ids = checks
        .iter()
        .filter_map(|row| row["id"].as_str())
        .collect::<Vec<_>>();
    for required in [
        "automation.directories.forbidden",
        "automation.root-language-files.forbidden",
        "automation.workflows.no-python",
        "automation.workflows.no-repo-bash-scripts",
        "automation.exec-bit.allowlist",
        "automation.shebang.allowlist",
        "automation.python-tooling.docs-only",
        "automation.tutorials.forbidden-patterns",
        "automation.clients.forbidden-patterns",
        "automation.ops.directory-purity",
    ] {
        assert!(
            check_ids.contains(&required),
            "required automation boundary check is missing: {required}"
        );
    }
    let violations = payload["violations"]
        .as_array()
        .expect("violations array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>();
    assert!(
        violations.iter().any(|v| v.contains("tools/")),
        "expected tools violation, got: {violations:?}"
    );
    assert!(
        violations.iter().any(|v| v.contains("requirements.txt")),
        "expected python tooling violation, got: {violations:?}"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.contains(".github/workflows/legacy.yml")),
        "expected workflow violation, got: {violations:?}"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.contains("ops/cli/perf/legacy_perf_helper.py")),
        "expected ops purity violation, got: {violations:?}"
    );
}

#[test]
fn contract_automation_boundaries_reports_fixture_violations() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contract",
            "automation-boundaries",
            "--repo-root",
            fixture_root().to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .output()
        .expect("run contract automation-boundaries");

    assert!(
        !output.status.success(),
        "fixture must fail boundaries; stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let raw = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(raw).expect("parse json");
    assert_eq!(payload["status"].as_str(), Some("fail"));
}
