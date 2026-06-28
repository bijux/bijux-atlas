// SPDX-License-Identifier: Apache-2.0

use crate::inventory::scenario_catalog::FailureScenarioSpec;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioEvidenceArtifacts {
    pub evidence_dir_rel: String,
    pub evidence_files: Vec<String>,
    pub before_after_files: Vec<String>,
    pub rollback_files: Vec<String>,
    pub failure_evidence_files: Vec<String>,
}

pub fn scenario_evidence_artifacts(scenario_id: &str, run_id: &str) -> ScenarioEvidenceArtifacts {
    let evidence_dir_rel = format!("artifacts/ops/scenarios/{scenario_id}/{run_id}");
    ScenarioEvidenceArtifacts {
        evidence_dir_rel: evidence_dir_rel.clone(),
        evidence_files: vec![
            format!("{evidence_dir_rel}/result.json"),
            format!("{evidence_dir_rel}/summary.md"),
        ],
        before_after_files: vec![
            format!("{evidence_dir_rel}/before-config.json"),
            format!("{evidence_dir_rel}/after-config.json"),
            format!("{evidence_dir_rel}/before-api-surface.json"),
            format!("{evidence_dir_rel}/after-api-surface.json"),
            format!("{evidence_dir_rel}/before-metrics.json"),
            format!("{evidence_dir_rel}/after-metrics.json"),
            format!("{evidence_dir_rel}/before-dataset-registry.json"),
            format!("{evidence_dir_rel}/after-dataset-registry.json"),
        ],
        rollback_files: vec![
            format!("{evidence_dir_rel}/rollback-restore-validation.json"),
            format!("{evidence_dir_rel}/rollback-query-correctness.json"),
        ],
        failure_evidence_files: vec![
            format!("{evidence_dir_rel}/failure-classification.json"),
            format!("{evidence_dir_rel}/metrics-snapshot.json"),
            format!("{evidence_dir_rel}/config-snapshot.json"),
            format!("{evidence_dir_rel}/logs-snapshot.txt"),
        ],
    }
}

