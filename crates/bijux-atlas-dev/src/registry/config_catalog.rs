// SPDX-License-Identifier: Apache-2.0
//! Config registry indexing and explain surfaces.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

const REGISTRY_PATH: &str = "configs/registry/inventory/configs.json";
const OWNERS_PATH: &str = "configs/registry/owners.json";
const CONSUMERS_PATH: &str = "configs/registry/consumers.json";
const SCHEMAS_PATH: &str = "configs/registry/schemas.json";

#[derive(Clone, Deserialize)]
struct ConfigsRegistry {
    schema_version: u64,
    max_groups: usize,
    max_depth: usize,
    max_group_depth: usize,
    root_files: Vec<String>,
    groups: Vec<ConfigsGroup>,
    #[serde(default)]
    exclusions: Vec<ConfigsExclusion>,
}

#[derive(Clone, Deserialize)]
struct ConfigsGroup {
    name: String,
    owner: String,
    schema_owner: String,
    stability: String,
    tool_entrypoints: Vec<String>,
    public_files: Vec<String>,
    internal_files: Vec<String>,
    generated_files: Vec<String>,
    schemas: Vec<String>,
}

#[derive(Clone, Deserialize)]
struct ConfigsExclusion {
    pattern: String,
    reason: String,
    approved_by: Option<String>,
    expires_on: Option<String>,
}

#[derive(Clone, Deserialize)]
struct ConfigsOwners {
    #[serde(default)]
    files: BTreeMap<String, String>,
    groups: BTreeMap<String, String>,
}

#[derive(Clone, Deserialize)]
struct ConfigsConsumers {
    #[serde(default)]
    files: BTreeMap<String, Vec<String>>,
    groups: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Deserialize)]
struct ConfigsSchemas {
    files: BTreeMap<String, String>,
}

#[derive(Clone)]
struct RegistryIndex {
    registry: ConfigsRegistry,
    root_files: BTreeSet<String>,
    group_files: BTreeMap<String, GroupFiles>,
}

#[derive(Clone, Default)]
struct GroupFiles {
    public: BTreeSet<String>,
    internal: BTreeSet<String>,
    generated: BTreeSet<String>,
}

impl GroupFiles {
    fn all(&self) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        out.extend(self.public.iter().cloned());
        out.extend(self.internal.iter().cloned());
        out.extend(self.generated.iter().cloned());
        out
    }
}

pub fn generated_index_payload(repo_root: &Path) -> Result<serde_json::Value, String> {
    generated_index_json(repo_root)
}

pub fn list_payload(repo_root: &Path) -> Result<serde_json::Value, String> {
    let index = registry_index(repo_root)?;
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
        "groups": rows.clone(),
        "rows": rows
    }))
}

