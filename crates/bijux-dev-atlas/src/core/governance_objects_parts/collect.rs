use super::*;

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
    let path = repo_root.join("ops/governance/repository/domain-review-dates.json");
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

pub(super) fn collect_governance_objects(
    repo_root: &Path,
) -> Result<Vec<GovernanceObject>, String> {
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
                owner: row["owner"]
                    .as_str()
                    .unwrap_or("docs-governance")
                    .to_string(),
                consumers: vec!["docs/index.md".to_string()],
                lifecycle: lifecycle.to_string(),
                evidence: vec!["artifacts/governance/docs/pages.json".to_string()],
                links: vec![path.to_string()],
                authority_source: "docs/_internal/registry/registry.json".to_string(),
                reviewed_on: row["last_reviewed"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
            },
        );
    }

    let configs_inventory = read_json(&repo_root.join("configs/inventory/configs.json"))?;
    for row in configs_inventory["groups"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
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
                consumers: vec!["bijux dev atlas contract run --domain ops".to_string()],
                lifecycle: "stable".to_string(),
                evidence: vec!["artifacts/governance/ops/contracts.json".to_string()],
                links: ops_contract_paths.clone(),
                authority_source: "ops/inventory/contracts.json".to_string(),
                reviewed_on: domain_reviews.get("ops").cloned().unwrap_or_default(),
            },
        );
    }

    let make_targets = read_json(&repo_root.join("configs/ops/make-target-registry.json"))?;
    for row in make_targets["targets"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
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

    let docker_manifest = read_json(&repo_root.join("ops/docker/images.manifest.json"))?;
    for row in docker_manifest["images"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
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
                consumers: vec!["bijux dev atlas contract run --domain docker".to_string()],
                lifecycle: "stable".to_string(),
                evidence: vec!["artifacts/governance/ops/docker/images.json".to_string()],
                links: vec![
                    dockerfile,
                    "ops/docker/images.manifest.json".to_string(),
                    "ops/docker/policy.json".to_string(),
                ],
                authority_source: "ops/docker/images.manifest.json".to_string(),
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
