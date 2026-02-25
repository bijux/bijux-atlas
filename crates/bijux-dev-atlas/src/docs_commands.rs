// SPDX-License-Identifier: Apache-2.0

use crate::*;
use std::collections::{BTreeMap, BTreeSet};

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
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
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
        if rels == "makefiles/GENERATED_TARGETS.md" {
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
        "README.md" | "CONTRIBUTING.md" | "SECURITY.md" | "CHANGELOG.md"
    ) || path.starts_with("docs/")
        || path.starts_with("crates/")
        || path.starts_with("ops/")
        || path.starts_with("configs/")
        || path.starts_with("docker/")
        || path.starts_with("makefiles/")
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
        owner.unwrap_or_else(|| "docs-governance".to_string()),
        stability.unwrap_or_else(|| "stable".to_string()),
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

pub(crate) fn crate_doc_contract_status(
    repo_root: &Path,
) -> (Vec<serde_json::Value>, Vec<String>, Vec<String>) {
    let required_common = [
        "README.md",
        "ARCHITECTURE.md",
        "CONTRACT.md",
        "TESTING.md",
        "ERROR_TAXONOMY.md",
        "EXAMPLES.md",
        "BENCHMARKS.md",
        "VERSIONING.md",
    ];
    let mut rows = Vec::<serde_json::Value>::new();
    let mut errors = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();

    for crate_root in workspace_crate_roots(repo_root) {
        let crate_name = crate_root
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("unknown");
        let root_md = read_dir_entries(&crate_root)
            .into_iter()
            .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("md"))
            .collect::<Vec<_>>();
        let docs_dir = crate_root.join("docs");
        let docs_md = if docs_dir.exists() {
            walk_files_local(&docs_dir)
                .into_iter()
                .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("md"))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        let root_names = root_md
            .iter()
            .filter_map(|p| p.file_name().and_then(|v| v.to_str()))
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        let mut allowed_root = BTreeSet::from([
            "README.md".to_string(),
            "ARCHITECTURE.md".to_string(),
            "CONTRACT.md".to_string(),
            "TESTING.md".to_string(),
            "ERROR_TAXONOMY.md".to_string(),
            "EXAMPLES.md".to_string(),
            "BENCHMARKS.md".to_string(),
            "VERSIONING.md".to_string(),
        ]);
        if crate_name.contains("-model") {
            allowed_root.insert("DATA_MODEL.md".to_string());
        }
        if crate_name.contains("-policies") {
            allowed_root.insert("EXTENSION_GUIDE.md".to_string());
        }
        if crate_name.ends_with("-adapters") {
            allowed_root.insert("ADAPTER_BOUNDARY.md".to_string());
        }
        if crate_name.ends_with("-cli") || crate_name.ends_with("dev-atlas") {
            allowed_root.insert("COMMAND_SURFACE.md".to_string());
        }
        if root_names.len() > 10 {
            errors.push(format!(
                "CRATE_DOC_BUDGET_ERROR: `{crate_name}` has {} root docs (budget=10)",
                root_names.len()
            ));
        }
        for root_name in &root_names {
            if !allowed_root.contains(root_name) {
                warnings.push(format!(
                    "CRATE_DOC_ALLOWED_TYPE_WARN: `{crate_name}` has non-canonical root doc `{root_name}`"
                ));
            }
        }

        for required in &required_common {
            if !root_names.contains(*required) {
                errors.push(format!(
                    "CRATE_DOC_REQUIRED_ERROR: `{crate_name}` missing `{required}`"
                ));
            }
        }

        if crate_name.contains("-model") && !root_names.contains("DATA_MODEL.md") {
            errors.push(format!(
                "CRATE_DOC_REQUIRED_ERROR: `{crate_name}` missing `DATA_MODEL.md`"
            ));
        }
        if crate_name.contains("-policies") && !root_names.contains("EXTENSION_GUIDE.md") {
            errors.push(format!(
                "CRATE_DOC_REQUIRED_ERROR: `{crate_name}` missing `EXTENSION_GUIDE.md`"
            ));
        }
        if (crate_name.ends_with("-cli") || crate_name.ends_with("dev-atlas"))
            && !root_names.contains("COMMAND_SURFACE.md")
        {
            errors.push(format!(
                "CRATE_DOC_REQUIRED_ERROR: `{crate_name}` missing `COMMAND_SURFACE.md`"
            ));
        }
        if crate_name.ends_with("-adapters") && !root_names.contains("ADAPTER_BOUNDARY.md") {
            errors.push(format!(
                "CRATE_DOC_REQUIRED_ERROR: `{crate_name}` missing `ADAPTER_BOUNDARY.md`"
            ));
        }

        if crate_name.contains("-core") {
            let architecture = crate_root.join("ARCHITECTURE.md");
            let has_invariants = fs::read_to_string(&architecture)
                .ok()
                .is_some_and(|text| text.contains("## Invariants"));
            if !has_invariants {
                warnings.push(format!(
                    "CRATE_DOC_INVARIANTS_WARN: `{crate_name}` ARCHITECTURE.md should include `## Invariants`"
                ));
            }
        }

        let readme_path = crate_root.join("README.md");
        if let Ok(readme) = fs::read_to_string(&readme_path) {
            if !readme.contains("docs/") {
                warnings.push(format!(
                    "CRATE_DOC_LINK_WARN: `{crate_name}` README.md should link to crate docs/"
                ));
            }
        }
        let index_path = crate_root.join("docs/INDEX.md");
        if let Ok(index) = fs::read_to_string(&index_path) {
            for expected in ["architecture.md", "public-api.md", "testing.md"] {
                if !index.contains(expected) {
                    warnings.push(format!(
                        "CRATE_DOC_CROSSLINK_WARN: `{crate_name}` docs/INDEX.md should reference `{expected}`"
                    ));
                }
            }
        }

        let mut diagram_count = 0usize;
        let mut rust_fences = 0usize;
        let mut rust_fences_tagged = 0usize;
        let mut docs_with_owner = 0usize;
        let mut docs_with_last_reviewed = 0usize;
        for doc in root_md.iter().chain(docs_md.iter()) {
            let text = fs::read_to_string(doc).unwrap_or_default();
            diagram_count += text.matches("![").count();
            docs_with_owner += usize::from(text.contains("- Owner:"));
            docs_with_last_reviewed += usize::from(text.contains("Last Reviewed:"));
            for line in text.lines() {
                if line.trim_start().starts_with("```") {
                    rust_fences += usize::from(line.trim() == "```");
                    rust_fences_tagged += usize::from(line.trim().starts_with("```rust"));
                }
            }
        }
        if diagram_count > 20 {
            warnings.push(format!(
                "CRATE_DOC_DIAGRAM_BUDGET_WARN: `{crate_name}` has {diagram_count} diagrams (budget=20)"
            ));
        }
        if rust_fences > rust_fences_tagged {
            warnings.push(format!(
                "CRATE_DOC_EXAMPLE_TAG_WARN: `{crate_name}` has untagged code fences; prefer ```rust for examples"
            ));
        }
        let total_docs = root_md.len() + docs_md.len();
        if docs_with_owner < total_docs {
            warnings.push(format!(
                "CRATE_DOC_OWNER_METADATA_WARN: `{crate_name}` owner metadata present in {docs_with_owner}/{total_docs} docs"
            ));
        }
        if docs_with_last_reviewed == 0 {
            warnings.push(format!(
                "CRATE_DOC_FRESHNESS_WARN: `{crate_name}` has no `Last Reviewed:` metadata in crate docs"
            ));
        }

        rows.push(serde_json::json!({
            "crate": crate_name,
            "root_doc_count": root_names.len(),
            "docs_dir_count": docs_md.len(),
            "required": required_common,
            "has": root_names,
            "diagram_count": diagram_count,
            "owner_metadata_docs": docs_with_owner,
            "freshness_docs": docs_with_last_reviewed
        }));
    }
    let mut concept_index = BTreeMap::<String, Vec<String>>::new();
    for crate_root in workspace_crate_roots(repo_root) {
        let crate_name = crate_root
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("unknown")
            .to_string();
        let docs_dir = crate_root.join("docs");
        if !docs_dir.exists() {
            continue;
        }
        for file in walk_files_local(&docs_dir) {
            if file.extension().and_then(|v| v.to_str()) != Some("md") {
                continue;
            }
            let Some(name) = file.file_name().and_then(|v| v.to_str()) else {
                continue;
            };
            let concept = name.to_ascii_lowercase();
            if matches!(
                concept.as_str(),
                "index.md"
                    | "architecture.md"
                    | "public-api.md"
                    | "testing.md"
                    | "effects.md"
                    | "effect-boundary-map.md"
            ) {
                continue;
            }
            concept_index
                .entry(concept)
                .or_default()
                .push(crate_name.clone());
        }
    }
    for (concept, crates) in concept_index {
        let distinct = crates.into_iter().collect::<BTreeSet<_>>();
        if distinct.len() > 1 {
            warnings.push(format!(
                "CRATE_DOC_DUPLICATE_CONCEPT_WARN: `{concept}` appears across crates: {}",
                distinct.into_iter().collect::<Vec<_>>().join(", ")
            ));
        }
    }
    rows.sort_by(|a, b| a["crate"].as_str().cmp(&b["crate"].as_str()));
    errors.sort();
    errors.dedup();
    warnings.sort();
    warnings.dedup();
    (rows, errors, warnings)
}

