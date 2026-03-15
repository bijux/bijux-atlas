
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct DocsQualityPolicy {
    #[serde(default = "default_reference_date")]
    pub(crate) reference_date: String,
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

fn default_reference_date() -> String {
    "2026-02-25".to_string()
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
            reference_date: default_reference_date(),
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
                .map(|r| (p.clone(), r.display().to_string()))
        })
        .map(|(path, rel)| {
            let (owner, stability, last_reviewed) =
                extract_frontmatter_docs_metadata(&path).unwrap_or((None, None, None));
            DocsPageRow {
                in_nav: nav_set.contains(&rel),
                path: rel,
                owner,
                stability,
                last_reviewed,
            }
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

fn extract_frontmatter_docs_metadata(
    path: &Path,
) -> Result<FrontmatterDocsMetadata, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Ok((None, None, None));
    }
    let mut owner = None;
    let mut stability = None;
    let mut last_reviewed = None;
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            break;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
        match key {
            "owner" if !value.is_empty() => owner = Some(value),
            "stability" if !value.is_empty() => stability = Some(value),
            "last_reviewed" if !value.is_empty() => last_reviewed = Some(value),
            _ => {}
        }
    }
    Ok((owner, stability, last_reviewed))
}

type FrontmatterDocsMetadata = (Option<String>, Option<String>, Option<String>);