pub fn graph_payload(repo_root: &Path) -> Result<serde_json::Value, String> {
    let index = registry_index(repo_root)?;
    let owners = read_owners(repo_root)?;
    let consumers = read_consumers(repo_root)?;
    let schemas = read_schemas(repo_root)?;

    let mut nodes = Vec::<serde_json::Value>::new();
    let mut edges = Vec::<serde_json::Value>::new();
    let mut all_covered = BTreeSet::<String>::new();
    let mut orphan_nodes = Vec::<String>::new();

    let is_governed_structured = |path: &str| {
        Path::new(path)
            .extension()
            .and_then(|v| v.to_str())
            .map(|ext| matches!(ext, "json" | "jsonc" | "toml" | "yaml" | "yml"))
            .unwrap_or(false)
    };

    for file in &index.root_files {
        all_covered.insert(file.clone());
        let owner = owners
            .files
            .get(file)
            .cloned()
            .unwrap_or_else(|| "platform".to_string());
        let file_consumers = matching_file_consumers(&consumers, file);
        let schema = matched_schema_path(&schemas, file);
        if file_consumers.is_empty() && is_governed_structured(file) {
            orphan_nodes.push(file.clone());
        }
        nodes.push(serde_json::json!({
            "id": file,
            "kind": "config_file",
            "group": serde_json::Value::Null,
            "owner": owner,
            "consumers": file_consumers,
            "schema": schema,
            "visibility": "root",
        }));
    }

    for group in &index.registry.groups {
        let files = index
            .group_files
            .get(&group.name)
            .cloned()
            .unwrap_or_default()
            .all();
        for file in files {
            all_covered.insert(file.clone());
            let file_consumers = matching_file_consumers(&consumers, &file);
            let effective_consumers = if file_consumers.is_empty() {
                consumers
                    .groups
                    .get(&group.name)
                    .cloned()
                    .unwrap_or_default()
            } else {
                file_consumers
            };
            if effective_consumers.is_empty() && is_governed_structured(&file) {
                orphan_nodes.push(file.clone());
            }
            let visibility = if index
                .group_files
                .get(&group.name)
                .is_some_and(|bucket| bucket.public.contains(&file))
            {
                "public"
            } else if index
                .group_files
                .get(&group.name)
                .is_some_and(|bucket| bucket.internal.contains(&file))
            {
                "internal"
            } else {
                "generated"
            };
            let schema = matched_schema_path(&schemas, &file);
            nodes.push(serde_json::json!({
                "id": file,
                "kind": "config_file",
                "group": group.name,
                "owner": owners.groups.get(&group.name).cloned().unwrap_or_else(|| group.owner.clone()),
                "consumers": effective_consumers,
                "schema": schema,
                "visibility": visibility,
                "lifecycle": group.stability
            }));
        }
        edges.push(serde_json::json!({
            "from": format!("group:{}", group.name),
            "to": format!("owner:{}", group.owner),
            "kind": "group_owner"
        }));
        edges.push(serde_json::json!({
            "from": format!("group:{}", group.name),
            "to": format!("schema_owner:{}", group.schema_owner),
            "kind": "group_schema_owner"
        }));
    }

    nodes.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    edges.sort_by(|a, b| {
        a["from"]
            .as_str()
            .cmp(&b["from"].as_str())
            .then(a["to"].as_str().cmp(&b["to"].as_str()))
            .then(a["kind"].as_str().cmp(&b["kind"].as_str()))
    });
    orphan_nodes.sort();
    orphan_nodes.dedup();

    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "configs_graph",
        "registry_path": REGISTRY_PATH,
        "nodes": nodes,
        "edges": edges,
        "orphans": orphan_nodes,
        "counts": {
            "files": all_covered.len(),
            "nodes": nodes.len(),
            "edges": edges.len()
        }
    }))
}