fn tags_for_path(path: &str) -> Vec<String> {
    let mut out = BTreeSet::new();
    for segment in path.split('/') {
        if segment.is_empty() || segment == "docs" || segment == "crates" {
            continue;
        }
        let tag = segment
            .trim_end_matches(".md")
            .replace('_', "-")
            .to_ascii_lowercase();
        if tag.len() >= 3 {
            out.insert(tag);
        }
    }
    out.into_iter().take(8).collect()
}

pub(crate) fn search_synonyms(repo_root: &Path) -> Vec<serde_json::Value> {
    let path = repo_root.join("docs/metadata/search-synonyms.json");
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| v.get("synonyms").and_then(|s| s.as_array().cloned()))
        .unwrap_or_default()
}

pub(crate) fn docs_registry_payload(ctx: &DocsContext) -> serde_json::Value {
    let mut docs = Vec::new();
    for file in scan_registry_markdown_files(&ctx.repo_root) {
        let Ok(rel) = file.strip_prefix(&ctx.repo_root) else {
            continue;
        };
        let rel_path = rel.display().to_string();
        let (owner, stability) = parse_owner_and_stability(&file);
        let crate_name = crate_association(&rel_path);
        docs.push(serde_json::json!({
            "path": rel_path,
            "doc_type": infer_doc_type(&rel.display().to_string()),
            "owner": owner,
            "crate": crate_name,
            "stability": stability,
            "last_reviewed": "2026-02-25",
            "review_due": "2026-08-24",
            "lifecycle": infer_lifecycle(&rel.display().to_string()),
            "tags": tags_for_path(&rel.display().to_string()),
            "keywords": tags_for_path(&rel.display().to_string()),
            "doc_version": "v1",
            "topic": rel.file_stem().and_then(|v| v.to_str()).unwrap_or("unknown")
        }));
    }
    docs.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    serde_json::json!({
        "schema_version": 1,
        "project_version": "v0.1.0",
        "generated_by": "bijux dev atlas docs registry build",
        "generated_from": "docs and crate docs",
        "documents": docs
    })
}

