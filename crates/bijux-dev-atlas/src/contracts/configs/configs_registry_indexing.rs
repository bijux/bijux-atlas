fn registry_index(repo_root: &Path) -> Result<RegistryIndex, String> {
    let text = read_text(&repo_root.join(REGISTRY_PATH))?;
    let registry = serde_json::from_str::<ConfigsRegistry>(&text)
        .map_err(|err| format!("parse {REGISTRY_PATH} failed: {err}"))?;
    let files = all_config_files(repo_root)?;
    let excluded_files = files
        .iter()
        .filter(|file| matches_any(registry.exclusions.iter().map(|item| &item.pattern), file))
        .cloned()
        .collect::<BTreeSet<_>>();
    let root_files = registry.root_files.iter().cloned().collect::<BTreeSet<_>>();
    let mut group_files = BTreeMap::new();
    for group in &registry.groups {
        let mut bucket = GroupFiles::default();
        for file in &files {
            if excluded_files.contains(file) {
                continue;
            }
            if matches_any(group.public_files.iter(), file) {
                bucket.public.insert(file.clone());
            }
            if matches_any(group.internal_files.iter(), file) {
                bucket.internal.insert(file.clone());
            }
            if matches_any(group.generated_files.iter(), file) {
                bucket.generated.insert(file.clone());
            }
        }
        group_files.insert(group.name.clone(), bucket);
    }
    let contract_doc = read_text(&repo_root.join("configs/CONTRACT.md"))?;
    let contract_surface_ids = contract_doc
        .lines()
        .filter_map(|line| {
            let mut rest = line;
            while let Some(start) = rest.find('`') {
                let after = &rest[start + 1..];
                let Some(end) = after.find('`') else {
                    break;
                };
                let token = &after[..end];
                if token.starts_with("CFG-") {
                    return Some(token.to_string());
                }
                rest = &after[end + 1..];
            }
            None
        })
        .collect::<BTreeSet<_>>();
    Ok(RegistryIndex {
        registry,
        files,
        excluded_files,
        root_files,
        group_files,
        contract_surface_ids,
    })
}

fn generated_index_json(repo_root: &Path) -> Result<serde_json::Value, String> {
    let index = registry_index(repo_root)?;
    let groups = index
        .registry
        .groups
        .iter()
        .map(|group| {
            let files = index
                .group_files
                .get(&group.name)
                .cloned()
                .unwrap_or_default();
            let covered_files = files.all().into_iter().collect::<Vec<_>>();
            serde_json::json!({
                "name": group.name,
                "owner": group.owner,
                "schema_owner": group.schema_owner,
                "stability": group.stability,
                "tool_entrypoints": group.tool_entrypoints,
                "counts": {
                    "public": files.public.len(),
                    "internal": files.internal.len(),
                    "generated": files.generated.len(),
                    "covered": covered_files.len(),
                    "schemas": group.schemas.len()
                },
                "files": covered_files,
                "schemas": group.schemas
            })
        })
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "configs-index",
        "registry_path": REGISTRY_PATH,
        "max_groups": index.registry.max_groups,
        "max_depth": index.registry.max_depth,
        "max_group_depth": index.registry.max_group_depth,
        "root_files": index.registry.root_files,
        "groups": groups,
        "exclusions": index.registry.exclusions.iter().map(|item| serde_json::json!({
            "pattern": item.pattern,
            "reason": item.reason,
            "approved_by": item.approved_by,
            "expires_on": item.expires_on
        })).collect::<Vec<_>>()
    }))
}

