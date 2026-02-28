// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Deserialize;

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

const REGISTRY_PATH: &str = "configs/inventory/configs.json";

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

#[derive(Clone)]
struct RegistryIndex {
    registry: ConfigsRegistry,
    files: Vec<String>,
    excluded_files: BTreeSet<String>,
    root_files: BTreeSet<String>,
    group_files: BTreeMap<String, GroupFiles>,
    contract_ids: BTreeSet<String>,
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
    let contract_ids = contract_doc
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let id = trimmed
                .strip_prefix("- `")
                .and_then(|value| value.split('`').next())?;
            if id.starts_with("CONFIGS-") {
                Some(id.to_string())
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();
    Ok(RegistryIndex {
        registry,
        files,
        excluded_files,
        root_files,
        group_files,
        contract_ids,
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
            if !pattern.contains("/_generated/") && !pattern.contains("/_generated.") {
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
            if !file.contains("/_generated/") && !file.contains("/_generated.") {
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
    let expected = (1..=25)
        .map(|n| format!("CONFIGS-{n:03}"))
        .collect::<BTreeSet<_>>();
    if index.contract_ids == expected {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "CONFIGS-010",
            "configs.contracts.no_policy_theater",
            "configs/CONTRACT.md",
            format!(
                "expected contract doc ids {:?}, found {:?}",
                expected, index.contract_ids
            ),
        )])
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
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts configs`.".to_string(),
    }
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts configs --mode static"
}