fn parse_ymd_date(s: &str) -> Option<(i32, i32, i32)> {
    let parts: Vec<_> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let y = parts[0].parse().ok()?;
    let m = parts[1].parse().ok()?;
    let d = parts[2].parse().ok()?;
    Some((y, m, d))
}

fn days_from_civil(y: i32, m: i32, d: i32) -> i64 {
    let y = y - i32::from(m <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = m + if m > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era * 146_097 + doe - 719_468) as i64
}

fn date_diff_days(older: &str, newer: &str) -> Option<i64> {
    let (y1, m1, d1) = parse_ymd_date(older)?;
    let (y2, m2, d2) = parse_ymd_date(newer)?;
    Some(days_from_civil(y2, m2, d2) - days_from_civil(y1, m1, d1))
}

pub(crate) fn has_required_section(text: &str, section: &str) -> bool {
    let needle = format!("## {section}");
    text.lines().any(|line| line.trim() == needle)
}

pub(crate) fn registry_validate_payload(ctx: &DocsContext) -> Result<serde_json::Value, String> {
    let registry_path = ctx.repo_root.join("docs/registry.json");
    if !registry_path.exists() {
        return Ok(serde_json::json!({
            "schema_version": 1,
            "errors": [],
            "warnings": ["DOCS_REGISTRY_MISSING: docs/registry.json is missing"],
            "summary": {"errors": 0, "warnings": 1}
        }));
    }
    let text = fs::read_to_string(&registry_path)
        .map_err(|e| format!("failed to read {}: {e}", registry_path.display()))?;
    let registry: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("invalid docs registry json: {e}"))?;
    let docs = registry["documents"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let policy = load_quality_policy(&ctx.repo_root);
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut seen_paths = BTreeSet::new();
    let mut seen_topics = BTreeMap::<String, usize>::new();
    let scanned = scan_registry_markdown_files(&ctx.repo_root)
        .into_iter()
        .filter_map(|p| {
            p.strip_prefix(&ctx.repo_root)
                .ok()
                .map(|r| r.display().to_string())
        })
        .collect::<BTreeSet<_>>();
    for entry in &docs {
        let Some(path) = entry["path"].as_str() else {
            errors.push("DOCS_REGISTRY_INVALID_ENTRY: missing path".to_string());
            continue;
        };
        if !seen_paths.insert(path.to_string()) {
            errors.push(format!("DOCS_REGISTRY_DUPLICATE_PATH: `{path}`"));
        }
        if !ctx.repo_root.join(path).exists() {
            errors.push(format!("DOCS_REGISTRY_MISSING_FILE: `{path}`"));
        }
        if let Some(topic) = entry["topic"].as_str() {
            *seen_topics.entry(topic.to_string()).or_default() += 1;
        }
        if let Some(last_reviewed) = entry["last_reviewed"].as_str() {
            if let Some(age_days) = date_diff_days(last_reviewed, "2026-02-25") {
                if age_days > policy.stale_days {
                    warnings.push(format!(
                        "DOCS_REGISTRY_OUTDATED: `{path}` last_reviewed={last_reviewed} age_days={age_days}"
                    ));
                }
            }
        } else {
            warnings.push(format!("DOCS_REGISTRY_MISSING_LAST_REVIEWED: `{path}`"));
        }
        if entry["owner"].as_str().unwrap_or("unknown") == "unknown" {
            errors.push(format!(
                "DOCS_REGISTRY_OWNER_REQUIRED: `{path}` requires owner metadata"
            ));
        }
    }
    for path in scanned.difference(&seen_paths) {
        errors.push(format!("DOCS_REGISTRY_ORPHAN_DOC: `{path}` not registered"));
    }
    for path in seen_paths.difference(&scanned) {
        errors.push(format!("DOCS_REGISTRY_ORPHAN_ENTRY: `{path}` has no file"));
    }
    for (topic, count) in seen_topics {
        if count > 1 {
            warnings.push(format!(
                "DOCS_REGISTRY_DUPLICATE_TOPIC: `{topic}` appears {count} times"
            ));
        }
    }
    let mut per_crate = BTreeMap::<String, usize>::new();
    for entry in &docs {
        let bucket = entry["crate"].as_str().unwrap_or("docs-root").to_string();
        *per_crate.entry(bucket).or_default() += 1;
    }
    for (bucket, count) in per_crate {
        let budget = policy
            .area_budgets
            .get(&bucket)
            .copied()
            .unwrap_or(policy.default_area_budget);
        if count > budget {
            errors.push(format!(
                "DOCS_REGISTRY_DOC_BUDGET_ERROR: `{bucket}` has {count} docs (budget={budget})"
            ));
        }
    }
    let registered = docs
        .iter()
        .filter_map(|v| v["path"].as_str())
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let mut incoming = BTreeMap::<String, usize>::new();
    let link_re = Regex::new(r"\[[^\]]+\]\(([^)]+)\)").map_err(|e| e.to_string())?;
    for file in scan_registry_markdown_files(&ctx.repo_root) {
        let source = file
            .strip_prefix(&ctx.repo_root)
            .ok()
            .map(|v| v.display().to_string())
            .unwrap_or_default();
        let text = fs::read_to_string(&file).unwrap_or_default();
        for cap in link_re.captures_iter(&text) {
            let target = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
            if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
                || target.starts_with('#')
            {
                continue;
            }
            let path_part = target.split('#').next().unwrap_or_default();
            if path_part.is_empty() {
                continue;
            }
            let resolved = file.parent().unwrap_or(&ctx.repo_root).join(path_part);
            if let Ok(rel) = resolved.strip_prefix(&ctx.repo_root) {
                let rels = rel.display().to_string();
                if registered.contains(&rels) {
                    *incoming.entry(rels).or_default() += 1;
                }
            }
        }
        let _ = source;
    }
    for path in &registered {
        let basename = Path::new(path)
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        if matches!(basename, "README.md" | "INDEX.md") {
            continue;
        }
        if incoming.get(path).copied().unwrap_or(0) == 0 {
            warnings.push(format!(
                "DOCS_REGISTRY_UNUSED_DOC_WARN: `{path}` has no inbound doc links"
            ));
        }
    }
    let root_md = read_dir_entries(&ctx.repo_root)
        .into_iter()
        .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    for file in root_md {
        let name = file
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        if !matches!(
            name,
            "README.md" | "CONTRIBUTING.md" | "SECURITY.md" | "CHANGELOG.md"
        ) {
            errors.push(format!(
                "DOCS_REGISTRY_ROOT_DOC_FORBIDDEN: allowed root docs are README/CONTRIBUTING/SECURITY/CHANGELOG, found `{}`",
                name
            ));
        }
    }
    for file in walk_files_local(&ctx.repo_root) {
        if file.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let Ok(rel) = file.strip_prefix(&ctx.repo_root) else {
            continue;
        };
        let rels = rel.to_string_lossy().to_string();
        if rels.starts_with("artifacts/") || rels.contains("/target/") {
            continue;
        }
        if !is_allowed_doc_location(&rels) {
            errors.push(format!(
                "DOCS_REGISTRY_DOC_LOCATION_FORBIDDEN: `{}` is outside allowed documentation directories",
                rels
            ));
        }
    }
    let (crate_rows, crate_errors, crate_warnings) = crate_doc_contract_status(&ctx.repo_root);
    errors.extend(crate_errors);
    warnings.extend(crate_warnings);
    warnings.sort();
    warnings.dedup();
    errors.sort();
    errors.dedup();
    let pruning = warnings
        .iter()
        .filter(|w| {
            w.starts_with("DOCS_REGISTRY_OUTDATED:")
                || w.starts_with("DOCS_REGISTRY_UNUSED_DOC_WARN:")
                || w.starts_with("DOCS_REGISTRY_DUPLICATE_TOPIC:")
        })
        .cloned()
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "schema_version": 1,
        "errors": errors,
        "warnings": warnings,
        "crate_docs": crate_rows,
        "pruning_suggestions": pruning,
        "summary": {
            "registered": docs.len(),
            "errors": errors.len(),
            "warnings": warnings.len()
        }
    }))
}

