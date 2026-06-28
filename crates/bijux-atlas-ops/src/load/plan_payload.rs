// SPDX-License-Identifier: Apache-2.0

use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn sorted_env_rows(env: &BTreeMap<String, String>) -> Vec<Value> {
    let mut rows = env
        .iter()
        .map(|(name, value)| json!({"name": name, "value": value}))
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    rows
}

pub fn load_plan_payload(
    suite: &str,
    script: &str,
    dataset: &str,
    thresholds: &str,
    env: &BTreeMap<String, String>,
    errors: Vec<String>,
) -> Value {
    json!({
        "schema_version": 1,
        "text": format!("ops load plan suite={suite}"),
        "rows": [{
            "suite": suite,
            "script": script,
            "dataset": dataset,
            "thresholds": thresholds,
            "env": sorted_env_rows(env)
        }],
        "errors": errors,
        "summary": {"total": 1, "errors": errors.len(), "warnings": 0}
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_env_rows_are_stable() {
        let env = BTreeMap::from([
            ("ZETA".to_string(), "z".to_string()),
            ("ALPHA".to_string(), "a".to_string()),
        ]);

        let rows = sorted_env_rows(&env);

        assert_eq!(rows[0]["name"], "ALPHA");
        assert_eq!(rows[1]["name"], "ZETA");
    }

    #[test]
    fn load_plan_payload_tracks_manifest_errors() {
        let payload = load_plan_payload(
            "mixed",
            "ops/load/k6/suites/mixed.js",
            "ops/load/queries/pinned.json",
            "ops/load/thresholds/mixed.json",
            &BTreeMap::new(),
            vec!["missing file".to_string()],
        );

        assert_eq!(payload["text"], "ops load plan suite=mixed");
        assert_eq!(payload["summary"]["errors"], 1);
        assert_eq!(payload["errors"][0], "missing file");
    }
}
