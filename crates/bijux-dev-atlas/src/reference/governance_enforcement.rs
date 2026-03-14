// SPDX-License-Identifier: Apache-2.0
//! Governance enforcement rule registry, loader, and evaluator.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const GOVERNANCE_RULES_PATH: &str = "configs/governance/enforcement/rules.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceRuleType {
    RequiredFilesExist,
    ProhibitedFilesAbsent,
    RepoLayoutContract,
    DocsFrontMatterComplete,
    ContractRegistryComplete,
    ChecksRegistryComplete,
    ScenarioRegistryComplete,
    OpsArtifactRegistryComplete,
    ReleaseArtifactRegistryComplete,
    DocsNavigationConsistent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceSeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceRuleClassification {
    Repository,
    Documentation,
    Registry,
    Deployment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRule {
    pub id: String,
    pub title: String,
    pub severity: GovernanceSeverity,
    pub classification: GovernanceRuleClassification,
    pub rule_type: GovernanceRuleType,
    #[serde(default)]
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRuleRegistry {
    pub schema_version: u64,
    pub registry_id: String,
    pub rules: Vec<GovernanceRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceViolation {
    pub rule_id: String,
    pub severity: GovernanceSeverity,
    pub classification: GovernanceRuleClassification,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceEvaluation {
    pub schema_version: u64,
    pub kind: String,
    pub status: String,
    pub rule_count: usize,
    pub violations: Vec<GovernanceViolation>,
}

#[derive(Debug, Deserialize)]
struct RepoLayoutContract {
    schema_version: u64,
    required_directories: Vec<String>,
}

fn evaluate_registry_file(
    repo_root: &Path,
    rel: &str,
    array_key: &str,
    rule: &GovernanceRule,
    violations: &mut Vec<GovernanceViolation>,
    missing_message: &str,
) {
    let path = repo_root.join(rel);
    match fs::read_to_string(&path) {
        Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(value) => {
                let has_entries = value
                    .get(array_key)
                    .and_then(serde_json::Value::as_array)
                    .map(|rows| !rows.is_empty())
                    .unwrap_or(false);
                if !has_entries {
                    violations.push(GovernanceViolation {
                        rule_id: rule.id.clone(),
                        severity: rule.severity.clone(),
                        classification: rule.classification.clone(),
                        message: missing_message.to_string(),
                        path: Some(rel.to_string()),
                    });
                }
            }
            Err(err) => violations.push(GovernanceViolation {
                rule_id: rule.id.clone(),
                severity: rule.severity.clone(),
                classification: rule.classification.clone(),
                message: format!("registry parse failed: {err}"),
                path: Some(rel.to_string()),
            }),
        },
        Err(err) => violations.push(GovernanceViolation {
            rule_id: rule.id.clone(),
            severity: rule.severity.clone(),
            classification: rule.classification.clone(),
            message: format!("registry read failed: {err}"),
            path: Some(rel.to_string()),
        }),
    }
}

pub fn load_registry(repo_root: &Path) -> Result<GovernanceRuleRegistry, String> {
    let path = repo_root.join(GOVERNANCE_RULES_PATH);
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let registry: GovernanceRuleRegistry = serde_json::from_str(&text)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    if registry.schema_version != 1 {
        return Err(format!("{} must declare schema_version=1", path.display()));
    }
    Ok(registry)
}

pub fn evaluate_registry(
    repo_root: &Path,
    registry: &GovernanceRuleRegistry,
) -> GovernanceEvaluation {
    let mut violations = Vec::new();
    for rule in &registry.rules {
        match rule.rule_type {
            GovernanceRuleType::RequiredFilesExist => {
                for rel in &rule.paths {
                    let path = repo_root.join(rel);
                    if !path.exists() {
                        violations.push(GovernanceViolation {
                            rule_id: rule.id.clone(),
                            severity: rule.severity.clone(),
                            classification: rule.classification.clone(),
                            message: "required file is missing".to_string(),
                            path: Some(rel.clone()),
                        });
                    }
                }
            }
            GovernanceRuleType::ProhibitedFilesAbsent => {
                for rel in &rule.paths {
                    let path = repo_root.join(rel);
                    if path.exists() {
                        violations.push(GovernanceViolation {
                            rule_id: rule.id.clone(),
                            severity: rule.severity.clone(),
                            classification: rule.classification.clone(),
                            message: "prohibited file exists".to_string(),
                            path: Some(rel.clone()),
                        });
                    }
                }
            }
            GovernanceRuleType::RepoLayoutContract => {
                if let Some(contract_path) = rule.paths.first() {
                    let path = repo_root.join(contract_path);
                    match fs::read_to_string(&path) {
                        Ok(text) => match serde_json::from_str::<RepoLayoutContract>(&text) {
                            Ok(contract) => {
                                if contract.schema_version != 1 {
                                    violations.push(GovernanceViolation {
                                        rule_id: rule.id.clone(),
                                        severity: rule.severity.clone(),
                                        classification: rule.classification.clone(),
                                        message: "repo layout contract schema_version must be 1"
                                            .to_string(),
                                        path: Some(contract_path.clone()),
                                    });
                                }
                                for rel in contract.required_directories {
                                    if !repo_root.join(&rel).is_dir() {
                                        violations.push(GovernanceViolation {
                                            rule_id: rule.id.clone(),
                                            severity: rule.severity.clone(),
                                            classification: rule.classification.clone(),
                                            message: "required directory missing from repo layout"
                                                .to_string(),
                                            path: Some(rel),
                                        });
                                    }
                                }
                            }
                            Err(err) => violations.push(GovernanceViolation {
                                rule_id: rule.id.clone(),
                                severity: rule.severity.clone(),
                                classification: rule.classification.clone(),
                                message: format!("repo layout contract parse failed: {err}"),
                                path: Some(contract_path.clone()),
                            }),
                        },
                        Err(err) => violations.push(GovernanceViolation {
                            rule_id: rule.id.clone(),
                            severity: rule.severity.clone(),
                            classification: rule.classification.clone(),
                            message: format!("repo layout contract read failed: {err}"),
                            path: Some(contract_path.clone()),
                        }),
                    }
                }
            }
            GovernanceRuleType::DocsFrontMatterComplete => {
                for rel in &rule.paths {
                    let docs_root = repo_root.join(rel);
                    if !docs_root.exists() {
                        continue;
                    }
                    let mut stack = vec![docs_root];
                    while let Some(dir) = stack.pop() {
                        let Ok(read_dir) = fs::read_dir(&dir) else {
                            continue;
                        };
                        for entry in read_dir.flatten() {
                            let path = entry.path();
                            if path.is_dir() {
                                stack.push(path);
                                continue;
                            }
                            if path.extension().and_then(|v| v.to_str()) != Some("md") {
                                continue;
                            }
                            let Ok(text) = fs::read_to_string(&path) else {
                                continue;
                            };
                            if !(text.starts_with("---\n") && text[4..].contains("\n---\n")) {
                                let rel_path = path
                                    .strip_prefix(repo_root)
                                    .unwrap_or(&path)
                                    .display()
                                    .to_string();
                                violations.push(GovernanceViolation {
                                    rule_id: rule.id.clone(),
                                    severity: rule.severity.clone(),
                                    classification: rule.classification.clone(),
                                    message: "markdown file missing front matter block".to_string(),
                                    path: Some(rel_path),
                                });
                            }
                        }
                    }
                }
            }
            GovernanceRuleType::ContractRegistryComplete => {
                if let Some(rel) = rule.paths.first() {
                    evaluate_registry_file(
                        repo_root,
                        rel,
                        "contracts",
                        rule,
                        &mut violations,
                        "contract registry has no contracts entries",
                    );
                }
            }
            GovernanceRuleType::ChecksRegistryComplete => {
                if let Some(rel) = rule.paths.first() {
                    evaluate_registry_file(
                        repo_root,
                        rel,
                        "checks",
                        rule,
                        &mut violations,
                        "checks registry has no checks entries",
                    );
                }
            }
            GovernanceRuleType::ScenarioRegistryComplete => {
                if let Some(rel) = rule.paths.first() {
                    evaluate_registry_file(
                        repo_root,
                        rel,
                        "scenarios",
                        rule,
                        &mut violations,
                        "scenario registry has no scenarios entries",
                    );
                }
            }
            GovernanceRuleType::OpsArtifactRegistryComplete => {
                if let Some(rel) = rule.paths.first() {
                    evaluate_registry_file(
                        repo_root,
                        rel,
                        "render_outputs",
                        rule,
                        &mut violations,
                        "ops artifact registry has no render outputs",
                    );
                }
            }
            GovernanceRuleType::ReleaseArtifactRegistryComplete => {
                if let Some(rel) = rule.paths.first() {
                    let path = repo_root.join(rel);
                    match fs::read_to_string(&path) {
                        Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
                            Ok(value) => {
                                let has_schema = value.get("schema_version").is_some();
                                let has_release_assets = value.get("image_artifacts").is_some()
                                    || value.get("reports").is_some()
                                    || value.get("sboms").is_some();
                                if !(has_schema && has_release_assets) {
                                    violations.push(GovernanceViolation {
                                        rule_id: rule.id.clone(),
                                        severity: rule.severity.clone(),
                                        classification: rule.classification.clone(),
                                        message: "release artifact registry is missing required release asset sections".to_string(),
                                        path: Some(rel.clone()),
                                    });
                                }
                            }
                            Err(err) => violations.push(GovernanceViolation {
                                rule_id: rule.id.clone(),
                                severity: rule.severity.clone(),
                                classification: rule.classification.clone(),
                                message: format!("release artifact registry parse failed: {err}"),
                                path: Some(rel.clone()),
                            }),
                        },
                        Err(err) => violations.push(GovernanceViolation {
                            rule_id: rule.id.clone(),
                            severity: rule.severity.clone(),
                            classification: rule.classification.clone(),
                            message: format!("release artifact registry read failed: {err}"),
                            path: Some(rel.clone()),
                        }),
                    }
                }
            }
            GovernanceRuleType::DocsNavigationConsistent => {
                if let Some(rel) = rule.paths.first() {
                    let path = repo_root.join(rel);
                    match fs::read_to_string(&path) {
                        Ok(text) => match serde_yaml::from_str::<serde_yaml::Value>(&text) {
                            Ok(value) => {
                                let nav_len = value
                                    .get("nav")
                                    .and_then(serde_yaml::Value::as_sequence)
                                    .map_or(0, |rows| rows.len());
                                if nav_len == 0 {
                                    violations.push(GovernanceViolation {
                                        rule_id: rule.id.clone(),
                                        severity: rule.severity.clone(),
                                        classification: rule.classification.clone(),
                                        message: "docs navigation is missing or empty".to_string(),
                                        path: Some(rel.clone()),
                                    });
                                }
                            }
                            Err(err) => violations.push(GovernanceViolation {
                                rule_id: rule.id.clone(),
                                severity: rule.severity.clone(),
                                classification: rule.classification.clone(),
                                message: format!("docs navigation parse failed: {err}"),
                                path: Some(rel.clone()),
                            }),
                        },
                        Err(err) => violations.push(GovernanceViolation {
                            rule_id: rule.id.clone(),
                            severity: rule.severity.clone(),
                            classification: rule.classification.clone(),
                            message: format!("docs navigation read failed: {err}"),
                            path: Some(rel.clone()),
                        }),
                    }
                }
            }
        }
    }

    GovernanceEvaluation {
        schema_version: 1,
        kind: "governance_enforcement_evaluation".to_string(),
        status: if violations.is_empty() {
            "ok".to_string()
        } else {
            "failed".to_string()
        },
        rule_count: registry.rules.len(),
        violations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_loads_from_repo() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let registry = load_registry(&root).expect("registry load");
        assert_eq!(registry.schema_version, 1);
        assert!(!registry.rules.is_empty());
    }

    #[test]
    fn evaluation_runs_and_returns_shape() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let registry = load_registry(&root).expect("registry load");
        let result = evaluate_registry(&root, &registry);
        assert_eq!(result.schema_version, 1);
        assert_eq!(result.kind, "governance_enforcement_evaluation");
        assert_eq!(result.rule_count, registry.rules.len());
    }
}
