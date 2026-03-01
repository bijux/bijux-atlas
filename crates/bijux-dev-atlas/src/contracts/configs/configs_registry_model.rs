const REGISTRY_PATH: &str = "configs/inventory/configs.json";
const CONTRACT_SURFACE_PATH: &str = "configs/configs.contracts.json";
const OWNERS_PATH: &str = "configs/owners-registry.json";
const CONSUMERS_PATH: &str = "configs/consumers-registry.json";
const SCHEMAS_PATH: &str = "configs/schema-map.json";
const SCHEMA_VERSIONING_POLICY_PATH: &str = "configs/schema/versioning-policy.json";
const ROOT_CANONICAL_JSON_FILES: [&str; 6] = [
    "configs/owners-registry.json",
    "configs/consumers-registry.json",
    "configs/schema-map.json",
    "configs/configs.contracts.json",
    "configs/inventory.json",
    "configs/_generated/configs-index.json",
];
const ROOT_MARKDOWN_FILES: [&str; 3] =
    ["configs/README.md", "configs/INDEX.md", "configs/CONTRACT.md"];
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
    approved_by: Option<String>,
    expires_on: Option<String>,
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
struct SchemaVersioningPolicy {
    schema_version: u64,
    kind: String,
    policies: Vec<SchemaVersioningRule>,
}

#[derive(Clone, Deserialize)]
struct SchemaVersioningRule {
    schema: String,
    compatibility: String,
    versioning: String,
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

fn looks_like_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(idx, byte)| matches!(idx, 4 | 7) || byte.is_ascii_digit())
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

fn read_schema_versioning_policy(repo_root: &Path) -> Result<SchemaVersioningPolicy, String> {
    let text = read_text(&repo_root.join(SCHEMA_VERSIONING_POLICY_PATH))?;
    serde_json::from_str::<SchemaVersioningPolicy>(&text)
        .map_err(|err| format!("parse {SCHEMA_VERSIONING_POLICY_PATH} failed: {err}"))
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
