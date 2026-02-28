// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

const REGISTRY_PATH: &str = "configs/inventory/configs.json";
const CONTRACT_SURFACE_PATH: &str = "configs/configs.contracts.json";
const OWNERS_PATH: &str = "configs/OWNERS.json";
const CONSUMERS_PATH: &str = "configs/CONSUMERS.json";
const SCHEMAS_PATH: &str = "configs/SCHEMAS.json";
const ROOT_CANONICAL_JSON_FILES: [&str; 6] = [
    "configs/OWNERS.json",
    "configs/CONSUMERS.json",
    "configs/SCHEMAS.json",
    "configs/configs.contracts.json",
    "configs/inventory.json",
    "configs/_generated/configs-index.json",
];
const ROOT_MARKDOWN_FILES: [&str; 2] = ["configs/README.md", "configs/CONTRACT.md"];
const DOCS_TOOLING_PATTERNS: [&str; 6] = [
    "configs/docs/.markdownlint-cli2.jsonc",
    "configs/docs/.vale.ini",
    "configs/docs/*.json",
    "configs/docs/*.jsonc",
    "configs/docs/*.txt",
    "configs/docs/.vale/styles/**",
];

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
}

#[derive(Clone, Deserialize)]
struct ConfigsOwners {
    schema_version: u64,
    groups: BTreeMap<String, String>,
}

#[derive(Clone, Deserialize)]
struct ConfigsConsumers {
    schema_version: u64,
    #[serde(default)]
    files: BTreeMap<String, Vec<String>>,
    groups: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Deserialize)]
struct ConfigsSchemas {
    schema_version: u64,
    files: BTreeMap<String, String>,
}

#[derive(Clone, Deserialize)]
struct ConfigsContractSurface {
    schema_version: u64,
    domain: String,
    contracts: Vec<ConfigsContractRow>,
}

#[derive(Clone, Deserialize)]
struct ConfigsContractRow {
    id: String,
    title: String,
    severity: String,
    contract_type: String,
    rationale: String,
    enforced_by: ConfigsContractEnforcement,
    touched_paths: Vec<String>,
    evidence_artifact: Option<String>,
}

#[derive(Clone, Deserialize)]
struct ConfigsContractEnforcement {
    command: String,
    test_id: String,
}

#[derive(Clone)]
struct RegistryIndex {
    registry: ConfigsRegistry,
    files: Vec<String>,
    excluded_files: BTreeSet<String>,
    root_files: BTreeSet<String>,
    group_files: BTreeMap<String, GroupFiles>,
    contract_surface_ids: BTreeSet<String>,
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

fn fail(contract_id: &str, test_id: &str, file: &str, message: impl Into<String>) -> TestResult {
    TestResult::Fail(vec![Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file: Some(file.to_string()),
        line: Some(1),
        message: message.into(),
        evidence: None,
    }])
}

fn violation(
    contract_id: &str,
    test_id: &str,
    file: &str,
    message: impl Into<String>,
) -> Violation {
    Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file: Some(file.to_string()),
        line: Some(1),
        message: message.into(),
        evidence: None,
    }
}

fn read_text(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))
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

fn read_contract_surface(repo_root: &Path) -> Result<ConfigsContractSurface, String> {
    let text = read_text(&repo_root.join(CONTRACT_SURFACE_PATH))?;
    serde_json::from_str::<ConfigsContractSurface>(&text)
        .map_err(|err| format!("parse {CONTRACT_SURFACE_PATH} failed: {err}"))
}

fn all_config_files(root: &Path) -> Result<Vec<String>, String> {
    fn walk(dir: &Path, repo_root: &Path, out: &mut Vec<String>) -> Result<(), String> {
        let entries = std::fs::read_dir(dir)
            .map_err(|err| format!("read {} failed: {err}", dir.display()))?;
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

fn path_depth(path: &str) -> usize {
    path.split('/').count().saturating_sub(1)
}

fn group_depth(path: &str, group: &str) -> Option<usize> {
    let prefix = format!("configs/{group}");
    if path == prefix {
        return Some(0);
    }
    let rest = path.strip_prefix(&(prefix + "/"))?;
    Some(rest.split('/').count().saturating_sub(1))
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
            "reason": item.reason
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
            if rel.starts_with("configs/schema/generated/") {
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

fn test_configs_001_root_surface(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-001",
                "configs.root.only_root_docs",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let actual = root_config_files(&index);
    let expected = index.root_files;
    if actual == expected {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "CONFIGS-001",
            "configs.root.only_root_docs",
            "configs",
            format!("expected root files {expected:?}, found {actual:?}"),
        )])
    }
}

fn test_configs_002_no_undocumented_files(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-002",
                "configs.registry.no_undocumented_files",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut covered = index.root_files.clone();
    for files in index.group_files.values() {
        covered.extend(files.all());
    }
    let missing = config_files_without_exclusions(&index)
        .into_iter()
        .filter(|file| !covered.contains(file))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(
            missing
                .into_iter()
                .map(|file| {
                    violation(
                        "CONFIGS-002",
                        "configs.registry.no_undocumented_files",
                        &file,
                        "config file is not covered by configs/inventory/configs.json",
                    )
                })
                .collect(),
        )
    }
}

