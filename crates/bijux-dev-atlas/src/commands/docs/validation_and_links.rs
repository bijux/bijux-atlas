#[derive(Debug, serde::Deserialize)]
struct ExternalLinkAllowlistEntry {
    pattern: String,
}

#[derive(Debug, serde::Deserialize)]
struct ExternalLinkAllowlist {
    entries: Vec<ExternalLinkAllowlistEntry>,
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
    for required in ["INDEX.md", "START_HERE.md"] {
        if !ctx.docs_root.join(required).exists() {
            issues.errors.push(format!(
                "DOCS_STRUCTURE_ERROR: missing required docs entrypoint `docs/{required}`"
            ));
        }
    }
    let mut top_level_dirs = BTreeSet::<String>::new();
    let mut page_counts = BTreeMap::<String, usize>::new();
    for file in docs_markdown_files(&ctx.docs_root, common.include_drafts) {
        let rel = file.strip_prefix(&ctx.docs_root).unwrap_or(&file);
        if let Some(component) = rel.components().next().and_then(|c| c.as_os_str().to_str()) {
            if component != "_generated" && component != "_drafts" {
                top_level_dirs.insert(component.to_string());
                *page_counts.entry(component.to_string()).or_insert(0) += 1;
            }
        }
    }
    if top_level_dirs.len() > 8 {
        issues.warnings.push(format!(
            "DOCS_BUDGET_WARN: docs top-level category count {} exceeds budget 8",
            top_level_dirs.len()
        ));
    }
    for (category, pages) in page_counts {
        if pages > 40 {
            issues.warnings.push(format!(
                "DOCS_BUDGET_WARN: docs category `{category}` has {pages} pages (soft budget=40)"
            ));
        }
    }
    for (_, rel) in mkdocs_nav_refs(&ctx.repo_root)? {
        if !ctx.docs_root.join(&rel).exists() {
            issues.errors.push(format!(
                "DOCS_NAV_ERROR: mkdocs nav references missing file `{rel}`"
            ));
        }
    }
    for rel in [
        "reference/commands.md",
        "reference/schemas.md",
        "reference/configs.md",
        "reference/make-targets.md",
    ] {
        if !ctx.docs_root.join(rel).exists() {
            issues.errors.push(format!(
                "DOCS_REFERENCE_ERROR: missing generated docs reference page `docs/{rel}`"
            ));
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
        if rel.ends_with("INDEX.md") || rel == "INDEX.md" || rel == "index.md" {
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
            issues.errors.push(format!(
                "DOCS_TODO_ERROR: `{rel}` contains TODO/TBD marker"
            ));
        }
        let line_count = text.lines().count();
        if line_count > 300 {
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
        if max_list_run > 40 {
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
            || page.starts_with("_generated/")
            || page.starts_with("_drafts/")
        {
            continue;
        }
        if !indexed_links.contains(&page) {
            issues.errors.push(format!(
                "DOCS_INDEX_ERROR: docs page `{page}` is not linked from any docs/**/INDEX.md"
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
                    seen.entry(cleaned.to_string()).or_default().push(rel.clone());
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
    Ok(payload.entries.into_iter().map(|entry| entry.pattern).collect())
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
