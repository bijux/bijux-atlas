use super::*;

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
                errors.push(format!("domain registry path missing for {domain}: {rel}"));
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
            let name = path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or_default();
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

pub(super) fn validate_governance_objects(
    repo_root: &Path,
    objects: &[GovernanceObject],
) -> GovernanceValidationReport {
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
        if object.domain == "docs"
            && !object
                .authority_source
                .starts_with("docs/_internal/registry/")
        {
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
            errors.push(format!(
                "governance objects missing required domain: {domain}"
            ));
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

    let registry_coverage_errors = validate_registry_map(repo_root, objects, &registry_map);
    errors.extend(registry_coverage_errors);
    let ssot_drift = detect_ssot_drift(&registry_map);
    errors.extend(ssot_drift);

    errors.sort();
    GovernanceValidationReport {
        errors,
        orphan_rows,
    }
}
