// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

const SCENARIOS_REL: &str = "ops/e2e/scenarios/scenarios.json";
const FAILURE_CATALOG_REL: &str = "ops/e2e/scenarios/failure/injection-catalog.json";

pub fn build_resilience_report_payload(
    repo_root: &Path,
    run_id: &str,
) -> Result<serde_json::Value, String> {
    let scenarios_path = repo_root.join(SCENARIOS_REL);
    let failure_catalog_path = repo_root.join(FAILURE_CATALOG_REL);
    let scenarios: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&scenarios_path)
            .map_err(|err| format!("failed to read {}: {err}", scenarios_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", scenarios_path.display()))?;
    let failure_catalog: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&failure_catalog_path)
            .map_err(|err| format!("failed to read {}: {err}", failure_catalog_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", failure_catalog_path.display()))?;

    let resilience_scenarios = scenarios
        .get("scenarios")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter(|value| {
            value.get("intent").and_then(|intent| intent.as_str()) == Some("resilience")
        })
        .cloned()
        .collect::<Vec<_>>();

    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "ops_resilience_report",
        "run_id": run_id,
        "catalog_path": FAILURE_CATALOG_REL,
        "resilience_scenarios": resilience_scenarios,
        "failure_mechanism_count": failure_catalog.get("mechanisms").and_then(|value| value.as_array()).map(|value| value.len()).unwrap_or(0),
        "summary": {
            "total_resilience_scenarios": resilience_scenarios.len(),
            "errors": 0,
            "warnings": 0
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::build_resilience_report_payload;

    #[test]
    fn resilience_report_reads_owned_scenario_catalogs() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/e2e/scenarios/failure"))
            .expect("mkdir failure");
        std::fs::write(
            root.path().join("ops/e2e/scenarios/scenarios.json"),
            r#"{"scenarios":[{"id":"resilience-a","intent":"resilience"},{"id":"upgrade-a","intent":"upgrade"}]}"#,
        )
        .expect("write scenarios");
        std::fs::write(
            root.path()
                .join("ops/e2e/scenarios/failure/injection-catalog.json"),
            r#"{"mechanisms":[{"id":"drop-packets"},{"id":"slow-dns"}]}"#,
        )
        .expect("write failure catalog");

        let payload =
            build_resilience_report_payload(root.path(), "owned-run").expect("resilience report");

        assert_eq!(payload["kind"], "ops_resilience_report");
        assert_eq!(payload["summary"]["total_resilience_scenarios"], 1);
        assert_eq!(payload["failure_mechanism_count"], 2);
    }
}
