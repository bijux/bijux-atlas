// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceObject {
    pub id: String,
    pub domain: String,
    pub owner: String,
    pub consumers: Vec<String>,
    pub lifecycle: String,
    pub evidence: Vec<String>,
    pub links: Vec<String>,
    pub authority_source: String,
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    serde_json::from_str(
        &fs::read_to_string(path).map_err(|e| format!("read {} failed: {e}", path.display()))?,
    )
    .map_err(|e| format!("parse {} failed: {e}", path.display()))
}

fn slug_from_path(path: &str) -> String {
    path.trim_end_matches(".md")
        .replace('/', ":")
        .replace('.', "-")
        .to_ascii_lowercase()
}

fn push_object(objects: &mut Vec<GovernanceObject>, object: GovernanceObject) {
    objects.push(object);
}

pub fn collect_governance_objects(repo_root: &Path) -> Result<Vec<GovernanceObject>, String> {
    let mut objects = Vec::<GovernanceObject>::new();

    let docs_registry = read_json(&repo_root.join("docs/_internal/registry/registry.json"))?;
    for row in docs_registry["documents"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
        let path = row["path"].as_str().unwrap_or_default();
        if !path.starts_with("docs/") || path.starts_with("docs/_internal/") {
            continue;
        }
        let lifecycle = row["lifecycle"]
            .as_str()
            .or_else(|| row["stability"].as_str())
            .unwrap_or("stable");
        if lifecycle != "stable" {
            continue;
        }
        push_object(
            &mut objects,
            GovernanceObject {
                id: format!("docs:page:{}", slug_from_path(path)),
                domain: "docs".to_string(),
                owner: row["owner"].as_str().unwrap_or("docs-governance").to_string(),
                consumers: vec!["docs/index.md".to_string()],
                lifecycle: lifecycle.to_string(),
                evidence: vec!["artifacts/governance/docs/pages.json".to_string()],
                links: vec![path.to_string()],
                authority_source: "docs/_internal/registry/registry.json".to_string(),
            },
        );
    }

    let configs_inventory = read_json(&repo_root.join("configs/inventory/configs.json"))?;
    for row in configs_inventory["groups"].as_array().cloned().unwrap_or_default() {
        let Some(name) = row["name"].as_str() else {
            continue;
        };
        let owner = row["owner"].as_str().unwrap_or("platform");
        let lifecycle = row["stability"].as_str().unwrap_or("stable");
        let consumers = row["tool_entrypoints"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect::<Vec<_>>();
        let mut links = Vec::<String>::new();
        links.extend(
            row["public_files"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|v| v.as_str().map(str::to_string)),
        );
        links.extend(
            row["schemas"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|v| v.as_str().map(str::to_string)),
        );
        push_object(
            &mut objects,
            GovernanceObject {
                id: format!("configs:group:{name}"),
                domain: "configs".to_string(),
                owner: owner.to_string(),
                consumers,
                lifecycle: lifecycle.to_string(),
                evidence: vec!["artifacts/governance/configs/groups.json".to_string()],
                links,
                authority_source: "configs/inventory/configs.json".to_string(),
            },
        );
    }

    let ops_contracts = read_json(&repo_root.join("ops/inventory/contracts.json"))?;
    let ops_contract_paths = ops_contracts["contracts"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v["path"].as_str().map(str::to_string))
        .filter(|path| repo_root.join(path).exists())
        .collect::<Vec<_>>();
    for contract_id in ops_contracts["contract_ids"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_string))
    {
        push_object(
            &mut objects,
            GovernanceObject {
                id: format!("ops:contract:{}", contract_id.to_ascii_lowercase()),
                domain: "ops".to_string(),
                owner: "bijux-atlas-operations".to_string(),
                consumers: vec!["bijux dev atlas contracts ops".to_string()],
                lifecycle: "stable".to_string(),
                evidence: vec!["artifacts/governance/ops/contracts.json".to_string()],
                links: ops_contract_paths.clone(),
                authority_source: "ops/inventory/contracts.json".to_string(),
            },
        );
    }

    let make_targets = read_json(&repo_root.join("configs/ops/make-target-registry.json"))?;
    for row in make_targets["targets"].as_array().cloned().unwrap_or_default() {
        let Some(name) = row["name"].as_str() else {
            continue;
        };
        let consumers = row["used_in"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect::<Vec<_>>();
        let links = row["defined_in"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .map(|path| {
                if path.starts_with("makefiles/") {
                    path.replacen("makefiles/", "make/", 1)
                } else {
                    path
                }
            })
            .collect::<Vec<_>>();
        let normalized_consumers = if consumers.is_empty() {
            vec![format!("make {name}")]
        } else {
            consumers
        };
        push_object(
            &mut objects,
            GovernanceObject {
                id: format!("make:target:{}", name.to_ascii_lowercase()),
                domain: "make".to_string(),
                owner: "platform".to_string(),
                consumers: normalized_consumers,
                lifecycle: "stable".to_string(),
                evidence: vec!["artifacts/governance/make/targets.json".to_string()],
                links,
                authority_source: "configs/ops/make-target-registry.json".to_string(),
            },
        );
    }

    let docker_manifest = read_json(&repo_root.join("docker/images.manifest.json"))?;
    for row in docker_manifest["images"].as_array().cloned().unwrap_or_default() {
        let Some(name) = row["name"].as_str() else {
            continue;
        };
        let dockerfile = row["dockerfile"].as_str().unwrap_or_default().to_string();
        push_object(
            &mut objects,
            GovernanceObject {
                id: format!("docker:image:{}", name.to_ascii_lowercase()),
                domain: "docker".to_string(),
                owner: "platform".to_string(),
                consumers: vec!["bijux dev atlas contracts docker".to_string()],
                lifecycle: "stable".to_string(),
                evidence: vec!["artifacts/governance/docker/images.json".to_string()],
                links: vec![
                    dockerfile,
                    "docker/images.manifest.json".to_string(),
                    "docker/policy.json".to_string(),
                ],
                authority_source: "docker/images.manifest.json".to_string(),
            },
        );
    }

    objects.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(objects)
}

pub fn governance_summary(objects: &[GovernanceObject]) -> BTreeMap<String, usize> {
    let mut by_domain = BTreeMap::<String, usize>::new();
    for obj in objects {
        *by_domain.entry(obj.domain.clone()).or_insert(0) += 1;
    }
    by_domain
}

pub fn validate_governance_objects(
    repo_root: &Path,
    objects: &[GovernanceObject],
) -> Vec<String> {
    let allowed_lifecycle = ["stable", "experimental", "deprecated"]
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut ids = BTreeSet::<String>::new();
    let mut errors = Vec::<String>::new();

    for object in objects {
        if !ids.insert(object.id.clone()) {
            errors.push(format!("duplicate governance object id: {}", object.id));
        }
        if !object.id.starts_with(&format!("{}:", object.domain)) {
            errors.push(format!(
                "governance object id must use domain prefix `{}`: {}",
                object.domain, object.id
            ));
        }
        if !allowed_lifecycle.contains(object.lifecycle.as_str()) {
            errors.push(format!(
                "unsupported lifecycle `{}` for {}",
                object.lifecycle, object.id
            ));
        }
        if object.evidence.is_empty() {
            errors.push(format!("missing evidence for {}", object.id));
        }
        if object
            .evidence
            .iter()
            .any(|path| !path.starts_with("artifacts/governance/"))
        {
            errors.push(format!(
                "evidence path must start with artifacts/governance/ for {}",
                object.id
            ));
        }
        if object.authority_source.is_empty() {
            errors.push(format!("missing authority source for {}", object.id));
        }
        if object.consumers.is_empty() {
            errors.push(format!("object has no consumers: {}", object.id));
        }
        if object.links.is_empty() {
            errors.push(format!("object has no links: {}", object.id));
        }
        for link in &object.links {
            if link.contains('*') || link.starts_with("http") {
                continue;
            }
            let path = repo_root.join(link);
            if !path.exists() {
                errors.push(format!("object {} has missing link: {}", object.id, link));
            }
        }
    }

    let required_domains = ["docs", "configs", "ops", "make", "docker"];
    for domain in required_domains {
        if !objects.iter().any(|obj| obj.domain == domain) {
            errors.push(format!("governance objects missing required domain: {domain}"));
        }
    }

    errors.sort();
    errors
}

pub fn find_governance_object<'a>(
    objects: &'a [GovernanceObject],
    id: &str,
) -> Option<&'a GovernanceObject> {
    objects.iter().find(|obj| obj.id == id)
}

