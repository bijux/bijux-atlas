// SPDX-License-Identifier: Apache-2.0

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

pub const OPS_RUNBOOK_SOURCE_REL: &str = "ops/README.md";

pub fn build_runbook_index_payload(
    repo_root: &Path,
    run_id: &str,
) -> Result<serde_json::Value, String> {
    let source_text = std::fs::read_to_string(repo_root.join(OPS_RUNBOOK_SOURCE_REL))
        .map_err(|err| format!("failed to read {OPS_RUNBOOK_SOURCE_REL}: {err}"))?;
    let rows = load_runbook_rows(repo_root)?;

    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": run_id,
        "generator": "ops generate runbook",
        "source": OPS_RUNBOOK_SOURCE_REL,
        "source_sha256": sha256_hex(&source_text),
        "status": "pass",
        "rows": rows,
        "summary": {"total": rows.len(), "errors": 0, "warnings": 0}
    }))
}

fn load_runbook_rows(repo_root: &Path) -> Result<Vec<serde_json::Value>, String> {
    let install_matrix_path = repo_root.join("ops/k8s/install-matrix.json");
    let install_matrix: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&install_matrix_path)
            .map_err(|err| format!("failed to read {}: {err}", install_matrix_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", install_matrix_path.display()))?;

    let mut profiles = BTreeMap::<String, serde_json::Value>::new();
    for profile in install_matrix
        .get("profiles")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
    {
        let Some(name) = profile.get("name").and_then(|value| value.as_str()) else {
            continue;
        };
        profiles.insert(name.to_string(), profile.clone());
    }

    let profile_intent_path = repo_root.join("ops/stack/profile-intent.json");
    let mut profile_intents = BTreeMap::<String, serde_json::Value>::new();
    if profile_intent_path.exists() {
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&profile_intent_path).map_err(
                |err| format!("failed to read {}: {err}", profile_intent_path.display()),
            )?)
            .map_err(|err| format!("failed to parse {}: {err}", profile_intent_path.display()))?;
        for profile in value
            .get("profiles")
            .and_then(|value| value.as_array())
            .into_iter()
            .flatten()
        {
            let Some(name) = profile.get("name").and_then(|value| value.as_str()) else {
                continue;
            };
            profile_intents.insert(name.to_string(), profile.clone());
        }
    }

    let toolchain_path = repo_root.join("ops/inventory/toolchain.json");
    let toolchain: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&toolchain_path)
            .map_err(|err| format!("failed to read {}: {err}", toolchain_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", toolchain_path.display()))?;
    let tool_versions = toolchain
        .get("tools")
        .and_then(|value| value.as_object())
        .map(|tools| {
            tools
                .iter()
                .map(|(binary, detail)| {
                    serde_json::json!({
                        "binary": binary,
                        "probe_argv": detail.get("probe_argv").cloned().unwrap_or_else(|| serde_json::json!([])),
                        "required": detail.get("required").and_then(|value| value.as_bool()).unwrap_or(false),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let failure_modes = vec![
        serde_json::json!({"code": "OPS_MANIFEST_ERROR", "meaning": "required ops manifest or generated input is missing or unreadable"}),
        serde_json::json!({"code": "OPS_SCHEMA_ERROR", "meaning": "authored inputs drifted outside their governed schema"}),
        serde_json::json!({"code": "OPS_TOOL_ERROR", "meaning": "required tool invocation failed or a required tool is unavailable"}),
        serde_json::json!({"code": "OPS_PROFILE_ERROR", "meaning": "selected profile is unknown or not declared in the governed registries"}),
        serde_json::json!({"code": "OPS_EFFECT_ERROR", "meaning": "effectful install action was requested without the required capability flags"}),
    ];

    let mut scenarios = install_matrix
        .get("scenarios")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    scenarios.sort_by(|left, right| {
        left.get("name")
            .and_then(|value| value.as_str())
            .cmp(&right.get("name").and_then(|value| value.as_str()))
    });

    let mut rows = Vec::new();
    for scenario in scenarios {
        let Some(name) = scenario.get("name").and_then(|value| value.as_str()) else {
            continue;
        };
        let Some(kind) = scenario.get("kind").and_then(|value| value.as_str()) else {
            continue;
        };
        let Some(profile) = scenario.get("profile").and_then(|value| value.as_str()) else {
            continue;
        };
        let suite = scenario
            .get("suite")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        let values_file = profiles
            .get(profile)
            .and_then(|value| value.get("values_file"))
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        let steps = match kind {
            "install" => vec![
                format!(
                    "bijux dev atlas ops render --profile {profile} --target helm --allow-subprocess --allow-write --format json"
                ),
                format!("bijux dev atlas ops install --profile {profile} --plan --format json"),
                format!(
                    "bijux dev atlas ops install --profile {profile} --kind --apply --allow-subprocess --allow-write --allow-network --format json"
                ),
            ],
            "upgrade" => vec![
                format!(
                    "bijux dev atlas ops render --profile {profile} --target helm --allow-subprocess --allow-write --format json"
                ),
                format!(
                    "bijux dev atlas ops install --profile {profile} --kind --apply --allow-subprocess --allow-write --allow-network --format json"
                ),
            ],
            "rollback" => vec![
                format!(
                    "bijux dev atlas ops install --profile {profile} --kind --apply --allow-subprocess --allow-write --allow-network --format json"
                ),
                format!(
                    "bijux dev atlas ops stack down --profile {profile} --allow-subprocess --allow-write --allow-network --force --format json"
                ),
            ],
            _ => Vec::new(),
        };
        let verification_commands = vec![
            format!("bijux dev atlas ops install --profile {profile} --plan --format json"),
            "kubectl get pods -n bijux-atlas".to_string(),
            "kubectl get svc -n bijux-atlas".to_string(),
            "curl -fsS http://127.0.0.1:8080/health".to_string(),
        ];
        let rollback_commands = vec![
            format!(
                "bijux dev atlas ops stack down --profile {profile} --allow-subprocess --allow-write --allow-network --force --format json"
            ),
            "kubectl delete namespace bijux-atlas --ignore-not-found".to_string(),
        ];
        rows.push(serde_json::json!({
            "scenario": name,
            "scenario_kind": kind,
            "profile": profile,
            "suite": suite,
            "values_file": values_file,
            "baseline_ref": scenario.get("baseline_ref").cloned(),
            "target_ref": scenario.get("target_ref").cloned(),
            "profile_intent": profile_intents.get(profile).cloned(),
            "steps": steps,
            "verification_commands": verification_commands,
            "rollback_commands": rollback_commands,
            "failure_modes": failure_modes,
            "tool_versions": tool_versions,
        }));
    }

    Ok(rows)
}

fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::{build_runbook_index_payload, OPS_RUNBOOK_SOURCE_REL};

    #[test]
    fn runbook_index_payload_reads_owned_ops_inputs() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/k8s")).expect("mkdir k8s");
        std::fs::create_dir_all(root.path().join("ops/inventory")).expect("mkdir inventory");
        std::fs::create_dir_all(root.path().join("ops/stack")).expect("mkdir stack");
        std::fs::write(root.path().join(OPS_RUNBOOK_SOURCE_REL), "# ops runbook\n")
            .expect("write runbook");
        std::fs::write(
            root.path().join("ops/k8s/install-matrix.json"),
            r#"{
              "profiles":[{"name":"foundation","values_file":"ops/k8s/foundation.values.yaml"}],
              "scenarios":[{"name":"foundation-install","kind":"install","profile":"foundation","suite":"smoke"}]
            }"#,
        )
        .expect("write install matrix");
        std::fs::write(
            root.path().join("ops/stack/profile-intent.json"),
            r#"{"profiles":[{"name":"foundation","intent":"bootstrap"}]}"#,
        )
        .expect("write profile intent");
        std::fs::write(
            root.path().join("ops/inventory/toolchain.json"),
            r#"{"tools":{"helm":{"probe_argv":["version","--short"],"required":true}}}"#,
        )
        .expect("write toolchain");

        let payload =
            build_runbook_index_payload(root.path(), "owned-run").expect("runbook payload");
        let rows = payload["rows"].as_array().expect("rows");

        assert_eq!(payload["source"], OPS_RUNBOOK_SOURCE_REL);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["profile"], "foundation");
        assert_eq!(rows[0]["scenario_kind"], "install");
    }
}