pub fn list_payload(repo_root: &Path) -> Result<serde_json::Value, String> {
    let index = registry_index(repo_root)?;
    let contract_surface = cfg_contract_coverage_payload(repo_root)?;
    let rows = index
        .registry
        .groups
        .iter()
        .map(|group| {
            let files = index
                .group_files
                .get(&group.name)
                .cloned()
                .unwrap_or_default();
            serde_json::json!({
                "group": group.name,
                "owner": group.owner,
                "schema_owner": group.schema_owner,
                "stability": group.stability,
                "tool_entrypoints": group.tool_entrypoints,
                "counts": {
                    "public": files.public.len(),
                    "internal": files.internal.len(),
                    "generated": files.generated.len(),
                    "schemas": group.schemas.len()
                }
            })
        })
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "configs",
        "registry_path": REGISTRY_PATH,
        "contract_surface": contract_surface,
        "groups": rows
    }))
}

pub fn ensure_generated_index(repo_root: &Path) -> Result<String, String> {
    let path = repo_root.join("configs/_generated/configs-index.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
    }
    let payload = generated_index_json(repo_root)?;
    std::fs::write(
        &path,
        canonical_json_string(&payload)
            .map_err(|err| format!("encode {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("write {} failed: {err}", path.display()))?;
    Ok(path.display().to_string())
}

fn schema_index_json(repo_root: &Path) -> Result<serde_json::Value, String> {
    let schemas = read_schemas(repo_root)?;
    let input_schemas = walk_files_under(&repo_root.join("configs/schema"))
        .into_iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .filter_map(|path| {
            let rel = path
                .strip_prefix(repo_root)
                .ok()?
                .display()
                .to_string()
                .replace('\\', "/");
            if rel.starts_with("configs/schema/generated/") || !schema_like(&rel) {
                None
            } else {
                Some(rel)
            }
        })
        .collect::<BTreeSet<_>>();
    let output_schemas = walk_files_under(&repo_root.join("configs/contracts"))
        .into_iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .filter_map(|path| {
            path.strip_prefix(repo_root)
                .ok()
                .map(|rel| rel.display().to_string().replace('\\', "/"))
        })
        .collect::<BTreeSet<_>>();
    let referenced = schemas.files.values().cloned().collect::<BTreeSet<_>>();
    let orphan_inputs = input_schemas
        .iter()
        .filter(|path| !referenced.contains(*path))
        .cloned()
        .collect::<Vec<_>>();
    let patterns = schemas.files.keys().cloned().collect::<Vec<_>>();
    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "configs-schema-index",
        "schema_map_path": SCHEMAS_PATH,
        "input_schemas": input_schemas,
        "output_schemas": output_schemas,
        "referenced_schemas": referenced,
        "orphan_input_schemas": orphan_inputs,
        "mapped_file_patterns": patterns
    }))
}

pub fn ensure_generated_schema_index(repo_root: &Path) -> Result<String, String> {
    let path = repo_root.join("configs/schema/generated/schema-index.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
    }
    let payload = schema_index_json(repo_root)?;
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&payload)
            .map(|text| format!("{text}\n"))
            .map_err(|err| format!("encode {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("write {} failed: {err}", path.display()))?;
    Ok(path.display().to_string())
}

pub fn cfg_contract_coverage_payload(repo_root: &Path) -> Result<serde_json::Value, String> {
    let surface = read_contract_surface(repo_root)?;
    let surface_text = read_text(&repo_root.join(CONTRACT_SURFACE_PATH))?;
    let executable_contracts = contracts(repo_root)?;
    let total_tests = executable_contracts
        .iter()
        .map(|contract| contract.tests.len())
        .sum::<usize>();
    let mapped_checks = surface
        .contracts
        .iter()
        .map(|row| row.enforced_by.test_id.clone())
        .collect::<BTreeSet<_>>();
    let executable_checks = executable_contracts
        .iter()
        .flat_map(|contract| contract.tests.iter().map(|test| test.id.0.clone()))
        .collect::<BTreeSet<_>>();
    let mapped_count = mapped_checks.intersection(&executable_checks).count();
    let coverage_pct = if total_tests == 0 {
        100
    } else {
        ((mapped_count * 100) / total_tests) as u64
    };
    let unmapped_checks = executable_checks
        .difference(&mapped_checks)
        .cloned()
        .collect::<Vec<_>>();
    let contract_type_counts =
        surface
            .contracts
            .iter()
            .fold(BTreeMap::<String, usize>::new(), |mut counts, row| {
                *counts.entry(row.contract_type.clone()).or_default() += 1;
                counts
            });
    let mut hasher = Sha256::new();
    hasher.update(surface_text.as_bytes());
    let registry_sha256 = format!("{:x}", hasher.finalize());
    Ok(serde_json::json!({
        "schema_version": 1,
        "registry_path": CONTRACT_SURFACE_PATH,
        "registry_sha256": registry_sha256,
        "contract_count": surface.contracts.len(),
        "mapped_checks": mapped_count,
        "total_checks": total_tests,
        "coverage_pct": coverage_pct,
        "unmapped_checks": unmapped_checks,
        "contract_type_counts": contract_type_counts
    }))
}

pub fn write_cfg_contract_coverage_artifact(
    repo_root: &Path,
    artifacts_root: &Path,
    run_id: &str,
) -> Result<String, String> {
    let payload = cfg_contract_coverage_payload(repo_root)?;
    let out_dir = artifacts_root
        .join("atlas-dev")
        .join("configs")
        .join(run_id);
    std::fs::create_dir_all(&out_dir)
        .map_err(|err| format!("create {} failed: {err}", out_dir.display()))?;
    let path = out_dir.join("cfg-contract-coverage.json");
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("write {} failed: {err}", path.display()))?;
    Ok(path.display().to_string())
}

