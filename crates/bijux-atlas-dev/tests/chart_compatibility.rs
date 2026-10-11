// SPDX-License-Identifier: Apache-2.0

use std::{fs, path::PathBuf, process::Command};

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn breaking_report(alias_retained: bool, removal_target: &str) -> (bool, serde_json::Value) {
    let artifacts = repository().join("artifacts/governance/compatibility-tests");
    fs::create_dir_all(&artifacts).unwrap();
    let fixture = tempfile::tempdir_in(artifacts).unwrap();
    let root = fixture.path();
    let inputs = [
        ("configs/schemas/contracts/reports/breaking-changes.schema.json", fs::read_to_string(repository().join("configs/schemas/contracts/reports/breaking-changes.schema.json")).unwrap()),
        ("ops/k8s/charts/bijux-atlas/Chart.yaml", "version: 0.2.0\n".to_string()),
        ("ops/k8s/charts/bijux-atlas/values.schema.json", if alias_retained {
            r#"{"properties":{"autoscaling":{"type":"object"},"hpa":{"type":"object"}}}"#
        } else { r#"{"properties":{"hpa":{"type":"object"}}}"# }.to_string()),
        ("artifacts/governance/compat-warnings.json", r#"{"rows":[]}"#.to_string()),
        ("configs/sources/repository/docs/redirects.json", "{}".to_string()),
        ("configs/sources/governance/governance/exceptions.yaml", fs::read_to_string(repository().join("configs/sources/governance/governance/exceptions.yaml")).unwrap()),
        ("ops/release/notes/breaking.json", r#"{"schema_version":1,"entries":[]}"#.to_string()),
        ("ops/release/notes/breaking.schema.json", r#"{"properties":{"schema_version":{"const":1}}}"#.to_string()),
        ("configs/sources/governance/governance/deprecations.yaml", format!(
            "schema_version: 1\ndeprecations:\n  - id: DEP-CHART-AUTOSCALING\n    surface: chart-value\n    old_name: autoscaling\n    new_name: hpa\n    introduced: 2000-01-01\n    removal_target: {removal_target}\n    redirect_required: false\n    comms_required: true\n"
        )),
    ];
    for (name, contents) in inputs {
        let path = root.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas-dev"))
        .args(["governance", "breaking", "validate", "--repo-root"])
        .arg(root)
        .args(["--format", "json"])
        .output()
        .unwrap();
    let report =
        fs::read(root.join("artifacts/governance/breaking-changes.json")).unwrap_or_else(|_| {
            panic!(
                "report missing: {}",
                String::from_utf8_lossy(&output.stderr)
            )
        });
    (
        output.status.success(),
        serde_json::from_slice(&report).unwrap(),
    )
}

#[test]
fn elapsed_removal_target_keeps_supported_alias_compatible() {
    let (success, report) = breaking_report(true, "2001-01-01");
    assert!(success, "{report}");
    assert_eq!(report["summary"]["chart_breaking"], 0);
}

#[test]
fn removed_alias_requires_breaking_notes_and_chart_version() {
    let (success, report) = breaking_report(false, "2001-01-01");
    assert!(!success);
    assert_eq!(report["summary"]["chart_breaking"], 1);
    assert_eq!(report["contracts"]["GOV-BREAK-002"], false);
}

#[test]
fn premature_alias_removal_is_breaking_before_its_target() {
    let (success, report) = breaking_report(false, "2999-01-01");
    assert!(!success);
    assert_eq!(report["summary"]["chart_breaking"], 1);
    assert_eq!(report["rows"][0]["removal_target_elapsed"], false);
}

#[test]
#[ignore = "requires the Helm executable for chart rendering qualification"]
fn legacy_autoscaling_renders_and_canonical_enabled_values_take_precedence() {
    let root = repository();
    let render = |canonical: bool| {
        let mut command = Command::new("helm");
        command.args(["template", "compatibility"])
            .arg(root.join("ops/k8s/charts/bijux-atlas"))
            .args(["--show-only", "templates/hpa.yaml", "--set", "autoscaling.enabled=true,autoscaling.minReplicas=3,autoscaling.maxReplicas=7,autoscaling.cpu.targetAverageUtilization=55"]);
        if canonical {
            command.args([
                "--set",
                "hpa.enabled=true,hpa.minReplicas=4,hpa.maxReplicas=9,hpa.cpuUtilization=65",
            ]);
        }
        let output = command.output().expect("Helm executable");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_yaml::from_slice::<serde_yaml::Value>(&output.stdout).unwrap()
    };
    let legacy = render(false);
    assert_eq!(legacy["spec"]["minReplicas"], 3);
    assert_eq!(legacy["spec"]["maxReplicas"], 7);
    assert_eq!(
        legacy["spec"]["metrics"][0]["resource"]["target"]["averageUtilization"],
        55
    );
    let canonical = render(true);
    assert_eq!(canonical["spec"]["minReplicas"], 4);
    assert_eq!(canonical["spec"]["maxReplicas"], 9);
    assert_eq!(
        canonical["spec"]["metrics"][0]["resource"]["target"]["averageUtilization"],
        65
    );
}
