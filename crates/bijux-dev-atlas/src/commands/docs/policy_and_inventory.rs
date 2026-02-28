
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct DocsQualityPolicy {
    #[serde(default = "default_stale_days")]
    pub(crate) stale_days: i64,
    #[serde(default = "default_area_budget")]
    pub(crate) default_area_budget: usize,
    #[serde(default)]
    pub(crate) area_budgets: BTreeMap<String, usize>,
    #[serde(default)]
    pub(crate) naming: NamingPolicy,
    #[serde(default)]
    pub(crate) terminology: TerminologyPolicy,
    #[serde(default)]
    pub(crate) markdown: MarkdownPolicy,
    #[serde(default)]
    pub(crate) diagrams: DiagramPolicy,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub(crate) struct NamingPolicy {
    #[serde(default)]
    pub(crate) forbidden_words: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub(crate) struct TerminologyPolicy {
    #[serde(default)]
    pub(crate) forbidden_terms: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct MarkdownPolicy {
    #[serde(default = "default_max_line_length")]
    pub(crate) max_line_length: usize,
    #[serde(default = "default_require_h1")]
    pub(crate) require_h1: bool,
    #[serde(default)]
    pub(crate) require_sections: Vec<String>,
}

impl Default for MarkdownPolicy {
    fn default() -> Self {
        Self {
            max_line_length: default_max_line_length(),
            require_h1: default_require_h1(),
            require_sections: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub(crate) struct DiagramPolicy {
    #[serde(default)]
    pub(crate) extensions: Vec<String>,
    #[serde(default)]
    pub(crate) roots: Vec<String>,
}

fn default_stale_days() -> i64 {
    90
}

fn default_area_budget() -> usize {
    10
}

fn default_max_line_length() -> usize {
    180
}

fn default_require_h1() -> bool {
    true
}

impl Default for DocsQualityPolicy {
    fn default() -> Self {
        Self {
            stale_days: default_stale_days(),
            default_area_budget: default_area_budget(),
            area_budgets: BTreeMap::new(),
            naming: NamingPolicy::default(),
            terminology: TerminologyPolicy::default(),
            markdown: MarkdownPolicy::default(),
            diagrams: DiagramPolicy::default(),
        }
    }
}

pub(crate) fn load_quality_policy(repo_root: &Path) -> DocsQualityPolicy {
    let path = repo_root.join("configs/docs/quality-policy.json");
    let Ok(text) = fs::read_to_string(path) else {
        return DocsQualityPolicy::default();
    };
    serde_json::from_str::<DocsQualityPolicy>(&text).unwrap_or_default()
}

pub(crate) fn docs_context(common: &DocsCommonArgs) -> Result<DocsContext, String> {
    let repo_root = if let Some(path) = common.repo_root.clone() {
        path.canonicalize().map_err(|err| err.to_string())?
    } else {
        resolve_repo_root(None)?
    };
    let artifacts_root = common
        .artifacts_root
        .clone()
        .unwrap_or_else(|| repo_root.join("artifacts"));
    let run_id = common
        .run_id
        .as_ref()
        .map(|v| RunId::parse(v))
        .transpose()?
        .unwrap_or_else(|| RunId::from_seed("docs_run"));
    Ok(DocsContext {
        docs_root: repo_root.join("docs"),
        repo_root,
        artifacts_root,
        run_id,
    })
}

fn slugify_anchor(text: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for c in text.chars().flat_map(|c| c.to_lowercase()) {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if (c.is_whitespace() || c == '-' || c == '_') && !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

pub(crate) fn docs_markdown_files(docs_root: &Path, include_drafts: bool) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if docs_root.exists() {
        for file in walk_files_local(docs_root) {
            if file.extension().and_then(|v| v.to_str()) == Some("md") {
                if !include_drafts {
                    if let Ok(rel) = file.strip_prefix(docs_root) {
                        if rel.to_string_lossy().starts_with("_drafts/") {
                            continue;
                        }
                    }
                }
                files.push(file);
            }
        }
    }
    files.sort();
    files
}

pub(crate) fn walk_files_local(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn parse_mkdocs_yaml(repo_root: &Path) -> Result<YamlValue, String> {
    let path = repo_root.join("mkdocs.yml");
    let text =
        fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    serde_yaml::from_str(&text).map_err(|e| format!("failed to parse {}: {e}", path.display()))
}

fn collect_nav_refs(node: &YamlValue, out: &mut Vec<(String, String)>) {
    match node {
        YamlValue::Sequence(seq) => {
            for item in seq {
                collect_nav_refs(item, out);
            }
        }
        YamlValue::Mapping(map) => {
            for (k, v) in map {
                let title = k.as_str().unwrap_or_default().to_string();
                if let Some(path) = v.as_str() {
                    out.push((title, path.to_string()));
                } else {
                    collect_nav_refs(v, out);
                }
            }
        }
        _ => {}
    }
}

fn collect_nav_depth(node: &YamlValue, depth: usize, max_depth: &mut usize) {
    *max_depth = (*max_depth).max(depth);
    match node {
        YamlValue::Sequence(seq) => {
            for item in seq {
                collect_nav_depth(item, depth + 1, max_depth);
            }
        }
        YamlValue::Mapping(map) => {
            for (_, v) in map {
                collect_nav_depth(v, depth + 1, max_depth);
            }
        }
        _ => {}
    }
}

pub(crate) fn mkdocs_nav_refs(repo_root: &Path) -> Result<Vec<(String, String)>, String> {
    let yaml = parse_mkdocs_yaml(repo_root)?;
    let nav = yaml
        .get("nav")
        .ok_or_else(|| "mkdocs.yml missing `nav`".to_string())?;
    let mut refs = Vec::new();
    collect_nav_refs(nav, &mut refs);
    refs.sort();
    Ok(refs)
}

pub(crate) fn docs_inventory_payload(
    ctx: &DocsContext,
    common: &DocsCommonArgs,
) -> Result<serde_json::Value, String> {
    let nav_refs = mkdocs_nav_refs(&ctx.repo_root)?;
    let nav_set = nav_refs
        .iter()
        .map(|(_, p)| p.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let rows = docs_markdown_files(&ctx.docs_root, common.include_drafts)
        .into_iter()
        .filter_map(|p| {
            p.strip_prefix(&ctx.docs_root)
                .ok()
                .map(|r| r.display().to_string())
        })
        .map(|rel| DocsPageRow {
            in_nav: nav_set.contains(&rel),
            path: rel,
        })
        .collect::<Vec<_>>();
    let orphan_pages = rows
        .iter()
        .filter(|r| {
            !r.in_nav
                && !r.path.starts_with("_assets/")
                && (common.include_drafts || !r.path.starts_with("_drafts/"))
        })
        .map(|r| r.path.clone())
        .collect::<Vec<_>>();
    let duplicate_titles = {
        let mut seen = BTreeMap::<String, usize>::new();
        for (title, _) in &nav_refs {
            *seen.entry(title.clone()).or_default() += 1;
        }
        let mut d = seen
            .into_iter()
            .filter(|(_, n)| *n > 1)
            .map(|(k, _)| k)
            .collect::<Vec<_>>();
        d.sort();
        d
    };
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},
        "options": {"strict": common.strict, "include_drafts": common.include_drafts},
        "nav": nav_refs.iter().map(|(title, path)| serde_json::json!({"title": title, "path": path})).collect::<Vec<_>>(),
        "pages": rows,
        "orphan_pages": orphan_pages,
        "duplicate_nav_titles": duplicate_titles
    }))
}

fn scan_registry_markdown_files(repo_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for file in walk_files_local(repo_root) {
        if file.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let Ok(rel) = file.strip_prefix(repo_root) else {
            continue;
        };
        let rels = rel.to_string_lossy();
        if rels.starts_with("artifacts/") || rels.contains("/target/") {
            continue;
        }
        if rels == "docs/_generated/make-targets.md" {
            continue;
        }
        if !is_allowed_doc_location(&rels) {
            continue;
        }
        if rels.starts_with("docs/_generated/") || rels.starts_with("docs/_drafts/") {
            continue;
        }
        files.push(file);
    }
    files.sort();
    files
}

fn is_allowed_doc_location(path: &str) -> bool {
    matches!(
        path,
        "README.md" | "CONTRIBUTING.md" | "SECURITY.md" | "CHANGELOG.md" | "CONTRACT.md"
    ) || path.starts_with("docs/")
        || path.starts_with("crates/")
        || path.starts_with("ops/")
        || path.starts_with("configs/")
        || path.starts_with("docker/")
        || path == "make/README.md"
        || path == "make/CONTRACT.md"
        || path.starts_with("make/makefiles/")
        || path.starts_with(".github/")
}

fn read_dir_entries(path: &Path) -> Vec<PathBuf> {
    match fs::read_dir(path) {
        Ok(entries) => entries.filter_map(Result::ok).map(|e| e.path()).collect(),
        Err(_) => Vec::new(),
    }
}

fn infer_doc_type(path: &str) -> &'static str {
    if path.contains("/runbooks/") {
        "runbook"
    } else if path.contains("/contracts/") || path.contains("SCHEMA") || path.contains("OPENAPI") {
        "spec"
    } else if path.contains("/quickstart/") || path.contains("how-to") {
        "how-to"
    } else if path.contains("/reference/") {
        "reference"
    } else {
        "concept"
    }
}

fn infer_lifecycle(path: &str) -> &'static str {
    if path.contains("/_drafts/") {
        "draft"
    } else if path.contains("/_style/") || path.contains("/_lint/") || path.contains("/_nav/") {
        "internal"
    } else {
        "stable"
    }
}

fn parse_owner_and_stability(file: &Path) -> (String, String) {
    let Ok(text) = fs::read_to_string(file) else {
        return ("docs-governance".to_string(), "stable".to_string());
    };
    let mut owner = None;
    let mut stability = None;
    for line in text.lines().take(40) {
        let trimmed = line.trim();
        if owner.is_none() && trimmed.starts_with("- Owner:") {
            owner = Some(
                trimmed
                    .trim_start_matches("- Owner:")
                    .trim()
                    .trim_matches('`')
                    .to_string(),
            );
        }
        if stability.is_none() && trimmed.starts_with("- Stability:") {
            stability = Some(
                trimmed
                    .trim_start_matches("- Stability:")
                    .trim()
                    .trim_matches('`')
                    .to_string(),
            );
        }
    }
    (
        owner
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "docs-governance".to_string()),
        stability
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "stable".to_string()),
    )
}

fn crate_association(path: &str) -> Option<String> {
    let parts = path.split('/').collect::<Vec<_>>();
    if parts.len() >= 3 && parts[0] == "crates" && parts[2] == "docs" {
        Some(parts[1].to_string())
    } else {
        None
    }
}

pub(crate) fn workspace_crate_roots(repo_root: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let crates_dir = repo_root.join("crates");
    if !crates_dir.exists() {
        return roots;
    }
    for entry in read_dir_entries(&crates_dir) {
        if !entry.is_dir() {
            continue;
        }
        if entry.join("Cargo.toml").exists() {
            roots.push(entry);
        }
    }
    roots.sort();
    roots
}
