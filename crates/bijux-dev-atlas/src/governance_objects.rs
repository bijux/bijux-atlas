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
    pub reviewed_on: String,
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

fn read_domain_review_dates(repo_root: &Path) -> BTreeMap<String, String> {
    let path = repo_root.join("governance/domain-review-dates.json");
    let Ok(value) = read_json(&path) else {
        return BTreeMap::new();
    };
    value["domains"]
        .as_object()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(domain, payload)| {
            payload
                .get("reviewed_on")
                .and_then(|v| v.as_str())
                .map(|date| (domain, date.to_string()))
        })
        .collect::<BTreeMap<_, _>>()
}

pub fn collect_governance_objects(repo_root: &Path) -> Result<Vec<GovernanceObject>, String> {
    let mut objects = Vec::<GovernanceObject>::new();
    let domain_reviews = read_domain_review_dates(repo_root);

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
                reviewed_on: row["last_reviewed"].as_str().unwrap_or_default().to_string(),
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
                reviewed_on: row["last_reviewed"]
                    .as_str()
                    .map(str::to_string)
                    .or_else(|| domain_reviews.get("configs").cloned())
                    .unwrap_or_default(),
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
                reviewed_on: domain_reviews.get("ops").cloned().unwrap_or_default(),
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
                reviewed_on: row["last_reviewed"]
                    .as_str()
                    .map(str::to_string)
                    .or_else(|| domain_reviews.get("make").cloned())
                    .unwrap_or_default(),
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
                reviewed_on: row["last_reviewed"]
                    .as_str()
                    .map(str::to_string)
                    .or_else(|| domain_reviews.get("docker").cloned())
                    .unwrap_or_default(),
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
    let mut orphan_rows = Vec::<serde_json::Value>::new();
    let owner_ids = load_owner_ids(repo_root);
    let registry_map = load_domain_registry_map(repo_root);
    let allowed_registry_json = load_allowed_registry_json_paths(repo_root);
    let unknown_registry_json = list_unknown_registry_json_paths(repo_root, &allowed_registry_json);
    for rel in unknown_registry_json {
        errors.push(format!(
            "registry.json is forbidden outside approved paths: {rel}"
        ));
    }

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
        if object.authority_source.contains(',')
            || object.authority_source.contains(';')
            || object.authority_source.contains(' ')
        {
            errors.push(format!(
                "authority source must be exactly one path for {}: {}",
                object.id, object.authority_source
            ));
        }
        if !owner_is_valid(&owner_ids, &object.owner) {
            errors.push(format!(
                "owner `{}` is missing from owners system for {}",
                object.owner, object.id
            ));
        }
        if object.consumers.is_empty() {
            errors.push(format!("object has no consumers: {}", object.id));
        }
        for consumer in &object.consumers {
            if !consumer_is_valid(repo_root, consumer) {
                errors.push(format!(
                    "consumer is not a real path/command/service for {}: {}",
                    object.id, consumer
                ));
            }
        }
        if object.links.is_empty() {
            errors.push(format!("object has no links: {}", object.id));
        }
        if object.lifecycle == "stable" && object.reviewed_on.trim().is_empty() {
            errors.push(format!(
                "stable governance object is missing reviewed_on date: {}",
                object.id
            ));
        }
        for link in &object.links {
            if link.contains('*') || link.starts_with("http") {
                continue;
            }
            let path = repo_root.join(link);
            if !path.exists() {
                let reason = if link.starts_with("ops/") {
                    "ops_inventory_entry_missing"
                } else {
                    "path_not_found"
                };
                orphan_rows.push(serde_json::json!({
                    "id": object.id,
                    "domain": object.domain,
                    "link": link,
                    "reason": reason,
                }));
                errors.push(format!("object {} has missing link: {}", object.id, link));
            }
        }
        if object.domain == "docs" && !object.authority_source.starts_with("docs/_internal/registry/") {
            errors.push(format!(
                "docs authority source must be docs registry for {}",
                object.id
            ));
        }
        if object.domain == "configs" && !object.authority_source.starts_with("configs/") {
            errors.push(format!(
                "configs authority source must stay under configs/ for {}",
                object.id
            ));
        }
    }

    let required_domains = ["docs", "configs", "ops", "make", "docker"];
    for domain in required_domains {
        if !objects.iter().any(|obj| obj.domain == domain) {
            errors.push(format!("governance objects missing required domain: {domain}"));
        }
    }
    let mapped_domains = registry_map.keys().cloned().collect::<BTreeSet<_>>();
    for domain in mapped_domains {
        if !objects.iter().any(|obj| obj.domain == domain) {
            errors.push(format!(
                "registry domain has no governance objects emitted: {domain}"
            ));
        }
    }

    write_governance_orphan_report(repo_root, &orphan_rows);
    let registry_coverage_errors = validate_registry_map(repo_root, objects, &registry_map);
    errors.extend(registry_coverage_errors);
    let ssot_drift = detect_ssot_drift(&registry_map);
    errors.extend(ssot_drift);

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
      "required": ["id", "domain", "owner", "consumers", "lifecycle", "evidence", "links", "authority_source", "reviewed_on"],
      "fields": {
        "id": "{domain}:{kind}:{name}",
        "domain": "docs|configs|ops|make|docker|...",
        "owner": "stable owner id",
        "consumers": "list of real commands/paths/services",
        "lifecycle": "stable|experimental|deprecated",
        "evidence": "artifacts/governance/<domain>/...",
        "links": "authority links in repository",
        "authority_source": "single authority file path",
        "reviewed_on": "YYYY-MM-DD required for stable objects"
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

pub fn governance_coverage_score(objects: &[GovernanceObject]) -> serde_json::Value {
    let total = objects.len() as f64;
    let mut proof_ready = 0usize;
    for object in objects {
        let complete = !object.owner.is_empty()
            && !object.reviewed_on.is_empty()
            && !object.consumers.is_empty()
            && !object.evidence.is_empty()
            && !object.links.is_empty()
            && !object.authority_source.is_empty();
        if complete {
            proof_ready += 1;
        }
    }
    let score = if total == 0.0 {
        0.0
    } else {
        ((proof_ready as f64 / total) * 10000.0).round() / 100.0
    };
    serde_json::json!({
        "schema_version": 1,
        "kind": "governance_coverage_score",
        "total_objects": objects.len(),
        "proof_ready_objects": proof_ready,
        "coverage_percent": score
    })
}

pub fn governance_coverage_path(repo_root: &Path) -> PathBuf {
    repo_root.join("artifacts/governance/governance-coverage.json")
}

fn load_owner_ids(repo_root: &Path) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    if let Ok(value) = read_json(&repo_root.join("configs/owners/identities.json")) {
        for key in value["identities"]
            .as_object()
            .cloned()
            .unwrap_or_default()
            .keys()
        {
            ids.insert(key.clone());
        }
    }
    if let Ok(value) = read_json(&repo_root.join("docs/_internal/registry/owners.json")) {
        for owner in value["section_owners"]
            .as_object()
            .cloned()
            .unwrap_or_default()
            .into_values()
            .filter_map(|v| v.as_str().map(str::to_string))
        {
            ids.insert(owner);
        }
    }
    ids
}