pub(crate) fn docs_validate_payload(
    ctx: &DocsContext,
    common: &DocsCommonArgs,
) -> Result<serde_json::Value, String> {
    let yaml = parse_mkdocs_yaml(&ctx.repo_root)?;
    let mut issues = DocsIssues::default();
    let mut nav_max_depth = 0usize;
    if let Some(nav) = yaml.get("nav") {
        collect_nav_depth(nav, 1, &mut nav_max_depth);
    }
    if nav_max_depth > 8 {
        issues.warnings.push(format!(
            "DOCS_NAV_DEPTH_WARN: nav depth {} exceeds limit 8",
            nav_max_depth
        ));
    }
    let docs_dir = yaml
        .get("docs_dir")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if docs_dir != "docs" {
        issues.errors.push(format!(
            "DOCS_NAV_ERROR: mkdocs.yml docs_dir must be `docs`, got `{docs_dir}`"
        ));
    }
    for (_, rel) in mkdocs_nav_refs(&ctx.repo_root)? {
        if !ctx.docs_root.join(&rel).exists() {
            issues.errors.push(format!(
                "DOCS_NAV_ERROR: mkdocs nav references missing file `{rel}`"
            ));
        }
    }
    let mut body_hashes = BTreeMap::<String, Vec<String>>::new();
    for file in docs_markdown_files(&ctx.docs_root, common.include_drafts) {
        let rel = file
            .strip_prefix(&ctx.docs_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let text = fs::read_to_string(&file).unwrap_or_default();
        let line_count = text.lines().count();
        if line_count > 500 {
            issues.warnings.push(format!(
                "DOCS_SIZE_WARN: `{rel}` has {line_count} lines (budget=500)"
            ));
        }
        let mut sentence_count = 0usize;
        let mut word_count = 0usize;
        for sentence in text.split('.') {
            let words = sentence.split_whitespace().count();
            if words > 0 {
                sentence_count += 1;
                word_count += words;
            }
        }
        if sentence_count > 0 {
            let avg = word_count as f64 / sentence_count as f64;
            if avg > 28.0 {
                issues.warnings.push(format!(
                    "DOCS_READABILITY_WARN: `{rel}` average sentence length {:.1} words",
                    avg
                ));
            }
        }
        let normalized = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        if normalized.len() > 200 {
            let mut hasher = Sha256::new();
            hasher.update(normalized.as_bytes());
            let hash = format!("{:x}", hasher.finalize());
            body_hashes.entry(hash).or_default().push(rel);
        }
    }
    for paths in body_hashes.values() {
        if paths.len() > 1 {
            issues.warnings.push(format!(
                "DOCS_DUPLICATION_WARN: duplicated content across {}",
                paths.join(", ")
            ));
        }
    }
    let inv = docs_inventory_payload(ctx, common)?;
    for dup in inv["duplicate_nav_titles"].as_array().into_iter().flatten() {
        if let Some(title) = dup.as_str() {
            issues.warnings.push(format!(
                "DOCS_NAV_ERROR: duplicate mkdocs nav title `{title}`"
            ));
        }
    }
    let registry_checks = registry_validate_payload(ctx)?;
    for err in registry_checks["errors"].as_array().into_iter().flatten() {
        if let Some(s) = err.as_str() {
            issues.errors.push(s.to_string());
        }
    }
    for warn in registry_checks["warnings"].as_array().into_iter().flatten() {
        if let Some(s) = warn.as_str() {
            issues.warnings.push(s.to_string());
        }
    }
    if common.strict {
        issues.errors.append(&mut issues.warnings);
    }
    let text = if issues.errors.is_empty() {
        format!("docs validate passed (warnings={})", issues.warnings.len())
    } else {
        format!(
            "docs validate failed (errors={} warnings={})",
            issues.errors.len(),
            issues.warnings.len()
        )
    };
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": text,
        "errors": issues.errors,
        "warnings": issues.warnings,
        "rows": inv["nav"].as_array().cloned().unwrap_or_default(),
        "registry": registry_checks,
        "summary": {"total": inv["nav"].as_array().map(|v| v.len()).unwrap_or(0), "errors": inv["errors"].as_array().map(|v| v.len()).unwrap_or(0), "warnings": inv["warnings"].as_array().map(|v| v.len()).unwrap_or(0), "nav_max_depth": nav_max_depth},
        "capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},
        "options": {"strict": common.strict, "include_drafts": common.include_drafts}
    }))
}