pub fn explain_payload(repo_root: &Path, file: &str) -> Result<serde_json::Value, String> {
    let index = registry_index(repo_root)?;
    let owners = read_owners(repo_root)?;
    let consumers = read_consumers(repo_root)?;
    let schemas = read_schemas(repo_root)?;
    let normalized = file.replace('\\', "/");
    if index.root_files.contains(&normalized) {
        return Ok(serde_json::json!({
            "schema_version": 1,
            "kind": "configs_explain",
            "path": normalized,
            "group": serde_json::Value::Null,
            "visibility": "root",
            "owner": serde_json::Value::Null,
            "consumers": matching_file_consumers(&consumers, &normalized),
            "schema": matched_schema_path(&schemas, &normalized),
            "schema_owner": serde_json::Value::Null,
            "stability": "stable",
            "tool_entrypoints": [],
            "summary": "root configs authority file"
        }));
    }
    for exclusion in &index.registry.exclusions {
        if wildcard_match(&exclusion.pattern, &normalized) {
            return Ok(serde_json::json!({
                "schema_version": 1,
                "kind": "configs_explain",
                "path": normalized,
                "group": serde_json::Value::Null,
                "visibility": "excluded",
                "owner": serde_json::Value::Null,
                "consumers": [],
                "schema": serde_json::Value::Null,
                "schema_owner": serde_json::Value::Null,
                "stability": serde_json::Value::Null,
                "tool_entrypoints": [],
                "summary": exclusion.reason
            }));
        }
    }
    for group in &index.registry.groups {
        let visibility = if matches_any(group.public_files.iter(), &normalized) {
            Some("public")
        } else if matches_any(group.internal_files.iter(), &normalized) {
            Some("internal")
        } else if matches_any(group.generated_files.iter(), &normalized) {
            Some("generated")
        } else {
            None
        };
        if let Some(visibility) = visibility {
            let file_consumers = matching_file_consumers(&consumers, &normalized);
            let effective_consumers = if file_consumers.is_empty() {
                consumers
                    .groups
                    .get(&group.name)
                    .cloned()
                    .unwrap_or_default()
            } else {
                file_consumers
            };
            return Ok(serde_json::json!({
                "schema_version": 1,
                "kind": "configs_explain",
                "path": normalized,
                "group": group.name,
                "visibility": visibility,
                "owner": owners.groups.get(&group.name).cloned().unwrap_or_else(|| group.owner.clone()),
                "consumers": effective_consumers,
                "schema": matched_schema_path(&schemas, &normalized),
                "schema_owner": group.schema_owner,
                "stability": group.stability,
                "tool_entrypoints": group.tool_entrypoints,
                "summary": format!("configs group `{}` {} file", group.name, visibility)
            }));
        }
    }
    Err(format!(
        "config path `{normalized}` is not covered by {REGISTRY_PATH}"
    ))
}

