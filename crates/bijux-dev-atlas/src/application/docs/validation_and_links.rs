#[derive(Debug, serde::Deserialize)]
struct ExternalLinkAllowlistEntry {
    pattern: String,
}

#[derive(Debug, serde::Deserialize)]
struct ExternalLinkAllowlist {
    entries: Vec<ExternalLinkAllowlistEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct ArtifactLinkAllowlist {
    entries: Vec<ArtifactLinkAllowlistEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct ArtifactLinkAllowlistEntry {
    file: String,
    target: String,
}

fn docs_budget_exempt(rel: &str) -> bool {
    rel.starts_with("_assets/")
        || matches!(rel, "reference/contracts/config-keys.md" | "reference/contracts/errors.md")
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
    let has_navigation_path = yaml
        .get("theme")
        .and_then(|theme| theme.get("features"))
        .and_then(|features| features.as_sequence())
        .map(|rows| {
            rows.iter().any(|row| {
                row.as_str()
                    .map(|value| value.trim() == "navigation.path")
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);
    if !has_navigation_path {
        issues.errors.push(
            "DOCS_NAV_ERROR: mkdocs theme.features must include `navigation.path`".to_string(),
        );
    }
    for required in ["index.md"] {
        if !ctx.docs_root.join(required).exists() {
            issues.errors.push(format!(
                "DOCS_STRUCTURE_ERROR: missing required docs entrypoint `docs/{required}`"
            ));
        }
    }
    let allowed_docs_roots = BTreeSet::from([
        "01-introduction".to_string(),
        "02-getting-started".to_string(),
        "03-user-guide".to_string(),
        "04-operations".to_string(),
        "05-architecture".to_string(),
        "06-development".to_string(),
        "07-reference".to_string(),
        "08-contracts".to_string(),
        "assets".to_string(),
        "index.md".to_string(),
    ]);
    let mut actual_docs_roots = BTreeSet::new();
    for entry in fs::read_dir(&ctx.docs_root)
        .map_err(|e| format!("failed to read {}: {e}", ctx.docs_root.display()))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        actual_docs_roots.insert(name);
    }
    for extra in actual_docs_roots.difference(&allowed_docs_roots) {
        issues.errors.push(format!(
            "DOCS_STRUCTURE_ERROR: docs/ contains forbidden top-level entry `{extra}`"
        ));
    }
    for missing in allowed_docs_roots.difference(&actual_docs_roots) {
        issues.errors.push(format!(
            "DOCS_STRUCTURE_ERROR: docs/ is missing required top-level entry `{missing}`"
        ));
    }
    let mut top_level_dirs = BTreeSet::<String>::new();
    let mut page_counts = BTreeMap::<String, usize>::new();
    for file in docs_markdown_files(&ctx.docs_root, common.include_drafts) {
        let rel = file.strip_prefix(&ctx.docs_root).unwrap_or(&file);
        if let Some(component) = rel.components().next().and_then(|c| c.as_os_str().to_str()) {
            if component != "_drafts" {
                top_level_dirs.insert(component.to_string());
                *page_counts.entry(component.to_string()).or_insert(0) += 1;
            }
        }
    }
    let max_top_level_categories = 13usize;
    if top_level_dirs.len() > max_top_level_categories {
        issues.warnings.push(format!(
            "DOCS_BUDGET_WARN: docs top-level category count {} exceeds budget {}",
            top_level_dirs.len(),
            max_top_level_categories
        ));
    }
    for (category, pages) in page_counts {
        if pages > 40 {
            issues.warnings.push(format!(
                "DOCS_BUDGET_WARN: docs category `{category}` has {pages} pages (soft budget=40)"
            ));
        }
    }
    let site_paths = bijux_dev_atlas::docs::site_output::parse_mkdocs_site_paths(&ctx.repo_root)?;
    let site_dir = ctx.repo_root.join(&site_paths.site_dir);
    if site_dir.exists() {
        for file in walk_files_local(&site_dir) {
            if file.extension().and_then(|v| v.to_str()) == Some("md") {
                issues.errors.push(format!(
                    "DOCS_SITE_DIR_ERROR: site_dir must not contain markdown source files `{}`",
                    file.display()
                ));
            }
        }
    }
    for (_, rel) in mkdocs_nav_refs(&ctx.repo_root)? {
        if !ctx.docs_root.join(&rel).exists() {
            issues.errors.push(format!(
                "DOCS_NAV_ERROR: mkdocs nav references missing file `{rel}`"
            ));
        }
    }
    let requirements_lock = ctx
        .repo_root
        .join("configs/sources/repository/docs/requirements.lock.txt");
    if requirements_lock.exists() {
        let requirements_text = fs::read_to_string(&requirements_lock)
            .map_err(|e| format!("failed to read {}: {e}", requirements_lock.display()))?;
        for package in [
            "mkdocs",
            "mkdocs-material",
            "mkdocs-git-revision-date-localized-plugin",
            "mkdocs-redirects",
        ] {
            let mut matched = false;
            for line in requirements_text.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with(&format!("{package}==")) {
                    matched = true;
                    break;
                }
                if trimmed.starts_with(package) && !trimmed.starts_with(&format!("{package}==")) {
                    issues.errors.push(format!(
                        "DOCS_TOOLCHAIN_PIN_ERROR: `{package}` must be pinned with `==` in configs/sources/repository/docs/requirements.lock.txt"
                    ));
                    matched = true;
                    break;
                }
            }
            if !matched {
                issues.errors.push(format!(
                    "DOCS_TOOLCHAIN_PIN_ERROR: `{package}` missing from configs/sources/repository/docs/requirements.lock.txt"
                ));
            }
        }
    }
    let mut body_hashes = BTreeMap::<String, Vec<String>>::new();
    let mut docs_pages = BTreeSet::<String>::new();
    let mut indexed_links = BTreeSet::<String>::new();
    let link_re = Regex::new(r"\[[^\]]+\]\(([^)]+)\)").map_err(|e| e.to_string())?;
    for file in docs_markdown_files(&ctx.docs_root, common.include_drafts) {
        let rel = file
            .strip_prefix(&ctx.docs_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        docs_pages.insert(rel.clone());
        let depth = rel.split('/').count();
        if depth > 4 {
            issues.errors.push(format!(
                "DOCS_DEPTH_ERROR: `{rel}` depth {depth} exceeds limit 4"
            ));
        }
        let text = fs::read_to_string(&file).unwrap_or_default();
        let frontmatter = extract_frontmatter_map(&text);
        if let Some(stability) = frontmatter.get("stability").map(String::as_str) {
            if !matches!(
                stability,
                "stable" | "experimental" | "deprecated" | "internal"
            ) {
                issues.warnings.push(format!(
                    "DOCS_METADATA_ERROR: `{rel}` has invalid stability `{stability}` (allowed: stable, experimental, deprecated, internal)"
                ));
            }
        }
        if !rel.starts_with("07-reference/") {
            let stem = std::path::Path::new(&rel)
                .file_stem()
                .and_then(|v| v.to_str())
                .unwrap_or_default();
            if stem == "reference"
                || stem.ends_with("-reference")
                || frontmatter
                    .get("type")
                    .is_some_and(|value| value.eq_ignore_ascii_case("reference"))
            {
                issues.warnings.push(format!(
                    "DOCS_REFERENCE_LOCATION_ERROR: reference-like page `{rel}` must live under docs/07-reference/"
                ));
            }
        }
        if rel.ends_with("INDEX.md")
            || rel == "INDEX.md"
            || rel == "index.md"
            || rel.ends_with("/index.md")
        {
            for cap in link_re.captures_iter(&text) {
                let target = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
                if target.starts_with("http://")
                    || target.starts_with("https://")
                    || target.starts_with('#')
                    || target.starts_with("mailto:")
                {
                    continue;
                }
                let target_file = target.split('#').next().unwrap_or_default().trim();
                if target_file.is_empty() {
                    continue;
                }
                let resolved = file.parent().unwrap_or(&ctx.docs_root).join(target_file);
                if let Ok(rel_target) = resolved.strip_prefix(&ctx.docs_root) {
                    indexed_links.insert(rel_target.display().to_string());
                }
            }
        }
        if text.contains("TODO") || text.contains("TBD") {
            issues
                .errors
                .push(format!("DOCS_TODO_ERROR: `{rel}` contains TODO/TBD marker"));
        }
        let line_count = text.lines().count();
        if !docs_budget_exempt(&rel) && line_count > 300 {
            issues.errors.push(format!(
                "DOCS_SIZE_ERROR: `{rel}` has {line_count} lines (budget=300)"
            ));
        }
        let mut max_list_run = 0usize;
        let mut current_list_run = 0usize;
        for line in text.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                current_list_run += 1;
                if current_list_run > max_list_run {
                    max_list_run = current_list_run;
                }
            } else {
                current_list_run = 0;
            }
        }
        if !docs_budget_exempt(&rel) && max_list_run > 40 {
            issues.errors.push(format!(
                "DOCS_LIST_ERROR: `{rel}` has list run of {max_list_run} items (budget=40)"
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
    for page in docs_pages {
        if page == "INDEX.md"
            || page == "index.md"
            || page.ends_with("/INDEX.md")
            || page.starts_with("_drafts/")
            || page.starts_with("_internal/")
            || page.starts_with("_assets/")
        {
            continue;
        }
        if !indexed_links.contains(&page) {
            issues.warnings.push(format!(
                "DOCS_INDEX_ERROR: docs page `{page}` is not linked from any docs/**/INDEX.md"
            ));
        }
    }
    let mut directories = BTreeMap::<String, BTreeSet<String>>::new();
    for rel in docs_markdown_files(&ctx.docs_root, common.include_drafts) {
        let rel = rel
            .strip_prefix(&ctx.docs_root)
            .unwrap_or(&rel)
            .display()
            .to_string();
        if rel.starts_with("_internal/") || rel.starts_with("_drafts/") {
            continue;
        }
        let path = std::path::Path::new(&rel);
        let parent = path
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| ".".to_string());
        let file = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        directories.entry(parent).or_default().insert(file);
    }
    for (dir, files) in directories {
        if !files.contains("index.md") {
            continue;
        }
        let basename = if dir == "." {
            None
        } else {
            std::path::Path::new(&dir)
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| format!("{name}.md"))
        };
        if let Some(candidate) = basename {
            if files.contains(&candidate) {
                issues.warnings.push(format!(
                    "DOCS_DUPLICATE_INDEX_ERROR: `{dir}/index.md` duplicates domain entrypoint `{dir}/{candidate}`"
                ));
            }
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
        "summary": {"total": inv["nav"].as_array().map(|v| v.len()).unwrap_or(0), "errors": inv["errors"].as_array().map(|v| v.len()).unwrap_or(0), "warnings": inv["warnings"].as_array().map(|v| v.len()).unwrap_or(0), "nav_max_depth": nav_max_depth},
        "capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},
        "options": {"strict": common.strict, "include_drafts": common.include_drafts}
    }))
}

fn extract_frontmatter_map(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return out;
    }
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            break;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        let value = value.trim().trim_matches('"').trim_matches('\'');
        out.insert(key.to_string(), value.to_string());
    }
    out
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
    let mkdocs = parse_mkdocs_yaml(&ctx.repo_root)?;
    let site_url = mkdocs
        .get("site_url")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .trim_end_matches('/')
        .to_string();
    let artifact_rewrites = std::collections::BTreeMap::<&str, String>::new();
    let artifact_allowlist_path = ctx
        .repo_root
        .join("configs/sources/repository/docs/artifact-link-allowlist.json");
    let artifact_allowlist: std::collections::BTreeSet<String> = if artifact_allowlist_path.exists()
    {
        let text = fs::read_to_string(&artifact_allowlist_path)
            .map_err(|e| format!("failed to read {}: {e}", artifact_allowlist_path.display()))?;
        let parsed: ArtifactLinkAllowlist = serde_json::from_str(&text)
            .map_err(|e| format!("failed to parse {}: {e}", artifact_allowlist_path.display()))?;
        parsed
            .entries
            .into_iter()
            .map(|entry| format!("{}::{}", entry.file, entry.target))
            .collect()
    } else {
        std::collections::BTreeSet::new()
    };
    let mut autofix_count = 0usize;
    for file in docs_markdown_files(&ctx.docs_root, common.include_drafts) {
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let text = fs::read_to_string(&file).map_err(|e| format!("failed to read {rel}: {e}"))?;
        let anchors = markdown_anchors(&text);
        let mut rewritten_text = text.clone();
        for (idx, line) in text.lines().enumerate() {
            if let Some(include_target_raw) = line
                .trim()
                .strip_prefix("--8<-- \"")
                .and_then(|v| v.strip_suffix('"'))
            {
                let include_target = include_target_raw
                    .strip_prefix("docs/")
                    .unwrap_or(include_target_raw);
                let include_path = ctx.docs_root.join(include_target);
                if !include_path.exists() {
                    issues.errors.push(format!(
                        "DOCS_INCLUDE_ERROR: {rel}:{} include target not found `{include_target}`",
                        idx + 1
                    ));
                } else {
                    let include_text = fs::read_to_string(&include_path).unwrap_or_default();
                    if include_text.contains("](./") {
                        issues.warnings.push(format!(
                            "DOCS_INCLUDE_BASE_WARN: {rel}:{} include `{include_target}` contains `./` links; prefer docs-root-relative paths",
                            idx + 1
                        ));
                    }
                }
            }
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
                    if target.contains("github.com/") && target.contains("/blob/") {
                        issues.warnings.push(format!(
                            "DOCS_GITHUB_BLOB_WARN: {rel}:{} github blob link should be replaced with site-relative docs or governed repo reference `{target}`",
                            idx + 1
                        ));
                    }
                    if !site_url.is_empty() && target.starts_with(&site_url) {
                        issues.warnings.push(format!(
                            "DOCS_SITE_URL_BYPASS_WARN: {rel}:{} absolute site link bypasses docs navigation context `{target}`",
                            idx + 1
                        ));
                    }
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
                if path_part.ends_with(".html") {
                    issues.errors.push(format!(
                        "DOCS_LINK_STYLE_ERROR: {rel}:{} link `{target}` hardcodes .html while use_directory_urls=true",
                        idx + 1
                    ));
                }
                if path_part.starts_with("artifacts/")
                    || path_part.starts_with("../artifacts/")
                    || path_part.starts_with("/artifacts/")
                {
                    let fingerprint = format!("{rel}::{target}");
                    if !artifact_allowlist.contains(&fingerprint) {
                        issues.errors.push(format!(
                            "DOCS_ARTIFACT_LINK_ERROR: {rel}:{} markdown links to artifacts are forbidden `{target}`",
                            idx + 1
                        ));
                    }
                }
                internal_links += 1;
                let resolved = file.parent().unwrap_or(&ctx.docs_root).join(path_part);
                if path_part.ends_with(".md")
                    && !resolved.starts_with(&ctx.docs_root)
                    && !path_part.starts_with("http://")
                    && !path_part.starts_with("https://")
                {
                    issues.errors.push(format!(
                        "DOCS_EXTERNAL_MD_ERROR: {rel}:{} markdown link points outside docs_dir `{target}`",
                        idx + 1
                    ));
                }
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
                    let message = format!(
                        "DOCS_LINK_ERROR: {rel}:{} unresolved link `{target}`",
                        idx + 1
                    );
                    issues.errors.push(message);
                }
                rows.push(
                    serde_json::json!({"file": rel, "line": idx + 1, "target": target, "ok": ok}),
                );
            }
        }
        if common.allow_write {
            for (from, to) in &artifact_rewrites {
                if rewritten_text.contains(from) {
                    rewritten_text = rewritten_text.replace(from, to);
                    autofix_count += 1;
                }
            }
            if rewritten_text != text {
                fs::write(&file, rewritten_text)
                    .map_err(|e| format!("failed to write {}: {e}", file.display()))?;
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
        "autofix": {"enabled": common.allow_write, "rewrites_applied": autofix_count},
        "errors":issues.errors,
        "warnings": issues.warnings,
        "capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},
        "options": {"strict": common.strict, "include_drafts": common.include_drafts},
        "external_link_check": {"enabled": common.allow_network, "mode": "disabled_best_effort"}
    }))
}