fn consumer_is_valid(repo_root: &Path, consumer: &str) -> bool {
    if consumer.starts_with("bijux dev atlas ")
        || consumer.starts_with("make ")
        || consumer.starts_with("cargo ")
        || consumer.starts_with("service:")
    {
        return true;
    }
    repo_root.join(consumer).exists()
}

fn owner_is_valid(owner_ids: &BTreeSet<String>, owner: &str) -> bool {
    owner
        .split('+')
        .flat_map(|part| part.split(','))
        .map(str::trim)
        .map(|part| part.trim_matches('`').trim())
        .filter(|part| !part.is_empty())
        .all(|part| owner_ids.contains(part))
}

fn write_governance_orphan_report(repo_root: &Path, rows: &[serde_json::Value]) {
    let path = repo_root.join("artifacts/governance/governance-orphans.json");
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "governance_orphans",
        "orphans": rows,
    });
    if let Ok(raw) = serde_json::to_string_pretty(&payload) {
        let _ = fs::write(path, raw);
    }
}

fn load_domain_registry_map(repo_root: &Path) -> BTreeMap<String, Vec<String>> {
    let path = repo_root.join("governance/domain-registry-map.json");
    let Ok(value) = read_json(&path) else {
        return BTreeMap::new();
    };
    value["domains"]
        .as_object()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|(domain, payload)| {
            let registries = payload["registries"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect::<Vec<_>>();
            (domain, registries)
        })
        .collect::<BTreeMap<_, _>>()
}

fn validate_registry_map(
    repo_root: &Path,
    objects: &[GovernanceObject],
    map: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut errors = Vec::new();
    for (domain, registries) in map {
        for rel in registries {
            if !repo_root.join(rel).exists() {
                errors.push(format!(
                    "domain registry path missing for {domain}: {rel}"
                ));
            }
        }
        if !objects.iter().any(|obj| &obj.domain == domain) {
            errors.push(format!(
                "domain registry has no governance object emission: {domain}"
            ));
        }
    }
    errors
}

fn detect_ssot_drift(map: &BTreeMap<String, Vec<String>>) -> Vec<String> {
    let mut source_to_domain = BTreeMap::<String, String>::new();
    let mut errors = Vec::new();
    for (domain, registries) in map {
        for registry in registries {
            let key = registry.to_ascii_lowercase();
            if let Some(previous_domain) = source_to_domain.insert(key.clone(), domain.clone()) {
                if previous_domain != *domain {
                    errors.push(format!(
                        "ssot drift: registry key `{}` mapped to multiple domains: {} and {}",
                        registry, previous_domain, domain
                    ));
                }
            }
        }
    }
    errors
}

fn load_allowed_registry_json_paths(repo_root: &Path) -> BTreeSet<String> {
    let path = repo_root.join("governance/domain-registry-map.json");
    let mut allowed = BTreeSet::new();
    if let Ok(value) = read_json(&path) {
        for rel in value["approved_registry_json_paths"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
        {
            allowed.insert(rel);
        }
    }
    allowed
}

fn list_unknown_registry_json_paths(repo_root: &Path, allowed: &BTreeSet<String>) -> Vec<String> {
    let mut found = Vec::new();
    for path in walk_files(repo_root) {
        let rel = path.strip_prefix(repo_root).unwrap_or(&path);
        if rel.file_name().and_then(|v| v.to_str()) != Some("registry.json") {
            continue;
        }
        let rel_str = rel.display().to_string();
        if !allowed.contains(&rel_str) {
            found.push(rel_str);
        }
    }
    found.sort();
    found
}

fn walk_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|v| v.to_str()).unwrap_or_default();
            if name.starts_with(".git") || name == "artifacts" || name == "target" {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
    out
}