pub fn governance_object_schema() -> serde_json::Value {
    serde_json::json!({
      "schema_version": 1,
      "kind": "governance_object_schema",
      "required": ["id", "domain", "owner", "consumers", "lifecycle", "evidence", "links", "authority_source"],
      "fields": {
        "id": "{domain}:{kind}:{name}",
        "domain": "docs|configs|ops|make|docker|...",
        "owner": "stable owner id",
        "consumers": "list of real commands/paths/services",
        "lifecycle": "stable|experimental|deprecated",
        "evidence": "artifacts/governance/<domain>/...",
        "links": "authority links in repository",
        "authority_source": "single authority file path"
      }
    })
}

pub fn governance_summary_markdown(objects: &[GovernanceObject]) -> String {
    let by_domain = governance_summary(objects);
    let mut out = String::new();
    out.push_str("# Governance Summary\n\n");
    out.push_str("| Domain | Objects |\n| --- | --- |\n");
    for (domain, count) in by_domain {
        out.push_str(&format!("| `{domain}` | `{count}` |\n"));
    }
    out
}

pub fn governance_summary_paths(repo_root: &Path) -> (PathBuf, PathBuf) {
    (
        repo_root.join("artifacts/governance/governance-graph.json"),
        repo_root.join("artifacts/governance/governance-summary.md"),
    )
}