fn test_configs_003_depth_budget(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-003",
                "configs.layout.depth_budget",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let violations = config_files_without_exclusions(&index)
        .into_iter()
        .filter(|file| path_depth(file) > index.registry.max_depth)
        .map(|file| {
            violation(
                "CONFIGS-003",
                "configs.layout.depth_budget",
                &file,
                format!(
                    "path depth {} exceeds configs max_depth {}",
                    path_depth(&file),
                    index.registry.max_depth
                ),
            )
        })
        .collect::<Vec<_>>();
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_004_internal_naming(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-004",
                "configs.naming.internal_surface",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        let files = index
            .group_files
            .get(&group.name)
            .cloned()
            .unwrap_or_default();
        for file in files.public.intersection(&files.internal) {
            violations.push(violation(
                "CONFIGS-004",
                "configs.naming.internal_surface",
                file,
                "a config file cannot be both public and internal",
            ));
        }
        for file in &files.public {
            let name = Path::new(file)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("");
            if name.starts_with('_') {
                violations.push(violation(
                    "CONFIGS-004",
                    "configs.naming.internal_surface",
                    file,
                    "internal-looking config file cannot be classified as public",
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_005_owner_complete(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-005",
                "configs.registry.owner_complete",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let violations = index
        .registry
        .groups
        .iter()
        .filter(|group| group.owner.trim().is_empty())
        .map(|group| {
            violation(
                "CONFIGS-005",
                "configs.registry.owner_complete",
                REGISTRY_PATH,
                format!("group `{}` is missing an owner", group.name),
            )
        })
        .collect::<Vec<_>>();
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_006_schema_coverage(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => return fail("CONFIGS-006", "configs.schema.coverage", REGISTRY_PATH, err),
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        let files = index
            .group_files
            .get(&group.name)
            .cloned()
            .unwrap_or_default();
        let has_json = files
            .all()
            .into_iter()
            .any(|file| json_like(&file) && !schema_like(&file));
        if has_json && group.schemas.is_empty() {
            violations.push(violation(
                "CONFIGS-006",
                "configs.schema.coverage",
                REGISTRY_PATH,
                format!(
                    "group `{}` contains json configs but declares no schemas",
                    group.name
                ),
            ));
        }
        for schema in &group.schemas {
            let exists = if schema.contains('*') {
                index.files.iter().any(|file| wildcard_match(schema, file))
            } else {
                ctx.repo_root.join(schema).is_file()
            };
            if !exists {
                violations.push(violation(
                    "CONFIGS-006",
                    "configs.schema.coverage",
                    schema,
                    format!("declared schema for group `{}` does not exist", group.name),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_007_lockfiles(ctx: &RunContext) -> TestResult {
    let required_pairs = [
        (
            "configs/docs/package.json",
            "configs/docs/package-lock.json",
        ),
        (
            "configs/docs/requirements.txt",
            "configs/docs/requirements.lock.txt",
        ),
    ];
    let mut violations = Vec::new();
    for (source, lockfile) in required_pairs {
        if ctx.repo_root.join(source).is_file() && !ctx.repo_root.join(lockfile).is_file() {
            violations.push(violation(
                "CONFIGS-007",
                "configs.lockfiles.required_pairs",
                lockfile,
                format!("lockfile is required when `{source}` exists"),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_008_no_overlap(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-008",
                "configs.registry.no_overlap",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut owners = BTreeMap::<String, Vec<String>>::new();
    for (group, files) in &index.group_files {
        for file in files.all() {
            owners.entry(file).or_default().push(group.clone());
        }
    }
    let violations = owners
        .into_iter()
        .filter(|(_, groups)| groups.len() > 1)
        .map(|(file, groups)| {
            violation(
                "CONFIGS-008",
                "configs.registry.no_overlap",
                &file,
                format!("file is claimed by multiple groups: {groups:?}"),
            )
        })
        .collect::<Vec<_>>();
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_009_generated_boundary(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-009",
                "configs.generated.authored_boundary",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        for pattern in &group.generated_files {
            if !pattern.contains("/_generated/")
                && !pattern.contains("/_generated.")
                && !pattern.contains("/schema/generated/")
            {
                violations.push(violation(
                    "CONFIGS-009",
                    "configs.generated.authored_boundary",
                    REGISTRY_PATH,
                    format!(
                        "generated pattern `{pattern}` for group `{}` must live under an _generated surface",
                        group.name
                    ),
                ));
            }
        }
        let files = index
            .group_files
            .get(&group.name)
            .cloned()
            .unwrap_or_default();
        for file in files.generated {
            if !file.contains("/_generated/")
                && !file.contains("/_generated.")
                && !file.contains("/schema/generated/")
            {
                violations.push(violation(
                    "CONFIGS-009",
                    "configs.generated.authored_boundary",
                    &file,
                    "generated configs must live under an _generated surface",
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_010_no_policy_theater(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let surface = match read_contract_surface(&ctx.repo_root) {
        Ok(surface) => surface,
        Err(err) => {
            return fail(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                CONTRACT_SURFACE_PATH,
                err,
            )
        }
    };
    if surface.schema_version != 1 {
        return fail(
            "CONFIGS-010",
            "configs.contracts.no_policy_theater",
            CONTRACT_SURFACE_PATH,
            format!(
                "unsupported configs contract registry schema_version {}",
                surface.schema_version
            ),
        );
    }
    if surface.domain != "configs" {
        return fail(
            "CONFIGS-010",
            "configs.contracts.no_policy_theater",
            CONTRACT_SURFACE_PATH,
            format!(
                "configs contract registry must declare domain `configs`, found `{}`",
                surface.domain
            ),
        );
    }
    let expected = (1..=37)
        .map(|n| format!("CFG-{n:03}"))
        .collect::<BTreeSet<_>>();
    let actual = surface
        .contracts
        .iter()
        .map(|row| row.id.clone())
        .collect::<BTreeSet<_>>();
    let executable_tests = contracts(&ctx.repo_root)
        .unwrap_or_default()
        .into_iter()
        .flat_map(|contract| contract.tests.into_iter().map(|test| test.id.0))
        .collect::<BTreeSet<_>>();
    let allowed_severities = ["blocker", "must", "should"];
    let allowed_types = ["static", "filelayout", "schema", "drift", "supplychain"];
    let mut violations = Vec::new();
    if actual != expected {
        violations.push(violation(
            "CONFIGS-010",
            "configs.contracts.no_policy_theater",
            CONTRACT_SURFACE_PATH,
            format!(
                "expected configs contract ids {:?}, found {:?}",
                expected, actual
            ),
        ));
    }
    if index.contract_surface_ids != actual {
        violations.push(violation(
            "CONFIGS-010",
            "configs.contracts.no_policy_theater",
            "configs/CONTRACT.md",
            format!(
                "contract markdown ids {:?} do not match configs contract registry ids {:?}",
                index.contract_surface_ids, actual
            ),
        ));
    }
    for row in &surface.contracts {
        if row.title.trim().is_empty() {
            violations.push(violation(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                CONTRACT_SURFACE_PATH,
                format!("contract `{}` is missing a title", row.id),
            ));
        }
        if !allowed_severities.contains(&row.severity.as_str()) {
            violations.push(violation(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                CONTRACT_SURFACE_PATH,
                format!(
                    "contract `{}` uses unsupported severity `{}`",
                    row.id, row.severity
                ),
            ));
        }
        if !allowed_types.contains(&row.contract_type.as_str()) {
            violations.push(violation(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                CONTRACT_SURFACE_PATH,
                format!(
                    "contract `{}` uses unsupported contract_type `{}`",
                    row.id, row.contract_type
                ),
            ));
        }
        if row.rationale.trim().is_empty() {
            violations.push(violation(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                CONTRACT_SURFACE_PATH,
                format!("contract `{}` is missing a rationale", row.id),
            ));
        }
        if row.enforced_by.command != "bijux dev atlas contracts configs" {
            violations.push(violation(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                CONTRACT_SURFACE_PATH,
                format!(
                    "contract `{}` must use `bijux dev atlas contracts configs` as its enforcement command",
                    row.id
                ),
            ));
        }
        if !executable_tests.contains(&row.enforced_by.test_id) {
            violations.push(violation(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                CONTRACT_SURFACE_PATH,
                format!(
                    "contract `{}` references unknown enforcement test `{}`",
                    row.id, row.enforced_by.test_id
                ),
            ));
        }
        if row.touched_paths.is_empty() {
            violations.push(violation(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                CONTRACT_SURFACE_PATH,
                format!("contract `{}` must declare touched_paths", row.id),
            ));
        }
        for path in &row.touched_paths {
            if !path.starts_with("configs/") && !path.starts_with("artifacts/") {
                violations.push(violation(
                    "CONFIGS-010",
                    "configs.contracts.no_policy_theater",
                    CONTRACT_SURFACE_PATH,
                    format!(
                        "contract `{}` has touched path `{path}` outside configs or artifacts",
                        row.id
                    ),
                ));
            }
        }
        if let Some(artifact) = &row.evidence_artifact {
            if !artifact.starts_with("artifacts/") {
                violations.push(violation(
                    "CONFIGS-010",
                    "configs.contracts.no_policy_theater",
                    CONTRACT_SURFACE_PATH,
                    format!(
                        "contract `{}` has evidence_artifact `{artifact}` outside artifacts/",
                        row.id
                    ),
                ));
            }
        }
    }
    let mapped_enforcement_checks = surface
        .contracts
        .iter()
        .map(|row| row.enforced_by.test_id.clone())
        .collect::<BTreeSet<_>>();
    let unmapped_executable_checks = executable_tests
        .difference(&mapped_enforcement_checks)
        .cloned()
        .collect::<Vec<_>>();
    if !unmapped_executable_checks.is_empty() {
        violations.push(violation(
            "CONFIGS-010",
            "configs.contracts.no_policy_theater",
            CONTRACT_SURFACE_PATH,
            format!(
                "configs contract registry leaves executable checks unmapped: {:?}",
                unmapped_executable_checks
            ),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_011_registry_complete(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-011",
                "configs.registry.complete_surface",
                REGISTRY_PATH,
                err,
            )
        }
    };
    if index.registry.schema_version != 1 {
        return fail(
            "CONFIGS-011",
            "configs.registry.complete_surface",
            REGISTRY_PATH,
            format!(
                "unsupported configs registry schema_version {}",
                index.registry.schema_version
            ),
        );
    }
    let has_root_readme = index.root_files.contains("configs/README.md");
    let has_root_contract = index.root_files.contains("configs/CONTRACT.md");
    if has_root_readme && has_root_contract {
        TestResult::Pass
    } else {
        fail(
            "CONFIGS-011",
            "configs.registry.complete_surface",
            REGISTRY_PATH,
            "configs registry root_files must include configs/README.md and configs/CONTRACT.md",
        )
    }
}

fn test_configs_012_no_orphans(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-012",
                "configs.registry.no_orphans",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut covered = index.root_files.clone();
    for files in index.group_files.values() {
        covered.extend(files.all());
    }
    let orphans = config_files_without_exclusions(&index)
        .into_iter()
        .filter(|file| !covered.contains(file))
        .collect::<Vec<_>>();
    if orphans.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(
            orphans
                .into_iter()
                .map(|file| {
                    violation(
                        "CONFIGS-012",
                        "configs.registry.no_orphans",
                        &file,
                        "config file is orphaned from the configs registry",
                    )
                })
                .collect(),
        )
    }
}

fn test_configs_013_no_dead_entries(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-013",
                "configs.registry.no_dead_entries",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        let files = index
            .group_files
            .get(&group.name)
            .cloned()
            .unwrap_or_default();
        if !group.public_files.is_empty() && files.public.is_empty() {
            violations.push(violation(
                "CONFIGS-013",
                "configs.registry.no_dead_entries",
                REGISTRY_PATH,
                format!(
                    "group `{}` has public patterns with no matching files",
                    group.name
                ),
            ));
        }
        if !group.internal_files.is_empty() && files.internal.is_empty() {
            violations.push(violation(
                "CONFIGS-013",
                "configs.registry.no_dead_entries",
                REGISTRY_PATH,
                format!(
                    "group `{}` has internal patterns with no matching files",
                    group.name
                ),
            ));
        }
    }
    for item in &index.registry.exclusions {
        let matched = index
            .files
            .iter()
            .any(|file| wildcard_match(&item.pattern, file));
        if !matched {
            violations.push(violation(
                "CONFIGS-013",
                "configs.registry.no_dead_entries",
                REGISTRY_PATH,
                format!(
                    "exclusion `{}` has no matching files ({})",
                    item.pattern, item.reason
                ),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_014_group_budget(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-014",
                "configs.registry.group_budget",
                REGISTRY_PATH,
                err,
            )
        }
    };
    if index.registry.groups.len() <= index.registry.max_groups {
        TestResult::Pass
    } else {
        fail(
            "CONFIGS-014",
            "configs.registry.group_budget",
            REGISTRY_PATH,
            format!(
                "configs registry declares {} groups, which exceeds max_groups {}",
                index.registry.groups.len(),
                index.registry.max_groups
            ),
        )
    }
}

fn test_configs_015_group_depth_budget(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-015",
                "configs.registry.group_depth_budget",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        let files = index
            .group_files
            .get(&group.name)
            .cloned()
            .unwrap_or_default();
        for file in files.all() {
            if let Some(depth) = group_depth(&file, &group.name) {
                if depth > index.registry.max_group_depth {
                    violations.push(violation(
                        "CONFIGS-015",
                        "configs.registry.group_depth_budget",
                        &file,
                        format!(
                            "group depth {} exceeds max_group_depth {} for `{}`",
                            depth, index.registry.max_group_depth, group.name
                        ),
                    ));
                }
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_016_visibility_classification(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-016",
                "configs.registry.visibility_classification",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        if group.stability != "stable" && group.stability != "experimental" {
            violations.push(violation(
                "CONFIGS-016",
                "configs.registry.visibility_classification",
                REGISTRY_PATH,
                format!(
                    "group `{}` has invalid stability `{}`",
                    group.name, group.stability
                ),
            ));
        }
        let files = index
            .group_files
            .get(&group.name)
            .cloned()
            .unwrap_or_default();
        for file in files.all() {
            let classifications = usize::from(files.public.contains(&file))
                + usize::from(files.internal.contains(&file))
                + usize::from(files.generated.contains(&file));
            if classifications != 1 {
                violations.push(violation(
                    "CONFIGS-016",
                    "configs.registry.visibility_classification",
                    &file,
                    "each config file must map to exactly one visibility bucket",
                ));
            }
        }
        if group.public_files.is_empty()
            && group.internal_files.is_empty()
            && group.generated_files.is_empty()
        {
            violations.push(violation(
                "CONFIGS-016",
                "configs.registry.visibility_classification",
                REGISTRY_PATH,
                format!("group `{}` declares no file buckets", group.name),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_017_tool_entrypoints(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-017",
                "configs.registry.tool_entrypoints",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        if group.tool_entrypoints.is_empty() {
            violations.push(violation(
                "CONFIGS-017",
                "configs.registry.tool_entrypoints",
                REGISTRY_PATH,
                format!(
                    "group `{}` must declare at least one tool entrypoint",
                    group.name
                ),
            ));
        }
        for entrypoint in &group.tool_entrypoints {
            if !entrypoint.starts_with("bijux ")
                && !entrypoint.starts_with("make ")
                && !entrypoint.starts_with("cargo ")
            {
                violations.push(violation(
                    "CONFIGS-017",
                    "configs.registry.tool_entrypoints",
                    REGISTRY_PATH,
                    format!(
                        "group `{}` has unsupported tool entrypoint `{entrypoint}`",
                        group.name
                    ),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_018_schema_owner(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-018",
                "configs.registry.schema_owner",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        if group.schema_owner.trim().is_empty() {
            violations.push(violation(
                "CONFIGS-018",
                "configs.registry.schema_owner",
                REGISTRY_PATH,
                format!("group `{}` is missing schema_owner", group.name),
            ));
        }
        for schema in &group.schemas {
            let exists = if schema.contains('*') {
                index.files.iter().any(|file| wildcard_match(schema, file))
            } else {
                ctx.repo_root.join(schema).is_file()
            };
            if !exists {
                violations.push(violation(
                    "CONFIGS-018",
                    "configs.registry.schema_owner",
                    schema,
                    format!(
                        "schema owner `{}` declares missing schema for group `{}`",
                        group.schema_owner, group.name
                    ),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_019_lifecycle(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-019",
                "configs.registry.lifecycle",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        if group.stability != "stable" && group.stability != "experimental" {
            violations.push(violation(
                "CONFIGS-019",
                "configs.registry.lifecycle",
                REGISTRY_PATH,
                format!(
                    "group `{}` has invalid stability `{}`",
                    group.name, group.stability
                ),
            ));
        }
        if group.owner.trim().is_empty() || group.schema_owner.trim().is_empty() {
            violations.push(violation(
                "CONFIGS-019",
                "configs.registry.lifecycle",
                REGISTRY_PATH,
                format!("group `{}` lifecycle metadata is incomplete", group.name),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_020_generated_index_deterministic(ctx: &RunContext) -> TestResult {
    let first = match generated_index_json(&ctx.repo_root) {
        Ok(value) => value,
        Err(err) => {
            return fail(
                "CONFIGS-020",
                "configs.generated_index.deterministic",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let second = match generated_index_json(&ctx.repo_root) {
        Ok(value) => value,
        Err(err) => {
            return fail(
                "CONFIGS-020",
                "configs.generated_index.deterministic",
                REGISTRY_PATH,
                err,
            )
        }
    };
    if first == second {
        TestResult::Pass
    } else {
        fail(
            "CONFIGS-020",
            "configs.generated_index.deterministic",
            "configs/_generated/configs-index.json",
            "generated configs index is not deterministic across consecutive renders",
        )
    }
}

fn test_configs_021_generated_index_matches_committed(ctx: &RunContext) -> TestResult {
    let expected = match generated_index_json(&ctx.repo_root) {
        Ok(value) => value,
        Err(err) => {
            return fail(
                "CONFIGS-021",
                "configs.generated_index.committed_match",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let path = ctx.repo_root.join("configs/_generated/configs-index.json");
    let text = match read_text(&path) {
        Ok(text) => text,
        Err(err) => {
            return fail(
                "CONFIGS-021",
                "configs.generated_index.committed_match",
                "configs/_generated/configs-index.json",
                err,
            )
        }
    };
    let actual = match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(value) => value,
        Err(err) => {
            return fail(
                "CONFIGS-021",
                "configs.generated_index.committed_match",
                "configs/_generated/configs-index.json",
                format!("parse configs/_generated/configs-index.json failed: {err}"),
            )
        }
    };
    if actual == expected {
        TestResult::Pass
    } else {
        fail(
            "CONFIGS-021",
            "configs.generated_index.committed_match",
            "configs/_generated/configs-index.json",
            "committed generated configs index does not match registry render",
        )
    }
}

fn test_configs_022_json_configs_parse(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => return fail("CONFIGS-022", "configs.parse.json", REGISTRY_PATH, err),
    };
    let violations = parse_checked_files(&index, &["json", "jsonc"])
        .into_iter()
        .filter_map(|file| {
            parse_supported_config_file(&ctx.repo_root.join(&file))
                .err()
                .map(|err| violation("CONFIGS-022", "configs.parse.json", &file, err))
        })
        .collect::<Vec<_>>();
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_023_yaml_configs_parse(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => return fail("CONFIGS-023", "configs.parse.yaml", REGISTRY_PATH, err),
    };
    let violations = parse_checked_files(&index, &["yaml", "yml"])
        .into_iter()
        .filter_map(|file| {
            parse_supported_config_file(&ctx.repo_root.join(&file))
                .err()
                .map(|err| violation("CONFIGS-023", "configs.parse.yaml", &file, err))
        })
        .collect::<Vec<_>>();
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_024_toml_configs_parse(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => return fail("CONFIGS-024", "configs.parse.toml", REGISTRY_PATH, err),
    };
    let violations = parse_checked_files(&index, &["toml"])
        .into_iter()
        .filter_map(|file| {
            parse_supported_config_file(&ctx.repo_root.join(&file))
                .err()
                .map(|err| violation("CONFIGS-024", "configs.parse.toml", &file, err))
        })
        .collect::<Vec<_>>();
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_025_text_hygiene(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => return fail("CONFIGS-025", "configs.text.hygiene", REGISTRY_PATH, err),
    };
    let text_exts = ["md", "txt", "toml", "json", "jsonc", "yml", "yaml", "ini"];
    let mut violations = Vec::new();
    for file in config_files_without_exclusions(&index) {
        let ext = Path::new(&file)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if !text_exts.contains(&ext) {
            continue;
        }
        let text = match read_text(&ctx.repo_root.join(&file)) {
            Ok(text) => text,
            Err(err) => {
                violations.push(violation("CONFIGS-025", "configs.text.hygiene", &file, err));
                continue;
            }
        };
        for (line_no, line) in text.lines().enumerate() {
            if line.ends_with(' ') || line.ends_with('\t') {
                violations.push(Violation {
                    contract_id: "CONFIGS-025".to_string(),
                    test_id: "configs.text.hygiene".to_string(),
                    file: Some(file.clone()),
                    line: Some(line_no + 1),
                    message: "trailing whitespace is forbidden in config text files".to_string(),
                    evidence: None,
                });
                break;
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_026_docs_markdown_removed(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-026",
                "configs.docs.no_nested_markdown",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let violations = config_files_without_exclusions(&index)
        .into_iter()
        .filter(|file| file.ends_with(".md"))
        .filter(|file| !ROOT_MARKDOWN_FILES.contains(&file.as_str()))
        .map(|file| {
            violation(
                "CONFIGS-026",
                "configs.docs.no_nested_markdown",
                &file,
                "configs keeps markdown only at the root authority surface; move narrative markdown into docs/",
            )
        })
        .collect::<Vec<_>>();
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_027_docs_tooling_surface(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-027",
                "configs.docs.tooling_surface",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for file in config_files_without_exclusions(&index) {
        if !file.starts_with("configs/docs/") {
            continue;
        }
        if !DOCS_TOOLING_PATTERNS
            .iter()
            .any(|pattern| wildcard_match(pattern, &file))
        {
            violations.push(violation(
                "CONFIGS-027",
                "configs.docs.tooling_surface",
                &file,
                "configs/docs contains a file outside the declared tooling surface",
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_028_owner_map_alignment(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-028",
                "configs.owners.group_alignment",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let owners = match read_owners(&ctx.repo_root) {
        Ok(owners) => owners,
        Err(err) => {
            return fail(
                "CONFIGS-028",
                "configs.owners.group_alignment",
                OWNERS_PATH,
                err,
            )
        }
    };
    if owners.schema_version != 1 {
        return fail(
            "CONFIGS-028",
            "configs.owners.group_alignment",
            OWNERS_PATH,
            format!(
                "unsupported configs owner map schema_version {}",
                owners.schema_version
            ),
        );
    }
    let mut violations = Vec::new();
    let expected_groups = index
        .registry
        .groups
        .iter()
        .map(|group| group.name.clone())
        .collect::<BTreeSet<_>>();
    let actual_groups = owners.groups.keys().cloned().collect::<BTreeSet<_>>();
    for missing in expected_groups.difference(&actual_groups) {
        violations.push(violation(
            "CONFIGS-028",
            "configs.owners.group_alignment",
            OWNERS_PATH,
            format!("owner map is missing group `{missing}`"),
        ));
    }
    for extra in actual_groups.difference(&expected_groups) {
        violations.push(violation(
            "CONFIGS-028",
            "configs.owners.group_alignment",
            OWNERS_PATH,
            format!("owner map declares unknown group `{extra}`"),
        ));
    }
    for group in &index.registry.groups {
        match owners.groups.get(&group.name) {
            Some(owner) if owner == &group.owner => {}
            Some(owner) => violations.push(violation(
                "CONFIGS-028",
                "configs.owners.group_alignment",
                OWNERS_PATH,
                format!(
                    "owner map mismatch for `{}`: expected `{}`, found `{owner}`",
                    group.name, group.owner
                ),
            )),
            None => {}
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_029_consumer_map_alignment(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-029",
                "configs.consumers.group_alignment",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let consumers = match read_consumers(&ctx.repo_root) {
        Ok(consumers) => consumers,
        Err(err) => {
            return fail(
                "CONFIGS-029",
                "configs.consumers.group_alignment",
                CONSUMERS_PATH,
                err,
            )
        }
    };
    if consumers.schema_version != 1 {
        return fail(
            "CONFIGS-029",
            "configs.consumers.group_alignment",
            CONSUMERS_PATH,
            format!(
                "unsupported configs consumer map schema_version {}",
                consumers.schema_version
            ),
        );
    }
    let mut violations = Vec::new();
    let expected_groups = index
        .registry
        .groups
        .iter()
        .map(|group| group.name.clone())
        .collect::<BTreeSet<_>>();
    let actual_groups = consumers.groups.keys().cloned().collect::<BTreeSet<_>>();
    for missing in expected_groups.difference(&actual_groups) {
        violations.push(violation(
            "CONFIGS-029",
            "configs.consumers.group_alignment",
            CONSUMERS_PATH,
            format!("consumer map is missing group `{missing}`"),
        ));
    }
    for extra in actual_groups.difference(&expected_groups) {
        violations.push(violation(
            "CONFIGS-029",
            "configs.consumers.group_alignment",
            CONSUMERS_PATH,
            format!("consumer map declares unknown group `{extra}`"),
        ));
    }
    for group in &index.registry.groups {
        if let Some(entries) = consumers.groups.get(&group.name) {
            if entries.is_empty() {
                violations.push(violation(
                    "CONFIGS-029",
                    "configs.consumers.group_alignment",
                    CONSUMERS_PATH,
                    format!(
                        "consumer map for `{}` must list at least one consumer",
                        group.name
                    ),
                ));
            }
        }
    }
    for (pattern, entries) in &consumers.files {
        let matches_public_file = index
            .root_files
            .iter()
            .any(|file| wildcard_match(pattern, file))
            || index.registry.groups.iter().any(|group| {
                let files = index
                    .group_files
                    .get(&group.name)
                    .cloned()
                    .unwrap_or_default();
                files
                    .public
                    .iter()
                    .chain(files.generated.iter())
                    .any(|file| wildcard_match(pattern, file))
            });
        if !matches_public_file {
            violations.push(violation(
                "CONFIGS-029",
                "configs.consumers.group_alignment",
                CONSUMERS_PATH,
                format!(
                    "consumer file pattern `{pattern}` does not match any public or generated config file"
                ),
            ));
        }
        if entries.is_empty() {
            violations.push(violation(
                "CONFIGS-029",
                "configs.consumers.group_alignment",
                CONSUMERS_PATH,
                format!("consumer file pattern `{pattern}` must list at least one consumer"),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_030_file_consumer_coverage(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-030",
                "configs.consumers.file_coverage",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let consumers = match read_consumers(&ctx.repo_root) {
        Ok(consumers) => consumers,
        Err(err) => {
            return fail(
                "CONFIGS-030",
                "configs.consumers.file_coverage",
                CONSUMERS_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        let files = index
            .group_files
            .get(&group.name)
            .cloned()
            .unwrap_or_default();
        for file in files.public.iter().chain(files.generated.iter()) {
            let matched = matching_file_consumers(&consumers, file);
            if matched.is_empty() {
                violations.push(violation(
                    "CONFIGS-030",
                    "configs.consumers.file_coverage",
                    file,
                    "public or generated config file is missing a per-file consumer declaration",
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_031_schema_map_coverage(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-031",
                "configs.schemas.file_coverage",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let schemas = match read_schemas(&ctx.repo_root) {
        Ok(schemas) => schemas,
        Err(err) => {
            return fail(
                "CONFIGS-031",
                "configs.schemas.file_coverage",
                SCHEMAS_PATH,
                err,
            )
        }
    };
    if schemas.schema_version != 1 {
        return fail(
            "CONFIGS-031",
            "configs.schemas.file_coverage",
            SCHEMAS_PATH,
            format!(
                "unsupported configs schema map schema_version {}",
                schemas.schema_version
            ),
        );
    }
    let mut violations = Vec::new();
    for (pattern, schema) in &schemas.files {
        let matches_file = index.registry.groups.iter().any(|group| {
            let files = index
                .group_files
                .get(&group.name)
                .cloned()
                .unwrap_or_default();
            files
                .public
                .iter()
                .chain(files.generated.iter())
                .filter(|file| json_like(file) && !schema_like(file))
                .any(|file| wildcard_match(pattern, file))
        }) || index
            .root_files
            .iter()
            .filter(|file| json_like(file) && !schema_like(file))
            .any(|file| wildcard_match(pattern, file));
        if !matches_file {
            violations.push(violation(
                "CONFIGS-031",
                "configs.schemas.file_coverage",
                SCHEMAS_PATH,
                format!(
                    "schema file pattern `{pattern}` does not match any governed json or jsonc config file"
                ),
            ));
        }
        if !ctx.repo_root.join(schema).is_file() {
            violations.push(violation(
                "CONFIGS-031",
                "configs.schemas.file_coverage",
                schema,
                format!("declared schema map target for `{pattern}` does not exist"),
            ));
        }
    }
    for file in index
        .root_files
        .iter()
        .filter(|file| json_like(file) && !schema_like(file))
    {
        if matched_schema_path(&schemas, file).is_none() {
            violations.push(violation(
                "CONFIGS-031",
                "configs.schemas.file_coverage",
                file,
                "root json or jsonc config file is missing a per-file schema declaration",
            ));
        }
    }
    for group in &index.registry.groups {
        let files = index
            .group_files
            .get(&group.name)
            .cloned()
            .unwrap_or_default();
        for file in files.public.iter().chain(files.generated.iter()) {
            if !json_like(file) || schema_like(file) {
                continue;
            }
            if matched_schema_path(&schemas, file).is_none() {
                violations.push(violation(
                    "CONFIGS-031",
                    "configs.schemas.file_coverage",
                    file,
                    "public or generated json or jsonc config file is missing a per-file schema declaration",
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_032_root_json_canonical(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for file in ROOT_CANONICAL_JSON_FILES {
        let path = ctx.repo_root.join(file);
        let text = match read_text(&path) {
            Ok(text) => text,
            Err(err) => {
                violations.push(violation(
                    "CONFIGS-032",
                    "configs.json.canonical_root_surface",
                    file,
                    err,
                ));
                continue;
            }
        };
        let value = match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(value) => value,
            Err(err) => {
                violations.push(violation(
                    "CONFIGS-032",
                    "configs.json.canonical_root_surface",
                    file,
                    format!("parse failed: {err}"),
                ));
                continue;
            }
        };
        let canonical = match canonical_json_string(&value) {
            Ok(text) => text,
            Err(err) => {
                violations.push(violation(
                    "CONFIGS-032",
                    "configs.json.canonical_root_surface",
                    file,
                    err,
                ));
                continue;
            }
        };
        if text != canonical {
            violations.push(violation(
                "CONFIGS-032",
                "configs.json.canonical_root_surface",
                file,
                "json must use stable two-space pretty formatting with lexicographically sorted object keys",
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_033_schema_index_matches_committed(ctx: &RunContext) -> TestResult {
    let expected = match schema_index_json(&ctx.repo_root) {
        Ok(value) => value,
        Err(err) => {
            return fail(
                "CONFIGS-033",
                "configs.schema.index_committed_match",
                SCHEMAS_PATH,
                err,
            )
        }
    };
    let path = ctx
        .repo_root
        .join("configs/schema/generated/schema-index.json");
    let text = match read_text(&path) {
        Ok(text) => text,
        Err(err) => {
            return fail(
                "CONFIGS-033",
                "configs.schema.index_committed_match",
                "configs/schema/generated/schema-index.json",
                err,
            )
        }
    };
    let actual = match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(value) => value,
        Err(err) => {
            return fail(
                "CONFIGS-033",
                "configs.schema.index_committed_match",
                "configs/schema/generated/schema-index.json",
                format!("parse configs/schema/generated/schema-index.json failed: {err}"),
            )
        }
    };
    if actual == expected {
        TestResult::Pass
    } else {
        fail(
            "CONFIGS-033",
            "configs.schema.index_committed_match",
            "configs/schema/generated/schema-index.json",
            "committed schema index does not match the canonical schema map render",
        )
    }
}

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![
        contract(
            "CONFIGS-001",
            "configs root keeps only declared root files",
            "configs.root.only_root_docs",
            "configs root file surface matches registry",
            test_configs_001_root_surface,
        ),
        contract(
            "CONFIGS-002",
            "configs files are documented by the registry",
            "configs.registry.no_undocumented_files",
            "registry covers every config file",
            test_configs_002_no_undocumented_files,
        ),
        contract(
            "CONFIGS-003",
            "configs path depth stays within budget",
            "configs.layout.depth_budget",
            "configs depth budget stays within registry limits",
            test_configs_003_depth_budget,
        ),
        contract(
            "CONFIGS-004",
            "configs internal surfaces stay explicitly internal",
            "configs.naming.internal_surface",
            "internal configs are not exposed as public",
            test_configs_004_internal_naming,
        ),
        contract(
            "CONFIGS-005",
            "configs groups declare owners",
            "configs.registry.owner_complete",
            "each configs group has an owner",
            test_configs_005_owner_complete,
        ),
        contract(
            "CONFIGS-006",
            "configs groups declare schema coverage",
            "configs.schema.coverage",
            "json-bearing groups declare real schema files",
            test_configs_006_schema_coverage,
        ),
        contract(
            "CONFIGS-007",
            "configs lockfile pairs stay complete",
            "configs.lockfiles.required_pairs",
            "tool dependency configs keep lockfiles",
            test_configs_007_lockfiles,
        ),
        contract(
            "CONFIGS-008",
            "configs registry avoids duplicate group ownership",
            "configs.registry.no_overlap",
            "no config file is claimed by multiple groups",
            test_configs_008_no_overlap,
        ),
        contract(
            "CONFIGS-009",
            "generated config surfaces stay separate from authored files",
            "configs.generated.authored_boundary",
            "generated config patterns stay under _generated surfaces",
            test_configs_009_generated_boundary,
        ),
        contract(
            "CONFIGS-010",
            "configs contracts doc mirrors executable checks",
            "configs.contracts.no_policy_theater",
            "contract docs match enforced config checks",
            test_configs_010_no_policy_theater,
        ),
        contract(
            "CONFIGS-011",
            "configs registry keeps a complete root surface",
            "configs.registry.complete_surface",
            "registry keeps the root docs and manifest visible",
            test_configs_011_registry_complete,
        ),
        contract(
            "CONFIGS-012",
            "configs registry leaves no orphan files",
            "configs.registry.no_orphans",
            "all config files belong to the registry",
            test_configs_012_no_orphans,
        ),
        contract(
            "CONFIGS-013",
            "configs registry leaves no dead entries",
            "configs.registry.no_dead_entries",
            "all registry patterns and exclusions match real files",
            test_configs_013_no_dead_entries,
        ),
        contract(
            "CONFIGS-014",
            "configs group count stays within budget",
            "configs.registry.group_budget",
            "configs group count stays under max_groups",
            test_configs_014_group_budget,
        ),
        contract(
            "CONFIGS-015",
            "configs group paths stay within group depth budget",
            "configs.registry.group_depth_budget",
            "config files do not exceed per-group depth budget",
            test_configs_015_group_depth_budget,
        ),
        contract(
            "CONFIGS-016",
            "configs files declare exactly one visibility class",
            "configs.registry.visibility_classification",
            "each config file maps to public, internal, or generated",
            test_configs_016_visibility_classification,
        ),
        contract(
            "CONFIGS-017",
            "configs groups declare tool entrypoints",
            "configs.registry.tool_entrypoints",
            "each configs group declares consuming command entrypoints",
            test_configs_017_tool_entrypoints,
        ),
        contract(
            "CONFIGS-018",
            "configs groups declare schema ownership",
            "configs.registry.schema_owner",
            "schema files map to an explicit schema owner",
            test_configs_018_schema_owner,
        ),
        contract(
            "CONFIGS-019",
            "configs groups declare lifecycle metadata",
            "configs.registry.lifecycle",
            "each configs group declares stability-tier lifecycle metadata",
            test_configs_019_lifecycle,
        ),
        contract(
            "CONFIGS-020",
            "configs generated index stays deterministic",
            "configs.generated_index.deterministic",
            "generated configs index renders deterministically",
            test_configs_020_generated_index_deterministic,
        ),
        contract(
            "CONFIGS-021",
            "configs generated index matches committed output",
            "configs.generated_index.committed_match",
            "committed generated configs index matches the registry render",
            test_configs_021_generated_index_matches_committed,
        ),
        contract(
            "CONFIGS-022",
            "configs json surfaces parse cleanly",
            "configs.parse.json",
            "json and jsonc config files parse successfully",
            test_configs_022_json_configs_parse,
        ),
        contract(
            "CONFIGS-023",
            "configs yaml surfaces parse cleanly",
            "configs.parse.yaml",
            "yaml config files parse successfully",
            test_configs_023_yaml_configs_parse,
        ),
        contract(
            "CONFIGS-024",
            "configs toml surfaces parse cleanly",
            "configs.parse.toml",
            "toml config files parse successfully",
            test_configs_024_toml_configs_parse,
        ),
        contract(
            "CONFIGS-025",
            "configs text surfaces avoid whitespace drift",
            "configs.text.hygiene",
            "text config files avoid trailing whitespace drift",
            test_configs_025_text_hygiene,
        ),
        contract(
            "CONFIGS-026",
            "configs docs directory forbids nested markdown",
            "configs.docs.no_nested_markdown",
            "configs docs keeps tooling inputs only",
            test_configs_026_docs_markdown_removed,
        ),
        contract(
            "CONFIGS-027",
            "configs docs directory stays tooling only",
            "configs.docs.tooling_surface",
            "configs docs files stay within the declared tooling surface",
            test_configs_027_docs_tooling_surface,
        ),
        contract(
            "CONFIGS-028",
            "configs owner map stays aligned with the registry",
            "configs.owners.group_alignment",
            "configs owner map matches the declared group owners",
            test_configs_028_owner_map_alignment,
        ),
        contract(
            "CONFIGS-029",
            "configs consumer map stays aligned with the registry",
            "configs.consumers.group_alignment",
            "configs consumer map matches the declared groups",
            test_configs_029_consumer_map_alignment,
        ),
        contract(
            "CONFIGS-030",
            "configs public files declare file-level consumers",
            "configs.consumers.file_coverage",
            "public and generated config files have per-file consumer coverage",
            test_configs_030_file_consumer_coverage,
        ),
        contract(
            "CONFIGS-031",
            "configs json files declare file-level schema coverage",
            "configs.schemas.file_coverage",
            "root, public, and generated json configs map to declared schemas",
            test_configs_031_schema_map_coverage,
        ),
        contract(
            "CONFIGS-032",
            "configs root json surfaces stay canonical",
            "configs.json.canonical_root_surface",
            "root authority json files use canonical stable formatting",
            test_configs_032_root_json_canonical,
        ),
        contract(
            "CONFIGS-033",
            "configs schema index matches committed output",
            "configs.schema.index_committed_match",
            "committed schema index matches the canonical schema map render",
            test_configs_033_schema_index_matches_committed,
        ),
    ])
}

fn contract(
    id: &'static str,
    title: &'static str,
    test_id: &'static str,
    test_title: &'static str,
    run: fn(&RunContext) -> TestResult,
) -> Contract {
    Contract {
        id: ContractId(id.to_string()),
        title,
        tests: vec![TestCase {
            id: TestId(test_id.to_string()),
            title: test_title,
            kind: TestKind::Pure,
            run,
        }],
    }
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "CONFIGS-001" => "The configs root is a tiny authority surface: README.md, CONTRACT.md, and the legacy inventory pointer only.".to_string(),
        "CONFIGS-002" => "Every config file must be covered by the canonical configs registry so filesystem drift is visible.".to_string(),
        "CONFIGS-003" => "Configs path depth stays within an explicit budget to avoid unreviewable nesting.".to_string(),
        "CONFIGS-004" => "Internal config surfaces must stay internal and cannot leak into public classifications.".to_string(),
        "CONFIGS-005" => "Every configs group needs an explicit owner in the registry.".to_string(),
        "CONFIGS-006" => "JSON-bearing config groups must declare real schema files so validation has a source of truth.".to_string(),
        "CONFIGS-007" => "Pinned tool dependency manifests require committed lockfiles.".to_string(),
        "CONFIGS-008" => "A config file can only have one group owner in the registry.".to_string(),
        "CONFIGS-009" => "Generated configs stay under explicit _generated surfaces instead of mixing with authored files.".to_string(),
        "CONFIGS-010" => "Configs contracts docs must match the executable checks; documentation alone is not evidence.".to_string(),
        "CONFIGS-011" => "The configs registry must describe the root surface completely and deterministically.".to_string(),
        "CONFIGS-012" => "No config file may exist outside the registry.".to_string(),
        "CONFIGS-013" => "Registry patterns and exclusions must resolve to real files, not stale entries.".to_string(),
        "CONFIGS-014" => "Configs groups stay within an explicit group-count budget.".to_string(),
        "CONFIGS-015" => "Each configs group stays within a bounded path depth budget.".to_string(),
        "CONFIGS-016" => "Each config file must map to exactly one visibility class and each group declares its stability.".to_string(),
        "CONFIGS-017" => "Every configs group must declare the commands that consume that configuration surface.".to_string(),
        "CONFIGS-018" => "Schema-bearing groups must declare an explicit schema owner and real schema files.".to_string(),
        "CONFIGS-019" => "Each configs group declares stable lifecycle metadata through owner, schema owner, and stability.".to_string(),
        "CONFIGS-020" => "The generated configs index must be deterministic from the registry.".to_string(),
        "CONFIGS-021" => "The committed generated configs index must match the canonical registry render.".to_string(),
        "CONFIGS-022" => "JSON and JSONC config files must parse successfully.".to_string(),
        "CONFIGS-023" => "YAML config files must parse successfully.".to_string(),
        "CONFIGS-024" => "TOML config files must parse successfully.".to_string(),
        "CONFIGS-025" => "Config text files must not accumulate trailing whitespace drift.".to_string(),
        "CONFIGS-026" => "The configs/docs directory must not contain narrative markdown.".to_string(),
        "CONFIGS-027" => "The configs/docs directory must stay within its declared tooling file surface.".to_string(),
        "CONFIGS-028" => "The canonical configs owner map must match the registry group owners.".to_string(),
        "CONFIGS-029" => "The canonical configs consumer map must cover the registry groups.".to_string(),
        "CONFIGS-030" => "Every public or generated config file must have explicit file-level consumer coverage in configs/CONSUMERS.json.".to_string(),
        "CONFIGS-031" => "Root, public, and generated JSON or JSONC configs must map to explicit schema coverage in configs/SCHEMAS.json.".to_string(),
        "CONFIGS-032" => "The root configs authority JSON files and generated configs index must stay in canonical sorted pretty JSON form.".to_string(),
        "CONFIGS-033" => "The committed configs schema index must match the canonical render from configs/SCHEMAS.json and the schema directories.".to_string(),
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts configs`.".to_string(),
    }
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
        "config path `{normalized}` is not covered by configs/inventory/configs.json"
    ))
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts configs --mode static"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace")
            .parent()
            .expect("repo")
            .to_path_buf()
    }

    #[test]
    fn registry_index_parses_and_exposes_groups() {
        let index = registry_index(&repo_root()).expect("registry");
        assert_eq!(index.registry.schema_version, 1);
        assert!(index
            .registry
            .groups
            .iter()
            .any(|group| group.name == "inventory"));
        assert!(index
            .registry
            .groups
            .iter()
            .any(|group| group.name == "inventory"
                && group
                    .tool_entrypoints
                    .iter()
                    .any(|entry| entry == "bijux dev atlas configs list")));
    }

    #[test]
    fn generated_index_render_is_stable() {
        let root = repo_root();
        let first = generated_index_json(&root).expect("first");
        let second = generated_index_json(&root).expect("second");
        assert_eq!(first, second);
        let groups = first["groups"].as_array().expect("groups");
        assert!(!groups.is_empty());
    }

    #[test]
    fn wildcard_match_supports_double_star_segments() {
        assert!(wildcard_match(
            "configs/openapi/**/*.json",
            "configs/openapi/v1/openapi.generated.json"
        ));
        assert!(wildcard_match(
            "configs/docs/.vale/styles/**",
            "configs/docs/.vale/styles/Bijux/terminology.yml"
        ));
        assert!(!wildcard_match(
            "configs/docs/*.json",
            "configs/docs/schema-validation.md"
        ));
    }

    #[test]
    fn explain_payload_returns_group_metadata() {
        let payload =
            explain_payload(&repo_root(), "configs/rust/rustfmt.toml").expect("explain payload");
        assert_eq!(payload["group"].as_str(), Some("rust"));
        assert_eq!(payload["visibility"].as_str(), Some("public"));
        assert_eq!(payload["owner"].as_str(), Some("rust-foundation"));
        assert!(payload["consumers"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty()));
        assert!(payload["schema"].is_null());
    }

    #[test]
    fn explain_payload_returns_schema_for_json_file() {
        let payload = explain_payload(&repo_root(), "configs/inventory/configs.json")
            .expect("explain payload");
        assert_eq!(
            payload["schema"].as_str(),
            Some("configs/contracts/inventory-configs.schema.json")
        );
    }

    #[test]
    fn contract_surface_registry_parses_and_covers_cfg_ids() {
        let surface = read_contract_surface(&repo_root()).expect("contract surface");
        assert_eq!(surface.schema_version, 1);
        assert_eq!(surface.domain, "configs");
        assert_eq!(surface.contracts.len(), 37);
        assert!(surface.contracts.iter().any(|row| row.id == "CFG-001"));
        assert!(surface
            .contracts
            .iter()
            .any(|row| row.enforced_by.test_id == "configs.schemas.file_coverage"));
    }

    #[test]
    fn cfg_contract_coverage_payload_is_stable() {
        let payload = cfg_contract_coverage_payload(&repo_root()).expect("coverage payload");
        assert_eq!(payload["contract_count"].as_u64(), Some(37));
        assert!(payload["mapped_checks"].as_u64().is_some());
        assert!(payload["total_checks"].as_u64().is_some());
        assert!(payload["coverage_pct"].as_u64().is_some());
        assert!(payload["registry_sha256"].as_str().is_some());
    }
}