fn markdown_anchors(text: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix('#') {
            let heading = rest.trim_start_matches('#').trim();
            if !heading.is_empty() {
                out.insert(slugify_anchor(heading));
            }
        }
    }
    out
}

pub(crate) fn docs_links_payload(
    ctx: &DocsContext,
    common: &DocsCommonArgs,
) -> Result<serde_json::Value, String> {
    let mut rows = Vec::<serde_json::Value>::new();
    let mut issues = DocsIssues::default();
    let link_re = Regex::new(r"\[[^\]]+\]\(([^)]+)\)").map_err(|e| e.to_string())?;
    let image_re = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").map_err(|e| e.to_string())?;
    let mut internal_links = 0usize;
    let mut external_links = 0usize;
    for file in docs_markdown_files(&ctx.docs_root, common.include_drafts) {
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let text = fs::read_to_string(&file).map_err(|e| format!("failed to read {rel}: {e}"))?;
        let anchors = markdown_anchors(&text);
        for (idx, line) in text.lines().enumerate() {
            for cap in image_re.captures_iter(line) {
                let alt = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
                let target = cap.get(2).map(|m| m.as_str()).unwrap_or("").trim();
                if alt.is_empty() {
                    issues.warnings.push(format!(
                        "DOCS_IMAGE_ALT_WARN: {rel}:{} image `{target}` has empty alt text",
                        idx + 1
                    ));
                }
            }
            for cap in link_re.captures_iter(line) {
                let target = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                if target.starts_with("http://")
                    || target.starts_with("https://")
                    || target.starts_with("mailto:")
                {
                    external_links += 1;
                    let mut ok = true;
                    if common.allow_network
                        && (target.starts_with("http://") || target.starts_with("https://"))
                    {
                        let out = ProcessCommand::new("curl")
                            .args(["-sS", "--max-time", "5", "-I", target])
                            .current_dir(&ctx.repo_root)
                            .output();
                        ok = out.map(|o| o.status.success()).unwrap_or(false);
                        if !ok {
                            issues.warnings.push(format!(
                                "DOCS_EXTERNAL_LINK_WARN: {rel}:{} external link check failed `{target}`",
                                idx + 1
                            ));
                        }
                    }
                    rows.push(serde_json::json!({"file": rel, "line": idx + 1, "target": target, "ok": ok, "external": true, "checked_network": common.allow_network}));
                    continue;
                }
                if let Some(anchor) = target.strip_prefix('#') {
                    internal_links += 1;
                    let ok = anchors.contains(anchor);
                    if !ok {
                        issues.errors.push(format!(
                            "DOCS_LINK_ERROR: {rel}:{} missing same-file anchor `#{anchor}`",
                            idx + 1
                        ));
                    }
                    rows.push(serde_json::json!({"file": rel, "line": idx + 1, "target": target, "ok": ok}));
                    continue;
                }
                let (path_part, anchor_part) = target
                    .split_once('#')
                    .map_or((target, None), |(a, b)| (a, Some(b)));
                if path_part.is_empty() || path_part.ends_with('/') {
                    continue;
                }
                internal_links += 1;
                let resolved = file.parent().unwrap_or(&ctx.docs_root).join(path_part);
                let exists = resolved.exists();
                let mut ok = exists;
                if exists {
                    if let Some(anchor) = anchor_part {
                        if resolved.extension().and_then(|v| v.to_str()) == Some("md") {
                            let target_text = fs::read_to_string(&resolved).unwrap_or_default();
                            ok = markdown_anchors(&target_text).contains(anchor);
                        }
                    }
                }
                if !ok {
                    let generated_target =
                        path_part.starts_with("_generated/") || path_part.contains("/_generated/");
                    let message = format!(
                        "DOCS_LINK_ERROR: {rel}:{} unresolved link `{target}`",
                        idx + 1
                    );
                    if generated_target && !common.strict {
                        issues.warnings.push(message);
                    } else {
                        issues.errors.push(message);
                    }
                }
                rows.push(
                    serde_json::json!({"file": rel, "line": idx + 1, "target": target, "ok": ok}),
                );
            }
        }
    }
    rows.sort_by(|a, b| {
        a["file"]
            .as_str()
            .cmp(&b["file"].as_str())
            .then(a["line"].as_u64().cmp(&b["line"].as_u64()))
            .then(a["target"].as_str().cmp(&b["target"].as_str()))
    });
    issues.errors.sort();
    issues.errors.dedup();
    issues.warnings.sort();
    issues.warnings.dedup();
    if common.strict && !issues.warnings.is_empty() {
        issues.errors.append(&mut issues.warnings);
        issues.errors.sort();
        issues.errors.dedup();
    }
    Ok(serde_json::json!({
        "schema_version":1,
        "run_id":ctx.run_id.as_str(),
        "text": if issues.errors.is_empty() {
            if issues.warnings.is_empty() {"docs links passed"} else {"docs links passed with warnings"}
        } else {"docs links failed"},
        "rows":rows,
        "stats": {"internal_links": internal_links, "external_links": external_links},
        "errors":issues.errors,
        "warnings": issues.warnings,
        "capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},
        "options": {"strict": common.strict, "include_drafts": common.include_drafts},
        "external_link_check": {"enabled": common.allow_network, "mode": "disabled_best_effort"}
    }))
}