fn docs_external_targets(
    docs_root: &std::path::Path,
    include_drafts: bool,
) -> Result<BTreeMap<String, Vec<String>>, String> {
    let link_re = Regex::new(r"\[[^\]]+\]\(([^)]+)\)").map_err(|e| e.to_string())?;
    let image_re = Regex::new(r"!\[[^\]]*\]\(([^)]+)\)").map_err(|e| e.to_string())?;
    let mut seen = BTreeMap::<String, Vec<String>>::new();
    for file in docs_markdown_files(docs_root, include_drafts) {
        let rel = file
            .strip_prefix(docs_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let text = fs::read_to_string(&file).map_err(|e| format!("failed to read {rel}: {e}"))?;
        for regex in [&link_re, &image_re] {
            for cap in regex.captures_iter(&text) {
                let target = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
                let cleaned = target.split('#').next().unwrap_or("").trim();
                if cleaned.starts_with("http://") || cleaned.starts_with("https://") {
                    seen.entry(cleaned.to_string())
                        .or_default()
                        .push(rel.clone());
                }
            }
        }
    }
    for refs in seen.values_mut() {
        refs.sort();
        refs.dedup();
    }
    Ok(seen)
}

fn docs_external_link_allowlist(
    repo_root: &std::path::Path,
    path: &std::path::Path,
) -> Result<Vec<String>, String> {
    let allowlist_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    };
    let text = fs::read_to_string(&allowlist_path)
        .map_err(|e| format!("failed to read {}: {e}", allowlist_path.display()))?;
    let payload: ExternalLinkAllowlist =
        serde_json::from_str(&text).map_err(|e| format!("invalid allowlist json: {e}"))?;
    Ok(payload
        .entries
        .into_iter()
        .map(|entry| entry.pattern)
        .collect())
}

