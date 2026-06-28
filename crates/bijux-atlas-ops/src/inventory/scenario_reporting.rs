// SPDX-License-Identifier: Apache-2.0

use crate::inventory::scenario_catalog::{FailureScenarioSpec, ScenarioSpec, UpgradeScenarioSpec};
use crate::lifecycle::simulation::ScenarioEvidenceArtifacts;

pub fn build_scenario_list_payload(scenarios: Vec<ScenarioSpec>) -> serde_json::Value {
    let mut rows = scenarios
        .into_iter()
        .map(|scenario| {
            serde_json::json!({
                "id": scenario.id,
                "description": scenario.description,
                "action_id": scenario.action_id,
                "entrypoint": scenario.entrypoint,
                "tags": [
                    scenario.evidence_class.unwrap_or_else(|| "slow".to_string()),
                    if scenario.compose.get("load").copied().unwrap_or(false) { "effect" } else { "offline" },
                ],
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left.get("id")
            .and_then(|value| value.as_str())
            .cmp(&right.get("id").and_then(|value| value.as_str()))
    });

    serde_json::json!({
        "schema_version": 1,
        "text": "ops scenario list",
        "rows": rows,
        "summary": {"total": rows.len(), "errors": 0, "warnings": 0}
    })
}

pub fn build_scenario_run_payload(
    requested_id: &str,
    scenario: &ScenarioSpec,
    mode: &str,
    run_id: &str,
    upgrade_spec: Option<&UpgradeScenarioSpec>,
    failure_spec: Option<&FailureScenarioSpec>,
    evidence: &ScenarioEvidenceArtifacts,
) -> serde_json::Value {
    let versioned_install = upgrade_spec.map(|spec| {
        serde_json::json!({
            "from_version": spec.from_version,
            "to_version": spec.to_version,
            "kind": spec.kind,
            "failure_expected": spec.failure_expected,
        })
    });

    serde_json::json!({
        "schema_version": 1,
        "text": format!("ops scenario run {requested_id}"),
        "rows": [{
            "scenario_id": requested_id,
            "action_id": scenario.action_id,
            "entrypoint": scenario.entrypoint,
            "mode": mode,
            "run_id": run_id,
            "compose": scenario.compose,
            "versioned_install": versioned_install,
            "failure_mode": failure_spec.map(|spec| spec.failure_mode.clone()),
            "failure_expected": failure_spec.map(|spec| spec.failure_expected),
            "recommended_action": failure_spec.map(|spec| spec.recommended_action.clone()),
            "upgrade_step": upgrade_spec.map(|spec| spec.steps.contains(&"upgrade".to_string())).unwrap_or(false),
            "rollback_step": upgrade_spec.map(|spec| spec.steps.contains(&"rollback".to_string())).unwrap_or(false),
            "scenario_steps": upgrade_spec.map(|spec| spec.steps.clone()).unwrap_or_default(),
            "evidence_directory": evidence.evidence_dir_rel,
            "required_evidence_files": evidence.evidence_files,
            "before_after_evidence_files": if upgrade_spec.is_some() { evidence.before_after_files.clone() } else { Vec::<String>::new() },
            "rollback_evidence_files": if scenario.id.starts_with("rollback-") { evidence.rollback_files.clone() } else { Vec::<String>::new() },
            "failure_evidence_files": if failure_spec.is_some() { evidence.failure_evidence_files.clone() } else { Vec::<String>::new() },
        }],
        "summary": {"total": 1, "errors": 0, "warnings": 0}
    })
}

#[cfg(test)]
mod tests {
    use super::{build_scenario_list_payload, build_scenario_run_payload};
    use crate::inventory::scenario_catalog::{
        FailureScenarioSpec, ScenarioSpec, UpgradeScenarioSpec,
    };
    use crate::lifecycle::simulation::scenario_evidence_artifacts;
    use std::collections::BTreeMap;

    #[test]
    fn scenario_list_payload_sorts_rows_by_id() {
        let payload = build_scenario_list_payload(vec![
            ScenarioSpec {
                id: "zeta".to_string(),
                description: "later".to_string(),
                action_id: "run-zeta".to_string(),
                entrypoint: None,
                compose: BTreeMap::new(),
                evidence_class: None,
            },
            ScenarioSpec {
                id: "alpha".to_string(),
                description: "first".to_string(),
                action_id: "run-alpha".to_string(),
                entrypoint: None,
                compose: BTreeMap::new(),
                evidence_class: Some("fast".to_string()),
            },
        ]);

        let rows = payload["rows"].as_array().expect("rows");
        assert_eq!(rows[0]["id"], "alpha");
        assert_eq!(rows[1]["id"], "zeta");
    }

    #[test]
    fn scenario_run_payload_reports_owned_evidence_contracts() {
        let scenario = ScenarioSpec {
            id: "rollback-minor".to_string(),
            description: "rollback".to_string(),
            action_id: "run-rollback".to_string(),
            entrypoint: Some("ops/scenario.sh".to_string()),
            compose: BTreeMap::new(),
            evidence_class: Some("slow".to_string()),
        };
        let upgrade = UpgradeScenarioSpec {
            schema_version: 1,
            id: "rollback-minor".to_string(),
            from_version: "1.0.0".to_string(),
            to_version: "1.1.0".to_string(),
            kind: "upgrade".to_string(),
            failure_expected: false,
            steps: vec!["upgrade".to_string(), "rollback".to_string()],
        };
        let failure = FailureScenarioSpec {
            schema_version: 1,
            id: "rollback-minor".to_string(),
            failure_mode: "simulate-downstream-timeout".to_string(),
            failure_expected: true,
            expected_behavior: "rollback".to_string(),
            recommended_action: "collect evidence".to_string(),
        };
        let artifacts = scenario_evidence_artifacts("rollback-minor", "owned-run");

        let payload = build_scenario_run_payload(
            "rollback-minor",
            &scenario,
            "evidence",
            "owned-run",
            Some(&upgrade),
            Some(&failure),
            &artifacts,
        );

        let row = &payload["rows"][0];
        assert_eq!(row["scenario_id"], "rollback-minor");
        assert!(row["rollback_step"].as_bool().expect("rollback step"));
        assert!(!row["failure_evidence_files"]
            .as_array()
            .expect("failure files")
            .is_empty());
    }
}