pub fn ensure_generated_index(repo_root: &Path) -> Result<String, String> {
    let path = repo_root.join("configs/generated/configs-index.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
    }
    let payload = generated_index_json(repo_root)?;
    fs::write(
        &path,
        canonical_json_string(&payload)
            .map_err(|err| format!("encode {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("write {} failed: {err}", path.display()))?;
    Ok(path.display().to_string())
}

pub fn ensure_generated_schema_index(repo_root: &Path) -> Result<String, String> {
    let path = repo_root.join("configs/schemas/registry/generated/schema-index.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
    }
    let payload = schema_index_json(repo_root)?;
    fs::write(
        &path,
        serde_json::to_string_pretty(&payload)
            .map(|text| format!("{text}\n"))
            .map_err(|err| format!("encode {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("write {} failed: {err}", path.display()))?;
    Ok(path.display().to_string())
}

fn registry_index(repo_root: &Path) -> Result<RegistryIndex, String> {
    let text = read_text(&repo_root.join(REGISTRY_PATH))?;
    let registry = serde_json::from_str::<ConfigsRegistry>(&text)
        .map_err(|err| format!("parse {REGISTRY_PATH} failed: {err}"))?;
    if registry.schema_version != 1 {
        return Err(format!("{REGISTRY_PATH} must declare schema_version=1"));
    }
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
    Ok(RegistryIndex {
        registry,
        root_files,
        group_files,
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

fn schema_index_json(repo_root: &Path) -> Result<serde_json::Value, String> {
    let schemas = read_schemas(repo_root)?;
    let input_schemas = walk_files_under(&repo_root.join("configs/schemas/registry"))
        .into_iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .filter_map(|path| {
            let rel = path
                .strip_prefix(repo_root)
                .ok()?
                .display()
                .to_string()
                .replace('\\', "/");
            if rel.starts_with("configs/schemas/registry/generated/") || !schema_like(&rel) {
                None
            } else {
                Some(rel)
            }
        })
        .collect::<BTreeSet<_>>();
    let output_schemas = walk_files_under(&repo_root.join("configs/schemas/contracts"))
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

fn read_text(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))
}

fn read_owners(repo_root: &Path) -> Result<ConfigsOwners, String> {
    let text = read_text(&repo_root.join(OWNERS_PATH))?;
    serde_json::from_str::<ConfigsOwners>(&text)
        .map_err(|err| format!("parse {OWNERS_PATH} failed: {err}"))
}

fn read_consumers(repo_root: &Path) -> Result<ConfigsConsumers, String> {
    let text = read_text(&repo_root.join(CONSUMERS_PATH))?;
    serde_json::from_str::<ConfigsConsumers>(&text)
        .map_err(|err| format!("parse {CONSUMERS_PATH} failed: {err}"))
}

fn read_schemas(repo_root: &Path) -> Result<ConfigsSchemas, String> {
    let text = read_text(&repo_root.join(SCHEMAS_PATH))?;
    serde_json::from_str::<ConfigsSchemas>(&text)
        .map_err(|err| format!("parse {SCHEMAS_PATH} failed: {err}"))
}

fn all_config_files(root: &Path) -> Result<Vec<String>, String> {
    fn walk(dir: &Path, repo_root: &Path, out: &mut Vec<String>) -> Result<(), String> {
        let entries =
            fs::read_dir(dir).map_err(|err| format!("read {} failed: {err}", dir.display()))?;
        let mut paths = entries
            .flatten()
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            if path.is_dir() {
                walk(&path, repo_root, out)?;
            } else if path.is_file() {
                let rel = path
                    .strip_prefix(repo_root)
                    .unwrap_or(&path)
                    .display()
                    .to_string()
                    .replace('\\', "/");
                out.push(rel);
            }
        }
        Ok(())
    }

    let mut out = Vec::new();
    walk(&root.join("configs"), root, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk_files_under(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(root) {
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

fn wildcard_match(pattern: &str, candidate: &str) -> bool {
    fn segment_match(pattern: &str, candidate: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        let p = pattern.as_bytes();
        let c = candidate.as_bytes();
        let mut pi = 0usize;
        let mut ci = 0usize;
        let mut star = None;
        let mut match_ci = 0usize;
        while ci < c.len() {
            if pi < p.len() && p[pi] == c[ci] {
                pi += 1;
                ci += 1;
            } else if pi < p.len() && p[pi] == b'*' {
                star = Some(pi);
                pi += 1;
                match_ci = ci;
            } else if let Some(star_idx) = star {
                pi = star_idx + 1;
                match_ci += 1;
                ci = match_ci;
            } else {
                return false;
            }
        }
        while pi < p.len() && p[pi] == b'*' {
            pi += 1;
        }
        pi == p.len()
    }

    fn match_segments(pattern: &[&str], candidate: &[&str]) -> bool {
        if pattern.is_empty() {
            return candidate.is_empty();
        }
        if pattern[0] == "**" {
            if match_segments(&pattern[1..], candidate) {
                return true;
            }
            if !candidate.is_empty() {
                return match_segments(pattern, &candidate[1..]);
            }
            return false;
        }
        if candidate.is_empty() {
            return false;
        }
        if !segment_match(pattern[0], candidate[0]) {
            return false;
        }
        match_segments(&pattern[1..], &candidate[1..])
    }

    let pattern_parts = pattern.split('/').collect::<Vec<_>>();
    let candidate_parts = candidate.split('/').collect::<Vec<_>>();
    match_segments(&pattern_parts, &candidate_parts)
}

fn matches_any<'a>(patterns: impl IntoIterator<Item = &'a String>, candidate: &str) -> bool {
    patterns
        .into_iter()
        .any(|pattern| wildcard_match(pattern, candidate))
}

fn matching_file_consumers(consumers: &ConfigsConsumers, candidate: &str) -> Vec<String> {
    let mut out = BTreeSet::new();
    for (pattern, entries) in &consumers.files {
        if wildcard_match(pattern, candidate) {
            out.extend(entries.iter().cloned());
        }
    }
    out.into_iter().collect()
}

fn matched_schema_path(schemas: &ConfigsSchemas, candidate: &str) -> Option<String> {
    schemas.files.iter().find_map(|(pattern, schema)| {
        if wildcard_match(pattern, candidate) {
            Some(schema.clone())
        } else {
            None
        }
    })
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