fn parse_checked_files(index: &RegistryIndex, exts: &[&str]) -> Vec<String> {
    config_files_without_exclusions(index)
        .into_iter()
        .filter(|file| {
            let path = Path::new(file);
            let ext = path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            exts.contains(&ext)
        })
        .collect()
}

fn parse_supported_config_file(path: &Path) -> Result<(), String> {
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let text =
        read_text(path).map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    match ext {
        "json" => {
            serde_json::from_str::<serde_json::Value>(&text)
                .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
        }
        "jsonc" => {
            let sanitized = text
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n");
            serde_json::from_str::<serde_json::Value>(&sanitized)
                .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
        }
        "toml" => {
            toml::from_str::<toml::Value>(&text)
                .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
        }
        "yaml" | "yml" => {
            serde_yaml::from_str::<serde_yaml::Value>(&text)
                .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
        }
        _ => {}
    }
    Ok(())
}

fn walk_files_under(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk_files_under(&path));
        } else {
            out.push(path);
        }
    }
    out.sort();
    out
}

fn root_config_files(index: &RegistryIndex) -> BTreeSet<String> {
    index
        .files
        .iter()
        .filter(|file| path_depth(file) == 1)
        .cloned()
        .collect()
}

fn config_files_without_exclusions(index: &RegistryIndex) -> Vec<String> {
    index
        .files
        .iter()
        .filter(|file| !index.excluded_files.contains(*file))
        .cloned()
        .collect()
}

fn json_like(path: &str) -> bool {
    path.ends_with(".json") || path.ends_with(".jsonc")
}

fn schema_like(path: &str) -> bool {
    path.ends_with(".schema.json")
}

fn canonicalize_json_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let sorted = map
                .iter()
                .map(|(key, child)| (key.clone(), canonicalize_json_value(child)))
                .collect::<BTreeMap<_, _>>();
            let mut normalized = serde_json::Map::new();
            for (key, child) in sorted {
                normalized.insert(key, child);
            }
            serde_json::Value::Object(normalized)
        }
        serde_json::Value::Array(items) => serde_json::Value::Array(
            items
                .iter()
                .map(canonicalize_json_value)
                .collect::<Vec<_>>(),
        ),
        _ => value.clone(),
    }
}

fn canonical_json_string(value: &serde_json::Value) -> Result<String, String> {
    serde_json::to_string_pretty(&canonicalize_json_value(value))
        .map(|text| format!("{text}\n"))
        .map_err(|err| format!("render canonical json failed: {err}"))
}