pub fn write_deterministic_scenario_evidence(
    repo_root: &Path,
    scenario_id: &str,
    profile: Option<&str>,
    mode: &str,
    run_id: &str,
    artifacts: &ScenarioEvidenceArtifacts,
    upgrade_enabled: bool,
    failure_spec: Option<&FailureScenarioSpec>,
) -> Result<(), String> {
    let evidence_dir = repo_root.join(&artifacts.evidence_dir_rel);
    std::fs::create_dir_all(&evidence_dir).map_err(|err| {
        format!(
            "failed to create evidence directory {}: {err}",
            evidence_dir.display()
        )
    })?;

    let now = "1970-01-01T00:00:00Z";
    let result = serde_json::json!({
        "schema_version": 1,
        "schema_ref": "ops/e2e/scenarios/result-schema.json",
        "runner_version": "1.0",
        "scenario_id": scenario_id,
        "run_id": run_id,
        "mode": mode,
        "status": "pass",
        "started_at_utc": now,
        "completed_at_utc": now,
        "summary": if failure_spec.is_some() { "failure scenario completed in deterministic evidence mode" } else { "scenario completed in deterministic evidence mode" },
        "prerequisites": ["ops/e2e/scenarios/scenarios.json", "ops/e2e/scenarios/version-compatibility.json", "ops/e2e/scenarios/result-schema.json"],
        "metrics": {"duration_ms": 0, "checks_passed": 1, "checks_failed": 0},
        "evidence": {"directory": artifacts.evidence_dir_rel, "files": artifacts.evidence_files},
        "pointers": {"report_json": format!("{}/result.json", artifacts.evidence_dir_rel), "report_markdown": format!("{}/summary.md", artifacts.evidence_dir_rel)}
    });
    let result_path = evidence_dir.join("result.json");
    let summary_path = evidence_dir.join("summary.md");
    std::fs::write(
        &result_path,
        serde_json::to_string_pretty(&result).map_err(|err| {
            format!(
                "failed to encode scenario result {}: {err}",
                result_path.display()
            )
        })?,
    )
    .map_err(|err| {
        format!(
            "failed to write scenario result {}: {err}",
            result_path.display()
        )
    })?;
    std::fs::write(
        &summary_path,
        format!(
            "# Scenario Evidence\n\n- scenario: `{scenario_id}`\n- run_id: `{run_id}`\n- mode: `{mode}`\n- status: `pass`\n"
        ),
    )
    .map_err(|err| format!("failed to write scenario summary {}: {err}", summary_path.display()))?;

    if upgrade_enabled {
        for rel in &artifacts.before_after_files {
            let path = repo_root.join(rel);
            std::fs::write(
                &path,
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": 1,
                    "scenario_id": scenario_id,
                    "run_id": run_id,
                    "snapshot": rel,
                }))
                .map_err(|err| format!("failed to encode snapshot {}: {err}", path.display()))?,
            )
            .map_err(|err| format!("failed to write snapshot {}: {err}", path.display()))?;
        }
    }

    if let Some(spec) = failure_spec {
        let classification_path = repo_root.join(&artifacts.failure_evidence_files[0]);
        let metrics_path = repo_root.join(&artifacts.failure_evidence_files[1]);
        let config_path = repo_root.join(&artifacts.failure_evidence_files[2]);
        let logs_path = repo_root.join(&artifacts.failure_evidence_files[3]);
        std::fs::write(
            &classification_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "scenario_id": scenario_id,
                "run_id": run_id,
                "failure_mode": spec.failure_mode,
                "failure_expected": spec.failure_expected,
                "expected_behavior": spec.expected_behavior,
                "recommended_action": spec.recommended_action,
                "classification": if spec.failure_expected { "controlled-failure" } else { "degraded-success" }
            }))
            .map_err(|err| format!("failed to encode failure classification {}: {err}", classification_path.display()))?,
        )
        .map_err(|err| format!("failed to write failure classification {}: {err}", classification_path.display()))?;
        std::fs::write(
            &metrics_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "scenario_id": scenario_id,
                "run_id": run_id,
                "metrics": {
                    "error_rate": if spec.failure_expected { 1.0 } else { 0.05 },
                    "warning_count": if spec.failure_expected { 1 } else { 3 },
                    "latency_violation_count": if spec.failure_mode == "simulate-downstream-timeout" { 1 } else { 0 }
                }
            }))
            .map_err(|err| format!("failed to encode metrics snapshot {}: {err}", metrics_path.display()))?,
        )
        .map_err(|err| format!("failed to write metrics snapshot {}: {err}", metrics_path.display()))?;
        std::fs::write(
            &config_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "scenario_id": scenario_id,
                "run_id": run_id,
                "snapshot": "deterministic",
                "profile": profile
            }))
            .map_err(|err| {
                format!(
                    "failed to encode config snapshot {}: {err}",
                    config_path.display()
                )
            })?,
        )
        .map_err(|err| {
            format!(
                "failed to write config snapshot {}: {err}",
                config_path.display()
            )
        })?;
        std::fs::write(
            &logs_path,
            format!(
                "level=ERROR scenario={scenario_id} run_id={run_id} failure_mode={} recommended_action=\"{}\"\n",
                spec.failure_mode, spec.recommended_action
            ),
        )
        .map_err(|err| format!("failed to write logs snapshot {}: {err}", logs_path.display()))?;
    }

    if scenario_id.starts_with("rollback-") {
        for rel in &artifacts.rollback_files {
            let path = repo_root.join(rel);
            std::fs::write(
                &path,
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": 1,
                    "scenario_id": scenario_id,
                    "run_id": run_id,
                    "status": "restored",
                    "report": rel,
                }))
                .map_err(|err| {
                    format!("failed to encode rollback report {}: {err}", path.display())
                })?,
            )
            .map_err(|err| format!("failed to write rollback report {}: {err}", path.display()))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{scenario_evidence_artifacts, write_deterministic_scenario_evidence};
    use crate::inventory::scenario_catalog::FailureScenarioSpec;

    #[test]
    fn scenario_evidence_writer_materializes_owned_files() {
        let root = tempfile::tempdir().expect("tempdir");
        let artifacts = scenario_evidence_artifacts("rollback-minor", "owned-run");
        let failure_spec = FailureScenarioSpec {
            schema_version: 1,
            id: "rollback-minor".to_string(),
            failure_mode: "simulate-downstream-timeout".to_string(),
            failure_expected: true,
            expected_behavior: "report and rollback".to_string(),
            recommended_action: "collect evidence".to_string(),
        };

        write_deterministic_scenario_evidence(
            root.path(),
            "rollback-minor",
            Some("kind"),
            "evidence",
            "owned-run",
            &artifacts,
            true,
            Some(&failure_spec),
        )
        .expect("write evidence");

        assert!(root.path().join(&artifacts.evidence_files[0]).exists());
        assert!(root.path().join(&artifacts.before_after_files[0]).exists());
        assert!(root.path().join(&artifacts.rollback_files[0]).exists());
        assert!(root
            .path()
            .join(&artifacts.failure_evidence_files[0])
            .exists());
    }
}
