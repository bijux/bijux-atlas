// SPDX-License-Identifier: Apache-2.0

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ScenarioManifest {
    pub schema_version: u64,
    pub scenarios: Vec<ScenarioSpec>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ScenarioSpec {
    pub id: String,
    pub description: String,
    pub action_id: String,
    #[serde(default)]
    pub entrypoint: Option<String>,
    #[serde(default)]
    pub compose: BTreeMap<String, bool>,
    #[serde(default)]
    pub evidence_class: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpgradeScenarioSpec {
    pub schema_version: u64,
    pub id: String,
    pub from_version: String,
    pub to_version: String,
    pub kind: String,
    #[serde(default)]
    pub failure_expected: bool,
    #[serde(default)]
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FailureScenarioSpec {
    pub schema_version: u64,
    pub id: String,
    pub failure_mode: String,
    #[serde(default)]
    pub failure_expected: bool,
    pub expected_behavior: String,
    pub recommended_action: String,
}

pub fn deterministic_scenario_run_id(scenario_id: &str, mode: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("scenario::{scenario_id}::{mode}").as_bytes());
    hex::encode(hasher.finalize()).chars().take(12).collect()
}

pub fn load_upgrade_spec(
    repo_root: &Path,
    scenario_id: &str,
) -> Result<Option<UpgradeScenarioSpec>, String> {
    let path = repo_root
        .join("ops/e2e/scenarios/upgrade")
        .join(format!("{scenario_id}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let parsed: UpgradeScenarioSpec = serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    if parsed.schema_version != 1 {
        return Err(format!(
            "{}: expected schema_version=1, got {}",
            path.display(),
            parsed.schema_version
        ));
    }
    if parsed.id != scenario_id {
        return Err(format!(
            "{}: scenario id mismatch (`{}` vs `{}`)",
            path.display(),
            parsed.id,
            scenario_id
        ));
    }
    Ok(Some(parsed))
}

pub fn load_failure_spec(
    repo_root: &Path,
    scenario_id: &str,
) -> Result<Option<FailureScenarioSpec>, String> {
    let path = repo_root
        .join("ops/e2e/scenarios/failure")
        .join(format!("{scenario_id}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let parsed: FailureScenarioSpec = serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    if parsed.schema_version != 1 {
        return Err(format!(
            "{}: expected schema_version=1, got {}",
            path.display(),
            parsed.schema_version
        ));
    }
    if parsed.id != scenario_id {
        return Err(format!(
            "{}: scenario id mismatch (`{}` vs `{}`)",
            path.display(),
            parsed.id,
            scenario_id
        ));
    }
    Ok(Some(parsed))
}

pub fn load_scenario_manifest(repo_root: &Path) -> Result<ScenarioManifest, String> {
    let path = repo_root.join("ops/e2e/scenarios/scenarios.json");
    let raw = std::fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let parsed: ScenarioManifest = serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    if parsed.schema_version != 1 {
        return Err(format!(
            "ops/e2e/scenarios/scenarios.json: expected schema_version=1, got {}",
            parsed.schema_version
        ));
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::{deterministic_scenario_run_id, load_scenario_manifest};
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("repository root")
            .to_path_buf()
    }

    #[test]
    fn deterministic_scenario_run_id_is_stable() {
        assert_eq!(
            deterministic_scenario_run_id("rollback-minor", "evidence"),
            deterministic_scenario_run_id("rollback-minor", "evidence")
        );
    }

    #[test]
    fn load_scenario_manifest_reads_owned_catalog() {
        let manifest = load_scenario_manifest(&repo_root()).expect("scenario manifest");
        assert!(
            manifest
                .scenarios
                .iter()
                .any(|scenario| scenario.id == "upgrade-minor"),
            "expected upgrade-minor scenario entry"
        );
    }
}