fn docs_probe_external_link(url: &str) -> Result<(bool, String), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| format!("client build failed: {e}"))?;
    match client.head(url).send() {
        Ok(response) => Ok((
            response.status().is_success(),
            format!("status {}", response.status().as_u16()),
        )),
        Err(head_err) => {
            let response = client
                .get(url)
                .send()
                .map_err(|get_err| format!("HEAD failed: {head_err}; GET failed: {get_err}"))?;
            Ok((
                response.status().is_success(),
                format!("status {}", response.status().as_u16()),
            ))
        }
    }
}

pub(crate) fn docs_external_links_payload(
    ctx: &DocsContext,
    common: &DocsCommonArgs,
    allowlist_path: &std::path::Path,
) -> Result<serde_json::Value, String> {
    if !common.allow_network {
        return Err("docs external-links requires --allow-network".to_string());
    }
    let allowlist = docs_external_link_allowlist(&ctx.repo_root, allowlist_path)?;
    let seen = docs_external_targets(&ctx.docs_root, common.include_drafts)?;
    let mut rows = Vec::<serde_json::Value>::new();
    let mut errors = Vec::<String>::new();
    for (target, refs) in seen {
        let allowlisted = allowlist.iter().any(|pattern| target.starts_with(pattern));
        let (ok, detail) = if allowlisted {
            (true, "allowlisted".to_string())
        } else {
            docs_probe_external_link(&target)?
        };
        if !ok {
            let refs_text = refs.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
            errors.push(format!(
                "DOCS_EXTERNAL_LINK_ERROR: {target} failed external link check ({detail}); referenced from {refs_text}"
            ));
        }
        rows.push(serde_json::json!({
            "target": target,
            "allowlisted": allowlisted,
            "ok": ok,
            "detail": detail,
            "references": refs
        }));
    }
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": if errors.is_empty() { "docs external links passed" } else { "docs external links failed" },
        "rows": rows,
        "errors": errors,
        "warnings": [],
        "capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},
        "options": {"strict": common.strict, "include_drafts": common.include_drafts, "allowlist": allowlist_path.display().to_string()}
    }))
}
