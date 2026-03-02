// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

fn collect_rendered_env_keys(rendered_yaml: &str) -> std::collections::BTreeSet<String> {
    fn collect_from_value(
        value: &serde_yaml::Value,
        env_keys: &mut std::collections::BTreeSet<String>,
    ) {
        match value {
            serde_yaml::Value::Mapping(map) => {
                for (key, child) in map {
                    if let Some(key_text) = key.as_str() {
                        if (key_text.starts_with("ATLAS_") || key_text.starts_with("BIJUX_"))
                            && key_text.len() > "ATLAS_".len()
                        {
                            env_keys.insert(key_text.to_string());
                        }
                        if key_text == "name" {
                            if let Some(env_name) = child.as_str() {
                                if (env_name.starts_with("ATLAS_")
                                    || env_name.starts_with("BIJUX_"))
                                    && env_name.len() > "ATLAS_".len()
                                {
                                    env_keys.insert(env_name.to_string());
                                }
                            }
                        }
                    }
                    collect_from_value(child, env_keys);
                }
            }
            serde_yaml::Value::Sequence(items) => {
                for child in items {
                    collect_from_value(child, env_keys);
                }
            }
            _ => {}
        }
    }

    let mut env_keys = std::collections::BTreeSet::<String>::new();
    for document in serde_yaml::Deserializer::from_str(rendered_yaml) {
        let value = match serde_yaml::Value::deserialize(document) {
            Ok(value) => value,
            Err(_) => continue,
        };
        collect_from_value(&value, &mut env_keys);
    }
    env_keys
}

fn violation(
    contract_id: &str,
    test_id: &str,
    file: Option<String>,
    message: impl Into<String>,
) -> Violation {
    Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file,
        line: None,
        message: message.into(),
        evidence: None,
    }
}

fn test_repo_001_law_registry_exists_and_is_valid(ctx: &RunContext) -> TestResult {
    let rel = "docs/_internal/contracts/repo-laws.json";
    let path = ctx.repo_root.join(rel);
    let text = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![violation(
                "REPO-001",
                "repo.laws.registry_present",
                Some(rel.to_string()),
                format!("read failed: {err}"),
            )])
        }
    };
    let json: serde_json::Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![violation(
                "REPO-001",
                "repo.laws.registry_present",
                Some(rel.to_string()),
                format!("invalid json: {err}"),
            )])
        }
    };
    let mut violations = Vec::new();
    if json.get("schema_version").and_then(|v| v.as_u64()) != Some(1) {
        violations.push(violation(
            "REPO-001",
            "repo.laws.registry_present",
            Some(rel.to_string()),
            "repo laws registry must declare schema_version=1",
        ));
    }
    if json.get("laws").and_then(|v| v.as_array()).is_none() {
        violations.push(violation(
            "REPO-001",
            "repo.laws.registry_present",
            Some(rel.to_string()),
            "repo laws registry must contain a laws array",
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_repo_002_root_allowlist_config_present(ctx: &RunContext) -> TestResult {
    let rel = "configs/repo/root-file-allowlist.json";
    if ctx.repo_root.join(rel).exists() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "REPO-002",
            "repo.surface.root_allowlist_present",
            Some(rel.to_string()),
            "root allowlist config is missing",
        )])
    }
}

fn test_repo_003_helm_env_surface_subset_of_runtime_contract(ctx: &RunContext) -> TestResult {
    if !ctx.allow_subprocess {
        return TestResult::Skip("helm env surface check requires --allow-subprocess".to_string());
    }

    let output = match Command::new("helm")
        .current_dir(&ctx.repo_root)
        .args([
            "template",
            "atlas-default",
            "ops/k8s/charts/bijux-atlas",
            "-f",
            "ops/k8s/charts/bijux-atlas/values.yaml",
        ])
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            return TestResult::Fail(vec![violation(
                "REPO-003",
                "repo.helm_env_surface.subset_of_runtime_allowlist",
                Some("ops/k8s/charts/bijux-atlas".to_string()),
                format!("helm template failed to start: {err}"),
            )]);
        }
    };
    if !output.status.success() {
        return TestResult::Fail(vec![violation(
            "REPO-003",
            "repo.helm_env_surface.subset_of_runtime_allowlist",
            Some("ops/k8s/charts/bijux-atlas".to_string()),
            format!(
                "helm template failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        )]);
    }

    let rendered = String::from_utf8_lossy(&output.stdout);
    let emitted = collect_rendered_env_keys(&rendered);
    let schema_path = ctx.repo_root.join("configs/contracts/env.schema.json");
    let schema_text = match fs::read_to_string(&schema_path) {
        Ok(text) => text,
        Err(err) => {
            return TestResult::Fail(vec![violation(
                "REPO-003",
                "repo.helm_env_surface.subset_of_runtime_allowlist",
                Some("configs/contracts/env.schema.json".to_string()),
                format!("read failed: {err}"),
            )]);
        }
    };
    let schema_json: serde_json::Value = match serde_json::from_str(&schema_text) {
        Ok(json) => json,
        Err(err) => {
            return TestResult::Fail(vec![violation(
                "REPO-003",
                "repo.helm_env_surface.subset_of_runtime_allowlist",
                Some("configs/contracts/env.schema.json".to_string()),
                format!("invalid json: {err}"),
            )]);
        }
    };
    let allowed = schema_json["allowed_env"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str())
        .map(str::to_string)
        .collect::<std::collections::BTreeSet<_>>();

    let mut violations = emitted
        .difference(&allowed)
        .map(|env_key| {
            violation(
                "REPO-003",
                "repo.helm_env_surface.subset_of_runtime_allowlist",
                Some(env_key.clone()),
                "helm-emitted env key is missing from configs/contracts/env.schema.json",
            )
        })
        .collect::<Vec<_>>();
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![
        Contract {
            id: ContractId("REPO-001".to_string()),
            title: "repo laws registry remains valid and parseable",
            tests: vec![TestCase {
                id: TestId("repo.laws.registry_present".to_string()),
                title: "repo laws registry exists and parses",
                kind: TestKind::Pure,
                run: test_repo_001_law_registry_exists_and_is_valid,
            }],
        },
        Contract {
            id: ContractId("REPO-002".to_string()),
            title: "repo root allowlist config remains present",
            tests: vec![TestCase {
                id: TestId("repo.surface.root_allowlist_present".to_string()),
                title: "root allowlist config exists",
                kind: TestKind::Pure,
                run: test_repo_002_root_allowlist_config_present,
            }],
        },
        Contract {
            id: ContractId("REPO-003".to_string()),
            title: "helm-emitted env keys stay inside the runtime allowlist",
            tests: vec![TestCase {
                id: TestId("repo.helm_env_surface.subset_of_runtime_allowlist".to_string()),
                title: "helm-emitted env keys are a subset of configs/contracts/env.schema.json",
                kind: TestKind::Subprocess,
                run: test_repo_003_helm_env_surface_subset_of_runtime_contract,
            }],
        },
    ])
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "REPO-001" => {
            "Ensures canonical repo law registry exists and is valid JSON with required metadata."
                .to_string()
        }
        "REPO-002" => "Ensures root allowlist authority config exists for root surface governance."
            .to_string(),
        "REPO-003" => "Ensures the rendered Helm env surface cannot drift outside the runtime env allowlist contract."
            .to_string(),
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts repo`.".to_string(),
    }
}

pub fn contract_gate_command(contract_id: &str) -> &'static str {
    match contract_id {
        "REPO-003" => "bijux dev atlas contracts repo --mode effect --allow-subprocess",
        _ => "bijux dev atlas contracts repo --mode static",
    }
}
