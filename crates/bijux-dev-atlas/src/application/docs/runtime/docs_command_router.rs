use std::io::{self, Write};

fn current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn docs_gate_artifact_dir(ctx: &DocsContext) -> std::path::PathBuf {
    ctx.artifacts_root
        .join("run")
        .join(ctx.run_id.as_str())
        .join("gates")
        .join("docs")
}

fn write_docs_gate_artifact(
    path: &std::path::Path,
    payload: &serde_json::Value,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(payload).map_err(|e| format!("encode failed: {e}"))?,
    )
    .map_err(|e| format!("write {} failed: {e}", path.display()))
}

#[derive(Debug, Clone, serde::Deserialize)]
struct DocsRealDataCatalog {
    runs: Vec<DocsRealDataRun>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct DocsRealDataRun {
    id: String,
    run_label: String,
    dataset: String,
    dataset_size_tier: String,
    ingest_mode: String,
    expected_artifacts: Vec<String>,
    expected_query_set: Vec<DocsRealDataQuery>,
    input_provenance: DocsRealDataProvenance,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct DocsRealDataQuery {
    name: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct DocsRealDataProvenance {
    url: String,
    retrieval_method: String,
    license_note: String,
}

fn docs_sync_redirects(repo_root: &std::path::Path) -> Result<serde_json::Value, String> {
    let mkdocs_path = repo_root.join("mkdocs.yml");
    let redirects_path = repo_root.join("docs/redirects.json");
    let redirect_registry_path = repo_root.join("docs/_internal/governance/redirect-registry.json");
    let legacy_inventory_path = repo_root.join("docs/_internal/generated/legacy-url-inventory.md");
    let start = "      # redirect_maps generated from docs/redirects.json; run bijux-dev-atlas docs redirects sync --allow-write";
    let end = "      # end generated redirect_maps";

    let mapping: serde_json::Map<String, serde_json::Value> = serde_json::from_str(
        &fs::read_to_string(&redirects_path)
            .map_err(|e| format!("read {} failed: {e}", redirects_path.display()))?,
    )
    .map_err(|e| format!("parse {} failed: {e}", redirects_path.display()))?;

    let filtered = mapping
        .into_iter()
        .filter_map(|(key, value)| {
            let value = value.as_str()?.to_string();
            if key.ends_with(".md") && value.ends_with(".md") {
                Some((key, value))
            } else {
                None
            }
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    let redirect_registry: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&redirect_registry_path)
            .map_err(|e| format!("read {} failed: {e}", redirect_registry_path.display()))?,
    )
    .map_err(|e| format!("parse {} failed: {e}", redirect_registry_path.display()))?;

    let mut block = String::from(start);
    block.push('\n');
    block.push_str("      redirect_maps:\n");
    for (key, value) in &filtered {
        let key = key
            .strip_prefix("docs/")
            .ok_or_else(|| format!("redirect key must start with docs/: {key}"))?;
        let value = value
            .strip_prefix("docs/")
            .ok_or_else(|| format!("redirect value must start with docs/: {value}"))?;
        block.push_str(&format!("        {key}: {value}\n"));
    }
    block.push_str(end);

    let mkdocs_text =
        fs::read_to_string(&mkdocs_path).map_err(|e| format!("read {} failed: {e}", mkdocs_path.display()))?;
    let start_index = mkdocs_text
        .find(start)
        .ok_or_else(|| format!("{} missing redirect start marker", mkdocs_path.display()))?;
    let end_marker_index = mkdocs_text[start_index..]
        .find(end)
        .map(|idx| start_index + idx)
        .ok_or_else(|| format!("{} missing redirect end marker", mkdocs_path.display()))?;
    let end_line_index = mkdocs_text[end_marker_index..]
        .find('\n')
        .map(|idx| end_marker_index + idx)
        .unwrap_or(mkdocs_text.len());
    let new_text = format!(
        "{}{}{}",
        &mkdocs_text[..start_index],
        block,
        &mkdocs_text[end_line_index..]
    );
    let changed = new_text != mkdocs_text;
    fs::write(&mkdocs_path, new_text)
        .map_err(|e| format!("write {} failed: {e}", mkdocs_path.display()))?;

    let legacy_inventory = render_legacy_url_inventory_markdown(&filtered, &redirect_registry);
    fs::write(&legacy_inventory_path, legacy_inventory)
        .map_err(|e| format!("write {} failed: {e}", legacy_inventory_path.display()))?;

    Ok(serde_json::json!({
        "schema_version": 1,
        "status": "ok",
        "text": "mkdocs redirect_maps synchronized",
        "changed": changed,
        "redirect_count": filtered.len(),
        "legacy_inventory": legacy_inventory_path.display().to_string(),
        "output": mkdocs_path.display().to_string(),
    }))
}

fn render_legacy_url_inventory_markdown(
    redirects: &std::collections::BTreeMap<String, String>,
    registry: &serde_json::Value,
) -> String {
    fn match_rule<'a>(source: &str, rules: &'a [serde_json::Value]) -> Option<&'a serde_json::Value> {
        for rule in rules {
            if let Some(path) = rule.get("matchPath").and_then(|value| value.as_str()) {
                if source == path {
                    return Some(rule);
                }
            }
            if let Some(prefix) = rule.get("matchPrefix").and_then(|value| value.as_str()) {
                if source.starts_with(prefix) {
                    return Some(rule);
                }
            }
        }
        None
    }

    let rules = registry["rules"].as_array().cloned().unwrap_or_default();
    let mut out = String::from("# Legacy URL Inventory\n\n");
    out.push_str("- Generated by: `bijux-dev-atlas docs redirects sync --allow-write`\n");
    out.push_str("- Do not edit by hand: regenerate with the control-plane command.\n\n");
    out.push_str("| Legacy Path | Current Path | Owner | Reason | Temporary Until |\n");
    out.push_str("| --- | --- | --- | --- | --- |\n");
    for (source, target) in redirects {
        let rule = match_rule(source, &rules).cloned().unwrap_or(serde_json::json!({}));
        let owner = rule["owner"].as_str().unwrap_or("unassigned");
        let reason = rule["reason"].as_str().unwrap_or("missing redirect metadata");
        let expiry = rule["expiresOn"].as_str().unwrap_or("none");
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} | `{}` |\n",
            source, target, owner, reason, expiry
        ));
    }
    out
}

fn render_summary_table(
    rows: &[serde_json::Value],
    title: &str,
    empty_row: &str,
    line_builder: impl Fn(&serde_json::Value) -> String,
) -> String {
    let mut lines = vec![
        format!("# {title}"),
        String::new(),
    ];
    if rows.is_empty() {
        lines.push(empty_row.to_string());
    } else {
        for row in rows {
            lines.push(line_builder(row));
        }
    }
    lines.push(String::new());
    lines.join("\n")
}

fn docs_write_summary_table(
    input: &std::path::Path,
    output: &std::path::Path,
    field: &str,
    content: impl Fn(Vec<serde_json::Value>) -> String,
) -> Result<serde_json::Value, String> {
    let rows = if input.exists() {
        let payload: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(input).map_err(|e| format!("read {} failed: {e}", input.display()))?,
        )
        .map_err(|e| format!("parse {} failed: {e}", input.display()))?;
        payload[field].as_array().cloned().unwrap_or_default()
    } else {
        Vec::new()
    };
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {} failed: {e}", parent.display()))?;
    }
    fs::write(output, content(rows.clone()))
        .map_err(|e| format!("write {} failed: {e}", output.display()))?;
    Ok(serde_json::json!({
        "schema_version": 1,
        "status": "ok",
        "input": input.display().to_string(),
        "output": output.display().to_string(),
        "row_count": rows.len(),
    }))
}

fn docs_spine_path(repo_root: &std::path::Path) -> std::path::PathBuf {
    repo_root.join("docs/_internal/governance/docs-spine.md")
}

fn parse_spine_pages(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let path = docs_spine_path(repo_root);
    let text =
        fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let mut pages = Vec::<String>::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- `docs/") && trimmed.ends_with('`') {
            pages.push(
                trimmed
                    .trim_start_matches("- `")
                    .trim_end_matches('`')
                    .to_string(),
            );
        }
    }
    if pages.is_empty() {
        return Err(format!(
            "{} must list spine pages using `- `docs/...`` entries",
            path.display()
        ));
    }
    Ok(pages)
}

fn docs_spine_report_payload(ctx: &DocsContext, common: &DocsCommonArgs) -> Result<serde_json::Value, String> {
    let spine_pages = parse_spine_pages(&ctx.repo_root)?;
    let docs_root = ctx.repo_root.join("docs");
    let markdown_files = docs_markdown_files(&docs_root, common.include_drafts);
    let link_re = regex::Regex::new(r"\[[^\]]+\]\(([^)]+)\)").map_err(|e| e.to_string())?;
    let mut linked = std::collections::BTreeSet::<String>::new();
    let mut pages = std::collections::BTreeSet::<String>::new();

    for file in &markdown_files {
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(file)
            .display()
            .to_string();
        if rel.starts_with("docs/_internal/") || rel.starts_with("docs/_assets/") {
            continue;
        }
        pages.insert(rel.clone());
        let text =
            fs::read_to_string(file).map_err(|e| format!("failed to read {}: {e}", file.display()))?;
        for capture in link_re.captures_iter(&text) {
            let target = capture.get(1).map(|m| m.as_str()).unwrap_or_default();
            if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
                || target.starts_with('#')
            {
                continue;
            }
            let target_path = target.split('#').next().unwrap_or_default().trim();
            if target_path.is_empty() {
                continue;
            }
            let resolved = file.parent().unwrap_or(&docs_root).join(target_path);
            if let Ok(rel_target) = resolved.strip_prefix(&ctx.repo_root) {
                linked.insert(rel_target.display().to_string());
            }
        }
    }

    let mut orphans = Vec::<String>::new();
    for page in pages {
        if spine_pages.iter().any(|spine| spine == &page) {
            continue;
        }
        if page == "docs/index.md" || page == "docs/start-here.md" || page.ends_with("/index.md") {
            continue;
        }
        if !linked.contains(&page) {
            orphans.push(page);
        }
    }
    orphans.sort();

    let generated_path = ctx.repo_root.join("docs/_internal/generated/orphan-pages.md");
    if common.allow_write {
        if let Some(parent) = generated_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
        }
        let mut markdown = String::from("# Orphan Pages\n\n");
        markdown.push_str("- Generated by: `bijux-dev-atlas docs spine report --allow-write`\n\n");
        if orphans.is_empty() {
            markdown.push_str("- None\n");
        } else {
            for orphan in &orphans {
                markdown.push_str(&format!("- `{orphan}`\n"));
            }
        }
        fs::write(&generated_path, markdown)
            .map_err(|e| format!("failed to write {}: {e}", generated_path.display()))?;
    }

    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": if orphans.is_empty() { "docs spine report clean" } else { "docs spine report found orphan pages" },
        "spine_pages": spine_pages,
        "orphans": orphans,
        "generated_report": generated_path.display().to_string()
    }))
}

fn docs_spine_validate_payload(
    ctx: &DocsContext,
    common: &DocsCommonArgs,
) -> Result<serde_json::Value, String> {
    let spine_pages = parse_spine_pages(&ctx.repo_root)?;
    let mut errors = Vec::<String>::new();
    let mut checks = Vec::<serde_json::Value>::new();
    for page in &spine_pages {
        let exists = ctx.repo_root.join(page).is_file();
        checks.push(serde_json::json!({"id":"SPINE-001","page":page,"pass":exists}));
        if !exists {
            errors.push(format!("spine page missing: {page}"));
        }
    }
    let max_top_level_categories = 13usize;
    let top_levels = mkdocs_nav_refs(&ctx.repo_root)?
        .into_iter()
        .filter_map(|(_, rel)| rel.split('/').next().map(str::to_string))
        .collect::<std::collections::BTreeSet<_>>();
    checks.push(serde_json::json!({
        "id": "SPINE-002",
        "pass": top_levels.len() <= max_top_level_categories,
        "top_level_categories": top_levels.len(),
        "max_top_level_categories": max_top_level_categories
    }));
    if top_levels.len() > max_top_level_categories {
        errors.push(format!(
            "top-level navigation categories {} exceeds maximum {}",
            top_levels.len(),
            max_top_level_categories
        ));
    }
    let report = docs_spine_report_payload(ctx, common)?;
    let orphan_count = report["orphans"].as_array().map(|v| v.len()).unwrap_or(0);
    checks.push(serde_json::json!({"id":"SPINE-003","pass": orphan_count == 0, "orphan_count": orphan_count}));
    if orphan_count > 0 {
        errors.push(format!("orphan pages detected: {orphan_count}"));
    }
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": if errors.is_empty() { "docs spine validate passed" } else { "docs spine validate failed" },
        "errors": errors,
        "checks": checks,
        "spine_pages": spine_pages
    }))
}

fn build_docs_link_graph(
    ctx: &DocsContext,
    common: &DocsCommonArgs,
) -> Result<std::collections::BTreeMap<String, usize>, String> {
    let docs_root = ctx.repo_root.join("docs");
    let markdown_files = docs_markdown_files(&docs_root, common.include_drafts);
    let link_re = regex::Regex::new(r"\[[^\]]+\]\(([^)]+)\)").map_err(|e| e.to_string())?;
    let mut inbound = std::collections::BTreeMap::<String, usize>::new();
    for file in &markdown_files {
        let rel = file
            .strip_prefix(&docs_root)
            .unwrap_or(file)
            .display()
            .to_string();
        if rel.starts_with("_internal/") || rel.starts_with("_assets/") {
            continue;
        }
        inbound.entry(rel).or_insert(0);
    }
    for file in &markdown_files {
        let src_rel = file
            .strip_prefix(&docs_root)
            .unwrap_or(file)
            .display()
            .to_string();
        if src_rel.starts_with("_internal/") || src_rel.starts_with("_assets/") {
            continue;
        }
        let text =
            fs::read_to_string(file).map_err(|e| format!("failed to read {}: {e}", file.display()))?;
        for capture in link_re.captures_iter(&text) {
            let target = capture.get(1).map(|m| m.as_str()).unwrap_or_default();
            if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
                || target.starts_with('#')
            {
                continue;
            }
            let target_path = target.split('#').next().unwrap_or_default().trim();
            if target_path.is_empty() {
                continue;
            }
            let resolved = file.parent().unwrap_or(&docs_root).join(target_path);
            if let Ok(rel_target) = resolved.strip_prefix(&docs_root) {
                let mut normalized = rel_target.display().to_string();
                if rel_target.extension().is_none() {
                    let index = rel_target.join("index.md");
                    if docs_root.join(&index).exists() {
                        normalized = index.display().to_string();
                    }
                }
                if let Some(count) = inbound.get_mut(&normalized) {
                    *count += 1;
                }
            }
        }
    }
    Ok(inbound)
}

fn docs_graph_payload(ctx: &DocsContext, common: &DocsCommonArgs) -> Result<serde_json::Value, String> {
    let inbound = build_docs_link_graph(ctx, common)?;
    let rows = inbound
        .iter()
        .map(|(path, count)| serde_json::json!({"path": path, "inbound_links": count}))
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": "docs graph",
        "rows": rows
    }))
}

fn docs_top_payload(
    ctx: &DocsContext,
    common: &DocsCommonArgs,
    limit: usize,
) -> Result<serde_json::Value, String> {
    let inbound = build_docs_link_graph(ctx, common)?;
    let mut rows = inbound
        .into_iter()
        .map(|(path, count)| (count, path))
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| b.cmp(a));
    let capped = rows
        .into_iter()
        .take(limit)
        .map(|(count, path)| serde_json::json!({"path": path, "inbound_links": count}))
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": "docs top",
        "limit": limit,
        "rows": capped
    }))
}

fn docs_dead_payload(ctx: &DocsContext, common: &DocsCommonArgs) -> Result<serde_json::Value, String> {
    let inbound = build_docs_link_graph(ctx, common)?;
    let rows = inbound
        .into_iter()
        .filter(|(path, count)| {
            *count == 0 && path != "index.md" && path != "start-here.md" && !path.ends_with("/index.md")
        })
        .map(|(path, count)| serde_json::json!({"path": path, "inbound_links": count}))
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": "docs dead pages",
        "rows": rows
    }))
}

fn docs_duplicates_payload(ctx: &DocsContext, common: &DocsCommonArgs) -> Result<serde_json::Value, String> {
    let docs_root = ctx.repo_root.join("docs");
    let mut heading_map = std::collections::BTreeMap::<String, Vec<String>>::new();
    for file in docs_markdown_files(&docs_root, common.include_drafts) {
        let rel = file
            .strip_prefix(&docs_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        if rel.starts_with("_internal/") || rel.starts_with("_assets/") {
            continue;
        }
        let text = fs::read_to_string(&file).unwrap_or_default();
        let key = text
            .lines()
            .filter_map(|line| line.trim_start().strip_prefix('#').map(str::trim))
            .filter(|line| !line.is_empty())
            .map(|line| line.to_lowercase())
            .take(8)
            .collect::<Vec<_>>()
            .join("|");
        if !key.is_empty() {
            heading_map.entry(key).or_default().push(rel);
        }
    }
    let mut rows = Vec::<serde_json::Value>::new();
    for (signature, pages) in heading_map {
        if pages.len() > 1 {
            rows.push(serde_json::json!({"signature": signature, "pages": pages}));
        }
    }
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": "docs duplicates",
        "rows": rows
    }))
}

fn docs_merge_validate_payload(
    ctx: &DocsContext,
    _common: &DocsCommonArgs,
) -> Result<serde_json::Value, String> {
    let redirects_path = ctx.repo_root.join("docs/redirects.json");
    let redirects_text = fs::read_to_string(&redirects_path)
        .map_err(|e| format!("failed to read {}: {e}", redirects_path.display()))?;
    let redirects: std::collections::BTreeMap<String, String> = serde_json::from_str(&redirects_text)
        .map_err(|e| format!("failed to parse {}: {e}", redirects_path.display()))?;
    let merge_plan_path = ctx
        .repo_root
        .join("docs/_internal/governance/docs-merge-plan.md");
    let merge_plan_text = fs::read_to_string(&merge_plan_path)
        .map_err(|e| format!("failed to read {}: {e}", merge_plan_path.display()))?;
    let mut source_paths = Vec::<String>::new();
    for line in merge_plan_text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let cols = trimmed
            .split('|')
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();
        if cols.len() < 3 || cols[0] == "Source page" {
            continue;
        }
        for token in cols[0].split('+') {
            let value = token.trim().trim_matches('`');
            if value.starts_with("docs/") {
                source_paths.push(value.to_string());
            }
        }
    }
    let mut issues = Vec::<String>::new();
    let mut checks = Vec::<serde_json::Value>::new();
    source_paths.sort();
    source_paths.dedup();
    for source in source_paths {
        let source_exists = ctx.repo_root.join(&source).is_file();
        let redirected = redirects.contains_key(&source);
        if source_exists && !redirected {
            issues.push(format!(
                "merge source exists without redirect mapping: {source}"
            ));
        }
        checks.push(serde_json::json!({
            "source": source,
            "source_exists": source_exists,
            "redirected": redirected
        }));
    }
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": if issues.is_empty() { "docs merge validate passed" } else { "docs merge validate failed" },
        "errors": issues,
        "checks": checks
    }))
}

fn docs_includes_check_payload(ctx: &DocsContext, common: &DocsCommonArgs) -> Result<serde_json::Value, String> {
    let docs_root = ctx.repo_root.join("docs");
    let markdown_files = docs_markdown_files(&docs_root, common.include_drafts);
    let include_re = regex::Regex::new(r#"--8<--\s*"([^"]+)""#).map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    let mut errors = Vec::new();
    for file in markdown_files {
        let rel = file
            .strip_prefix(&docs_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let text = fs::read_to_string(&file)
            .map_err(|e| format!("failed to read {}: {e}", file.display()))?;
        for cap in include_re.captures_iter(&text) {
            let include_target = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
            let include_path = docs_root.join(include_target);
            let exists = include_path.exists();
            if !exists {
                errors.push(format!(
                    "include target missing: `{include_target}` referenced from `docs/{rel}`"
                ));
            }
            rows.push(serde_json::json!({
                "source": format!("docs/{rel}"),
                "include_target": include_target,
                "exists": exists
            }));
        }
    }
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": if errors.is_empty() { "docs includes check passed" } else { "docs includes check failed" },
        "rows": rows,
        "errors": errors
    }))
}

fn docs_nav_integrity_payload(ctx: &DocsContext) -> Result<serde_json::Value, String> {
    let nav_refs = mkdocs_nav_refs(&ctx.repo_root)?;
    let mut errors = Vec::new();
    let mut rows = Vec::new();
    for (label, rel) in nav_refs {
        let path = ctx.repo_root.join("docs").join(&rel);
        let exists = path.is_file();
        if !exists {
            errors.push(format!("mkdocs nav entry `{label}` points to missing file `docs/{rel}`"));
        }
        rows.push(serde_json::json!({
            "label": label,
            "path": format!("docs/{rel}"),
            "exists": exists
        }));
    }
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": if errors.is_empty() { "docs nav integrity passed" } else { "docs nav integrity failed" },
        "rows": rows,
        "errors": errors
    }))
}

fn docs_generate_health_dashboard(repo_root: &std::path::Path) -> Result<serde_json::Value, String> {
    let docs_root = repo_root.join("docs");
    let output_path = docs_root.join("_internal/generated/docs-health-dashboard.md");
    let allowlist_path = repo_root.join("configs/sources/repository/docs/external-link-allowlist.json");
    let allowlist: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&allowlist_path)
            .map_err(|e| format!("read {} failed: {e}", allowlist_path.display()))?,
    )
    .map_err(|e| format!("parse {} failed: {e}", allowlist_path.display()))?;
    let allowed_domains = allowlist["allowed_domains"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let link_re = regex::Regex::new(r"\[[^\]]+\]\(([^)]+)\)").map_err(|e| e.to_string())?;
    let orphan_allowlist_path = docs_root.join("_internal/policies/orphan-allowlist.json");
    let orphan_allowlist = load_orphan_allowlist(&orphan_allowlist_path)?;

    let mut docs_files = docs_markdown_files(&docs_root, true);
    docs_files.sort();
    let mut external_urls = Vec::<String>::new();
    let mut linked_local = std::collections::BTreeSet::<String>::new();
    let mut missing_verified = Vec::<String>::new();
    let mut page_lengths = Vec::<(usize, String)>::new();
    let mut single_line_offenders = Vec::<(usize, String)>::new();

    for path in &docs_files {
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(path)
            .display()
            .to_string();
        let text = fs::read_to_string(path).map_err(|e| format!("read {} failed: {e}", path.display()))?;
        let lines = text.lines().collect::<Vec<_>>();
        if !text.contains("Last verified against:") {
            missing_verified.push(rel.clone());
        }
        page_lengths.push((lines.len(), rel.clone()));
        let longest_line = lines.iter().map(|line| line.len()).max().unwrap_or(0);
        if longest_line > 140 {
            single_line_offenders.push((longest_line, rel.clone()));
        }
        for capture in link_re.captures_iter(&text) {
            let target = capture.get(1).map(|m| m.as_str()).unwrap_or_default();
            if target.starts_with("http://") || target.starts_with("https://") {
                external_urls.push(target.to_string());
                continue;
            }
            if target.starts_with('#') || target.starts_with("mailto:") {
                continue;
            }
            let target_path = target.split('#').next().unwrap_or_default().trim();
            if target_path.is_empty() {
                continue;
            }
            let resolved = path.parent().unwrap_or(&docs_root).join(target_path);
            if let Ok(rel_target) = resolved.strip_prefix(&docs_root) {
                let mut normalized = rel_target.display().to_string();
                if rel_target.extension().is_none() {
                    let index = rel_target.join("index.md");
                    if docs_root.join(&index).exists() {
                        normalized = index.display().to_string();
                    }
                }
                linked_local.insert(normalized);
            }
        }
    }

    let mut orphan_pages = Vec::<String>::new();
    let mut allowlisted_orphans = Vec::<String>::new();
    let mut orphan_allowlist_expired = Vec::<String>::new();
    let today = std::env::var_os("BIJUX_DOCS_ORPHAN_AUDIT_DATE")
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "9999-12-31".to_string());
    for path in &docs_files {
        let rel = path
            .strip_prefix(&docs_root)
            .unwrap_or(path)
            .display()
            .to_string();
        if rel.starts_with("_internal/")
            || rel == "index.md"
            || rel.ends_with("/index.md")
            || rel == "start-here.md"
        {
            continue;
        }
        if !linked_local.contains(&rel) {
            let full_path = format!("docs/{rel}");
            let entry = orphan_allowlist.iter().find(|item| item.path == full_path);
            match entry {
                Some(allowlisted) => {
                    if allowlisted.expires_on >= today {
                        allowlisted_orphans.push(full_path);
                    } else {
                        orphan_allowlist_expired.push(format!(
                            "{} (expired {}, owner {})",
                            allowlisted.path, allowlisted.expires_on, allowlisted.owner
                        ));
                        orphan_pages.push(allowlisted.path.clone());
                    }
                }
                None => orphan_pages.push(full_path),
            }
        }
    }

    page_lengths.sort_by(|a, b| b.cmp(a));
    single_line_offenders.sort_by(|a, b| b.cmp(a));
    let domains = external_urls
        .iter()
        .filter_map(|url| {
            let parsed = url.split("://").nth(1)?;
            Some(parsed.split('/').next().unwrap_or_default().to_string())
        })
        .collect::<Vec<_>>();
    let unique_domains = domains
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let covered_domains = unique_domains
        .iter()
        .filter(|domain| allowed_domains.contains(domain.as_str()))
        .count();

    let mut lines = vec![
        "# Docs Health Dashboard".to_string(),
        String::new(),
        "- Owner: `docs`".to_string(),
        "- Review cadence: `quarterly`".to_string(),
        "- Type: `generated`".to_string(),
        "- Audience: `contributor`".to_string(),
        "- Stability: `stable`".to_string(),
        "- Generated by: `bijux-dev-atlas docs health-dashboard --allow-write`".to_string(),
        "- Do not edit by hand: regenerate with the control-plane command.".to_string(),
        String::new(),
        "## Summary".to_string(),
        String::new(),
        format!("- External links: `{}`", external_urls.len()),
        format!("- Unique external domains: `{}`", unique_domains.len()),
        format!(
            "- Allowlist-covered external domains: `{covered_domains}/{}`",
            unique_domains.len()
        ),
        format!("- Orphan pages: `{}`", orphan_pages.len()),
        format!("- Allowlisted orphan pages: `{}`", allowlisted_orphans.len()),
        format!(
            "- Expired orphan allowlist entries: `{}`",
            orphan_allowlist_expired.len()
        ),
        format!(
            "- Pages missing `Last verified against`: `{}`",
            missing_verified.len()
        ),
        String::new(),
        "## Longest Pages".to_string(),
        String::new(),
    ];
    for (length, path) in page_lengths.iter().take(10) {
        lines.push(format!("- `{path}`: `{length}` lines"));
    }
    lines.push(String::new());
    lines.push("## Single-Line Offenders".to_string());
    lines.push(String::new());
    if single_line_offenders.is_empty() {
        lines.push("- None".to_string());
    } else {
        for (length, path) in single_line_offenders.iter().take(10) {
            lines.push(format!("- `{path}`: longest line `{length}` characters"));
        }
    }
    lines.push(String::new());
    lines.push("## Missing Verification Markers".to_string());
    lines.push(String::new());
    if missing_verified.is_empty() {
        lines.push("- None".to_string());
    } else {
        for path in missing_verified.iter().take(20) {
            lines.push(format!("- `{path}`"));
        }
    }
    lines.push(String::new());
    lines.push("## Orphan Pages".to_string());
    lines.push(String::new());
    if orphan_pages.is_empty() {
        lines.push("- None".to_string());
    } else {
        for path in orphan_pages.iter().take(20) {
            lines.push(format!("- `{path}`"));
        }
    }
    lines.push(String::new());
    lines.push("## Allowlisted Orphan Pages".to_string());
    lines.push(String::new());
    if allowlisted_orphans.is_empty() {
        lines.push("- None".to_string());
    } else {
        for path in allowlisted_orphans.iter().take(20) {
            lines.push(format!("- `{path}`"));
        }
    }
    lines.push(String::new());
    lines.push("## Expired Orphan Allowlist Entries".to_string());
    lines.push(String::new());
    if orphan_allowlist_expired.is_empty() {
        lines.push("- None".to_string());
    } else {
        for row in orphan_allowlist_expired.iter().take(20) {
            lines.push(format!("- `{row}`"));
        }
    }
    lines.push(String::new());

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {} failed: {e}", parent.display()))?;
    }
    fs::write(&output_path, lines.join("\n"))
        .map_err(|e| format!("write {} failed: {e}", output_path.display()))?;
    Ok(serde_json::json!({
        "schema_version": 1,
        "status": "ok",
        "text": "docs health dashboard generated",
        "output": output_path.display().to_string(),
        "external_links": external_urls.len(),
        "orphan_pages": orphan_pages.len(),
        "allowlisted_orphans": allowlisted_orphans.len(),
        "expired_orphan_allowlist_entries": orphan_allowlist_expired.len(),
        "missing_verified": missing_verified.len(),
    }))
}

#[derive(serde::Deserialize)]
struct OrphanAllowlistRoot {
    entries: Vec<OrphanAllowlistEntry>,
}

#[derive(serde::Deserialize)]
struct OrphanAllowlistEntry {
    path: String,
    owner: String,
    justification: String,
    expires_on: String,
}

fn load_orphan_allowlist(path: &std::path::Path) -> Result<Vec<OrphanAllowlistEntry>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let value: OrphanAllowlistRoot = serde_json::from_str(
        &fs::read_to_string(path).map_err(|e| format!("read {} failed: {e}", path.display()))?,
    )
    .map_err(|e| format!("parse {} failed: {e}", path.display()))?;
    let mut errors = Vec::new();
    for entry in &value.entries {
        if !entry.path.starts_with("docs/") {
            errors.push(format!(
                "orphan allowlist path must start with docs/: {}",
                entry.path
            ));
        }
        if entry.justification.trim().is_empty() {
            errors.push(format!(
                "orphan allowlist entry must include non-empty justification: {}",
                entry.path
            ));
        }
        if entry.owner.trim().is_empty() {
            errors.push(format!(
                "orphan allowlist entry must include owner: {}",
                entry.path
            ));
        }
        let is_iso = entry.expires_on.len() == 10
            && entry.expires_on.chars().enumerate().all(|(idx, ch)| {
                if idx == 4 || idx == 7 {
                    ch == '-'
                } else {
                    ch.is_ascii_digit()
                }
            });
        if !is_iso {
            errors.push(format!(
                "orphan allowlist entry must use ISO date YYYY-MM-DD: {} -> {}",
                entry.path, entry.expires_on
            ));
        }
    }
    if errors.is_empty() {
        Ok(value.entries)
    } else {
        Err(errors.join("; "))
    }
}

fn write_generated_docs_file(
    repo_root: &std::path::Path,
    rel_path: &str,
    body: String,
) -> Result<String, String> {
    let path = repo_root.join(rel_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {} failed: {e}", parent.display()))?;
    }
    fs::write(&path, format_generated_doc_content(&body))
        .map_err(|e| format!("write {} failed: {e}", path.display()))?;
    Ok(rel_path.to_string())
}

fn generated_docs_header() -> &'static str {
    "<!-- autogenerated: bijux-dev-atlas docs generate -->\n<!-- do not edit by hand -->\n<!-- Generated by: bijux-dev-atlas docs generate -->\n<!-- Do not edit by hand: regenerate with bijux-dev-atlas docs generate -->\n\n"
}

fn format_generated_doc_content(body: &str) -> String {
    format!("{}{body}", generated_docs_header())
}

fn build_generated_command_lists_body(repo_root: &std::path::Path) -> Result<String, String> {
    let root_help = std::process::Command::new("cargo")
        .current_dir(repo_root)
        .args(["run", "-q", "-p", "bijux-dev-atlas", "--", "--help"])
        .output()
        .map_err(|e| format!("run dev atlas help failed: {e}"))?;
    if !root_help.status.success() {
        return Err("bijux-dev-atlas --help failed".to_string());
    }
    let make_list = std::process::Command::new("cargo")
        .current_dir(repo_root)
        .args(["run", "-q", "-p", "bijux-dev-atlas", "--", "make", "list", "--format", "text"])
        .output()
        .map_err(|e| format!("run make list failed: {e}"))?;
    if !make_list.status.success() {
        return Err("bijux-dev-atlas make list failed".to_string());
    }
    Ok(format!(
        "# Generated Command Lists\n\n## bijux-dev-atlas\n\n```text\n{}\n```\n\n## make wrappers\n\n```text\n{}\n```\n",
        String::from_utf8(root_help.stdout).map_err(|e| e.to_string())?,
        String::from_utf8(make_list.stdout).map_err(|e| e.to_string())?,
    ))
}

fn docs_generate_command_lists(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let content = build_generated_command_lists_body(repo_root)?;
    Ok(vec![write_generated_docs_file(
        repo_root,
        "docs/_generated/command-lists.md",
        content,
    )?])
}

fn build_generated_schema_snippets_body(repo_root: &std::path::Path) -> Result<String, String> {
    let schema_path =
        repo_root.join("docs/bijux-atlas-crate/server/generated/runtime-startup-config.schema.json");
    let schema = fs::read_to_string(&schema_path)
        .map_err(|e| format!("read {} failed: {e}", schema_path.display()))?;
    let value: serde_json::Value =
        serde_json::from_str(&schema).map_err(|e| format!("parse runtime schema failed: {e}"))?;
    let required = value
        .get("required")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect::<Vec<_>>();
    Ok(format!(
        "# Generated Schema Snippets\n\nRuntime startup schema source: `docs/bijux-atlas-crate/server/generated/runtime-startup-config.schema.json`\n\nRequired fields:\n\n{}\n",
        required
            .iter()
            .map(|item| format!("- `{item}`"))
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

fn docs_generate_schema_snippets(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let content = build_generated_schema_snippets_body(repo_root)?;
    Ok(vec![write_generated_docs_file(
        repo_root,
        "docs/_generated/schema-snippets.md",
        content,
    )?])
}

fn build_generated_openapi_snippets_body(repo_root: &std::path::Path) -> Result<String, String> {
    let openapi_path = repo_root.join("configs/generated/openapi/v1/openapi.json");
    let openapi_text = fs::read_to_string(&openapi_path)
        .map_err(|e| format!("read {} failed: {e}", openapi_path.display()))?;
    let openapi: serde_json::Value =
        serde_json::from_str(&openapi_text).map_err(|e| format!("parse openapi failed: {e}"))?;
    let mut endpoints = openapi
        .get("paths")
        .and_then(|v| v.as_object())
        .map(|paths| paths.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    endpoints.sort();
    let sample = endpoints.into_iter().take(25).collect::<Vec<_>>();
    Ok(format!(
        "# Generated OpenAPI Snippets\n\nOpenAPI source: `configs/generated/openapi/v1/openapi.json`\n\nSample endpoint paths:\n\n{}\n",
        sample
            .iter()
            .map(|path| format!("- `{path}`"))
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

fn docs_generate_openapi_snippets(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let content = build_generated_openapi_snippets_body(repo_root)?;
    Ok(vec![write_generated_docs_file(
        repo_root,
        "docs/_generated/openapi-snippets.md",
        content,
    )?])
}

fn build_generated_ops_snippets_body(repo_root: &std::path::Path) -> Result<String, String> {
    let values_dir = repo_root.join("ops/k8s/values");
    let mut values_files = Vec::new();
    if values_dir.exists() {
        for entry in fs::read_dir(&values_dir).map_err(|e| format!("read {} failed: {e}", values_dir.display()))? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.extension().and_then(|v| v.to_str()) == Some("yaml") {
                values_files.push(
                    path.strip_prefix(repo_root)
                        .map_err(|e| e.to_string())?
                        .display()
                        .to_string(),
                );
            }
        }
    }
    values_files.sort();
    Ok(format!(
        "# Generated Ops Snippets\n\nKnown values files:\n\n{}\n",
        values_files
            .iter()
            .map(|path| format!("- `{path}`"))
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

fn docs_generate_ops_snippets(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let content = build_generated_ops_snippets_body(repo_root)?;
    Ok(vec![write_generated_docs_file(
        repo_root,
        "docs/_generated/ops-snippets.md",
        content,
    )?])
}

fn load_docs_real_data_catalog(repo_root: &std::path::Path) -> Result<DocsRealDataCatalog, String> {
    let path = repo_root.join("configs/sources/tutorials/real-data-runs.json");
    let text =
        fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("failed to parse {}: {e}", path.display()))
}

fn build_real_data_runs_table_body(catalog: &DocsRealDataCatalog) -> String {
    let mut out = String::from(
        "# Real Data Runs Table\n\n| Run label | Run ID | Dataset | Size tier | Ingest mode | Run page |\n| --- | --- | --- | --- | --- | --- |\n",
    );
    for run in &catalog.runs {
        out.push_str(&format!(
            "| {} | `{}` | `{}` | `{}` | `{}` | [Details](../ops/tutorials/real-data/runs/{}.md) |\n",
            run.run_label, run.id, run.dataset, run.dataset_size_tier, run.ingest_mode, run.id
        ));
    }
    out
}

fn build_real_data_overview_body(catalog: &DocsRealDataCatalog) -> String {
    let mut out = String::from(
        "# Real Data Runs Overview\n\nYou are here: `docs/_internal/generated/real-data-runs-overview.md`\n\nReturn to: [Real Data Runs index](../../ops/tutorials/real-data/index.md)\n\nThis page is generated from `configs/sources/tutorials/real-data-runs.json`.\n\n",
    );
    out.push_str("## Coverage Summary\n\n");
    out.push_str(&format!("- Total runs: `{}`\n", catalog.runs.len()));
    let tiny = catalog
        .runs
        .iter()
        .filter(|run| run.dataset_size_tier == "tiny")
        .count();
    let medium = catalog
        .runs
        .iter()
        .filter(|run| run.dataset_size_tier == "medium")
        .count();
    let large_sample = catalog
        .runs
        .iter()
        .filter(|run| run.dataset_size_tier == "large-sample")
        .count();
    out.push_str(&format!("- Tiny runs: `{tiny}`\n"));
    out.push_str(&format!("- Medium runs: `{medium}`\n"));
    out.push_str(&format!("- Large-sample runs: `{large_sample}`\n\n"));
    out.push_str("## Run Links\n\n");
    for run in &catalog.runs {
        out.push_str(&format!(
            "- `{}`: [ops/tutorials/real-data/runs/{}.md](../../ops/tutorials/real-data/runs/{}.md)\n",
            run.run_label, run.id, run.id
        ));
    }
    out.push_str("\n## Related pages\n\n");
    out.push_str("- [Real Data Runs table](../../_generated/real-data-runs-table.md)\n");
    out.push_str("- [Real Data Runs report](../../reference/reports/real-data-runs.md)\n");
    out.push_str("\n---\nParent index: [Tutorials](../../ops/tutorials/index.md)\n");
    out
}

fn docs_markdown_link_inventory(repo_root: &std::path::Path) -> Result<Vec<(String, String)>, String> {
    let docs_root = repo_root.join("docs");
    let files = docs_markdown_files(&docs_root, false);
    let link_re = regex::Regex::new(r"\[[^\]]+\]\(([^)]+)\)").map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&docs_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let text = fs::read_to_string(&file).unwrap_or_default();
        for cap in link_re.captures_iter(&text) {
            let target = cap.get(1).map(|m| m.as_str()).unwrap_or_default().to_string();
            rows.push((rel.clone(), target));
        }
    }
    Ok(rows)
}

fn build_artifact_link_inventory_body(rows: &[(String, String)]) -> String {
    let mut artifacts = Vec::new();
    let mut docs_or_ops = Vec::new();
    for (page, link) in rows {
        let artifact_path_like = link.starts_with("artifacts/")
            || link.starts_with("../artifacts/")
            || link.starts_with("/artifacts/");
        if artifact_path_like {
            artifacts.push((page.clone(), link.clone()));
        }
        if link.starts_with("artifacts/docs/")
            || link.starts_with("../artifacts/docs/")
            || link.starts_with("/artifacts/docs/")
            || link.starts_with("artifacts/ops/")
            || link.starts_with("../artifacts/ops/")
            || link.starts_with("/artifacts/ops/")
        {
            docs_or_ops.push((page.clone(), link.clone()));
        }
    }
    let mut out = String::from(
        "# Docs Artifact Link Inventory\n\nYou are here: `docs/_internal/generated/docs-artifact-link-inventory.md`\n\nReturn to: [Docs governance index](../ops/governance/repository/index.md)\n\n",
    );
    out.push_str("## Links Containing `artifacts/`\n\n");
    if artifacts.is_empty() {
        out.push_str("- none\n");
    } else {
        for (page, link) in &artifacts {
            out.push_str(&format!("- `{page}` -> `{link}`\n"));
        }
    }
    out.push_str("\n## Links Containing `artifacts/docs/` or `artifacts/ops/`\n\n");
    if docs_or_ops.is_empty() {
        out.push_str("- none\n");
    } else {
        for (page, link) in &docs_or_ops {
            out.push_str(&format!("- `{page}` -> `{link}`\n"));
        }
    }
    out.push_str("\n---\nParent index: [Reference reports](../../reference/reports/index.md)\n");
    out
}

fn docs_generate_real_data_pages(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let mut generated = Vec::new();
    let catalog = load_docs_real_data_catalog(repo_root)?;
    generated.push(write_generated_docs_file(
        repo_root,
        "docs/_generated/real-data-runs-table.md",
        build_real_data_runs_table_body(&catalog),
    )?);
    generated.push(write_generated_docs_file(
        repo_root,
        "docs/_internal/generated/real-data-runs-overview.md",
        build_real_data_overview_body(&catalog),
    )?);
    let overview_json = serde_json::json!({
        "schema_version": 1,
        "runs": catalog.runs.iter().map(|run| serde_json::json!({
            "id": run.id,
            "run_label": run.run_label,
            "dataset": run.dataset,
            "dataset_size_tier": run.dataset_size_tier,
            "ingest_mode": run.ingest_mode,
            "query_names": run.expected_query_set.iter().map(|query| query.name.clone()).collect::<Vec<_>>(),
            "expected_artifacts": run.expected_artifacts,
            "input_provenance": {
                "url": run.input_provenance.url,
                "retrieval_method": run.input_provenance.retrieval_method,
                "license_note": run.input_provenance.license_note
            }
        })).collect::<Vec<_>>()
    });
    let overview_json_path = repo_root.join("docs/_internal/generated/real-data-runs-overview.json");
    fs::write(
        &overview_json_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&overview_json)
                .map_err(|e| format!("failed to encode {}: {e}", overview_json_path.display()))?
        ),
    )
    .map_err(|e| format!("write {} failed: {e}", overview_json_path.display()))?;
    generated.push("docs/_internal/generated/real-data-runs-overview.json".to_string());
    let link_rows = docs_markdown_link_inventory(repo_root)?;
    generated.push(write_generated_docs_file(
        repo_root,
        "docs/_internal/generated/docs-artifact-link-inventory.md",
        build_artifact_link_inventory_body(&link_rows),
    )?);
    Ok(generated)
}

fn build_generated_examples_body() -> String {
    [
        "# Generated Examples Index".to_string(),
        "".to_string(),
        "- [Command lists](command-lists.md)".to_string(),
        "- [Schema snippets](schema-snippets.md)".to_string(),
        "- [OpenAPI snippets](openapi-snippets.md)".to_string(),
        "- [Ops snippets](ops-snippets.md)".to_string(),
        "- [Real data runs table](real-data-runs-table.md)".to_string(),
        "".to_string(),
    ]
    .join("\n")
}

fn docs_generate_examples(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let content = build_generated_examples_body();
    Ok(vec![write_generated_docs_file(
        repo_root,
        "docs/_generated/examples.md",
        content,
    )?])
}

fn docs_generate_all(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let mut generated = Vec::new();
    generated.extend(docs_generate_command_lists(repo_root)?);
    generated.extend(docs_generate_schema_snippets(repo_root)?);
    generated.extend(docs_generate_openapi_snippets(repo_root)?);
    generated.extend(docs_generate_ops_snippets(repo_root)?);
    generated.extend(docs_generate_real_data_pages(repo_root)?);
    generated.extend(docs_generate_examples(repo_root)?);
    Ok(generated)
}

fn docs_verify_generated(repo_root: &std::path::Path) -> Result<serde_json::Value, String> {
    let catalog = load_docs_real_data_catalog(repo_root)?;
    let link_rows = docs_markdown_link_inventory(repo_root)?;
    let workspace_version = fs::read_to_string(repo_root.join("Cargo.toml"))
        .ok()
        .and_then(|text| text.lines().find(|line| line.trim_start().starts_with("version = ")).map(str::to_string))
        .and_then(|line| line.split('"').nth(1).map(str::to_string))
        .unwrap_or_else(|| "0.1.0".to_string());
    let ops_manifest_path = repo_root.join("ops/release/ops-release-manifest.json");
    let ops_manifest: serde_json::Value = fs::read_to_string(&ops_manifest_path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    let chart_version = ops_manifest
        .get("chart_version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(workspace_version.as_str());
    let chart_reference = ops_manifest
        .get("chart_reference")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("oci://ghcr.io/bijux/charts/bijux-atlas");
    let ops_matrix_json = serde_json::json!({
        "schema_version": 1,
        "kind": "release_ops_compatibility_matrix",
        "status": "ok",
        "rows": [{
            "runtime_version": workspace_version,
            "chart_version": chart_version,
            "client_version": workspace_version,
            "chart_reference": chart_reference
        }]
    });
    let ops_matrix_md = format!(
        "# Ops Compatibility Matrix\n\n| Runtime version | Chart version | Client version | Chart reference |\n| --- | --- | --- | --- |\n| `{}` | `{}` | `{}` | `{}` |\n",
        ops_matrix_json["rows"][0]["runtime_version"].as_str().unwrap_or_default(),
        ops_matrix_json["rows"][0]["chart_version"].as_str().unwrap_or_default(),
        ops_matrix_json["rows"][0]["client_version"].as_str().unwrap_or_default(),
        ops_matrix_json["rows"][0]["chart_reference"].as_str().unwrap_or_default()
    );
    let expected = [
        (
            "docs/_generated/examples.md",
            format_generated_doc_content(&build_generated_examples_body()),
        ),
        (
            "docs/_generated/command-lists.md",
            format_generated_doc_content(&build_generated_command_lists_body(repo_root)?),
        ),
        (
            "docs/_generated/schema-snippets.md",
            format_generated_doc_content(&build_generated_schema_snippets_body(repo_root)?),
        ),
        (
            "docs/_generated/openapi-snippets.md",
            format_generated_doc_content(&build_generated_openapi_snippets_body(repo_root)?),
        ),
        (
            "docs/_generated/ops-snippets.md",
            format_generated_doc_content(&build_generated_ops_snippets_body(repo_root)?),
        ),
        (
            "docs/_generated/real-data-runs-table.md",
            format_generated_doc_content(&build_real_data_runs_table_body(&catalog)),
        ),
        (
            "docs/_internal/generated/docs-artifact-link-inventory.md",
            format_generated_doc_content(&build_artifact_link_inventory_body(&link_rows)),
        ),
        (
            "docs/_internal/generated/ops-compatibility-matrix.md",
            format_generated_doc_content(&ops_matrix_md),
        ),
        (
            "docs/_internal/generated/ops-compatibility-matrix.json",
            format!(
                "{}\n",
                serde_json::to_string_pretty(&ops_matrix_json)
                    .map_err(|e| format!("encode ops compatibility matrix failed: {e}"))?
            ),
        ),
    ];
    let mut missing = Vec::new();
    let mut missing_header = Vec::new();
    let mut stale = Vec::new();
    for (rel, expected_content) in &expected {
        let path = repo_root.join(rel);
        if !path.exists() {
            missing.push((*rel).to_string());
            continue;
        }
        let actual = fs::read_to_string(&path)
            .map_err(|e| format!("read {} failed: {e}", path.display()))?;
        if rel.ends_with(".md") && !actual.starts_with(generated_docs_header()) {
            missing_header.push((*rel).to_string());
        }
        if rel.ends_with(".json") {
            let actual_json: serde_json::Value = serde_json::from_str(&actual)
                .map_err(|e| format!("parse {} failed: {e}", path.display()))?;
            let expected_json: serde_json::Value = serde_json::from_str(expected_content)
                .map_err(|e| format!("parse expected json for {} failed: {e}", rel))?;
            if actual_json != expected_json {
                stale.push((*rel).to_string());
            }
        } else if actual != *expected_content {
            stale.push((*rel).to_string());
        }
    }
    let registry_path = repo_root.join("configs/sources/repository/docs/generated-files-registry.json");
    let registry_json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&registry_path)
            .map_err(|e| format!("read {} failed: {e}", registry_path.display()))?,
    )
    .map_err(|e| format!("parse {} failed: {e}", registry_path.display()))?;
    let registry_entries = registry_json
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let required_set = expected
        .iter()
        .map(|(path, _)| path.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let mut registry_missing = Vec::new();
    let mut registry_extra = Vec::new();
    let mut registry_paths = std::collections::BTreeSet::new();
    for row in &registry_entries {
        if let Some(path) = row.get("path").and_then(serde_json::Value::as_str) {
            registry_paths.insert(path.to_string());
        }
    }
    for required in &required_set {
        if !registry_paths.contains(required) {
            registry_missing.push(required.clone());
        }
    }
    let allowed_registry_only = std::collections::BTreeSet::from([
        "docs/_internal/generated/real-data-runs-overview.md".to_string(),
        "docs/_internal/generated/real-data-runs-overview.json".to_string(),
    ]);
    for path in &registry_paths {
        if !required_set.contains(path) && !allowed_registry_only.contains(path) {
            registry_extra.push(path.clone());
        }
    }

    let freshness_path = repo_root.join("configs/sources/repository/docs/generated-files-freshness-policy.json");
    let freshness_json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&freshness_path)
            .map_err(|e| format!("read {} failed: {e}", freshness_path.display()))?,
    )
    .map_err(|e| format!("parse {} failed: {e}", freshness_path.display()))?;
    let max_age_days = freshness_json
        .get("max_age_days")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(30);
    let now = std::time::SystemTime::now();
    let max_age = std::time::Duration::from_secs(max_age_days * 24 * 60 * 60);
    let mut stale_freshness = Vec::new();
    for rel in required_set {
        let path = repo_root.join(&rel);
        if !path.exists() {
            continue;
        }
        let modified = fs::metadata(&path)
            .and_then(|m| m.modified())
            .unwrap_or(now);
        if now.duration_since(modified).unwrap_or_default() > max_age {
            stale_freshness.push(rel);
        }
    }
    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "docs_verify_generated",
        "status": if missing.is_empty()
            && missing_header.is_empty()
            && stale.is_empty()
            && registry_missing.is_empty()
            && registry_extra.is_empty()
            && stale_freshness.is_empty()
        { "ok" } else { "failed" },
        "required": expected.iter().map(|(path, _)| *path).collect::<Vec<_>>(),
        "missing": missing,
        "missing_header": missing_header,
        "stale": stale,
        "registry_missing": registry_missing,
        "registry_extra": registry_extra,
        "freshness_stale": stale_freshness,
    }))
}

pub(crate) fn run_docs_command(quiet: bool, command: DocsCommand) -> i32 {
    let run = (|| -> Result<(String, i32), String> {
        let started = std::time::Instant::now();
        match command {
            DocsCommand::Check(common) => {
                if !common.allow_subprocess {
                    return Err("docs check requires --allow-subprocess".to_string());
                }
                let ctx = docs_context(&common)?;
                let validate = docs_validate_payload(&ctx, &common)?;
                let links = docs_links_payload(&ctx, &common)?;
                let lint = docs_lint_payload(&ctx, &common)?;
                let spine_report = docs_spine_report_payload(&ctx, &common)?;
                let (build_payload, build_code) =
                    docs_build_or_serve_subprocess(&["build".to_string()], &common, "docs build")?;
                let errors = validate["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + links["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + lint["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + usize::from(build_code != 0);
                let mut payload = serde_json::json!({
                    "schema_version":1,
                    "run_id":ctx.run_id.as_str(),
                    "text": if errors == 0 { "docs check passed" } else { "docs check failed" },
                    "rows":[
                        {"name":"validate","errors": validate["errors"].as_array().map(|v| v.len()).unwrap_or(0)},
                        {"name":"links","errors": links["errors"].as_array().map(|v| v.len()).unwrap_or(0)},
                        {"name":"lint","errors": lint["errors"].as_array().map(|v| v.len()).unwrap_or(0)},
                        {"name":"spine-report","orphans": spine_report["orphans"].as_array().map(|v| v.len()).unwrap_or(0)},
                        {"name":"build","exit_code": build_code}
                    ],
                    "checks": {"validate": validate, "links": links, "lint": lint, "spine_report": spine_report, "build": build_payload},
                    "counts":{"errors": errors},
                    "capabilities":{"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "options":{"strict": common.strict, "include_drafts": common.include_drafts},
                    "duration_ms": started.elapsed().as_millis() as u64,
                    "error_code": if errors == 0 { serde_json::Value::Null } else { serde_json::Value::String("DOCS_BUILD_ERROR".to_string()) }
                });
                if common.allow_write {
                    let out_dir = docs_gate_artifact_dir(&ctx);
                    let check_path = out_dir.join("check.json");
                    let validate_path = out_dir.join("validate.json");
                    let links_path = out_dir.join("links.json");
                    let lint_path = out_dir.join("lint.json");
                    let spine_report_path = out_dir.join("spine-report.json");
                    let build_path = out_dir.join("build.json");
                    let meta_path = out_dir.join("meta.json");
                    let summary_path = out_dir.join("summary.json");
                    let meta = serde_json::json!({
                        "schema_version": 1,
                        "gate": "docs",
                        "command": "bijux dev atlas docs check",
                        "run_id": ctx.run_id.as_str(),
                        "status": if errors == 0 { "pass" } else { "fail" },
                        "timestamp_unix": current_unix_timestamp(),
                    });
                    let summary = serde_json::json!({
                        "schema_version": 1,
                        "gate": "docs",
                        "run_id": ctx.run_id.as_str(),
                        "status": if errors == 0 { "pass" } else { "fail" },
                        "counts": {
                            "errors": errors,
                            "validate_errors": validate["errors"].as_array().map(|v| v.len()).unwrap_or(0),
                            "links_errors": links["errors"].as_array().map(|v| v.len()).unwrap_or(0),
                            "lint_errors": lint["errors"].as_array().map(|v| v.len()).unwrap_or(0),
                            "build_exit_code": build_code
                        }
                    });
                    write_docs_gate_artifact(&validate_path, &validate)?;
                    write_docs_gate_artifact(&links_path, &links)?;
                    write_docs_gate_artifact(&lint_path, &lint)?;
                    write_docs_gate_artifact(&spine_report_path, &spine_report)?;
                    write_docs_gate_artifact(&build_path, &build_payload)?;
                    write_docs_gate_artifact(&meta_path, &meta)?;
                    write_docs_gate_artifact(&summary_path, &summary)?;
                    write_docs_gate_artifact(&check_path, &payload)?;
                    payload["artifacts"] = serde_json::json!({
                        "check": check_path.display().to_string(),
                        "validate": validate_path.display().to_string(),
                        "links": links_path.display().to_string(),
                        "lint": lint_path.display().to_string(),
                        "spine_report": spine_report_path.display().to_string(),
                        "build": build_path.display().to_string(),
                        "meta": meta_path.display().to_string(),
                        "summary": summary_path.display().to_string()
                    });
                }
                Ok((
                    emit_payload(common.format, common.out, &payload)?,
                    if errors == 0 { 0 } else { 1 },
                ))
            }
            DocsCommand::VerifyContracts(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_verify_contracts_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                if code != 0 {
                    payload["error_code"] = serde_json::json!("DOCS_CONTRACT_ERROR");
                }
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::Validate(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_validate_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                if code != 0 {
                    payload["error_code"] = serde_json::json!("DOCS_NAV_ERROR");
                }
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::Inventory(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_inventory_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            DocsCommand::Graph(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_graph_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            DocsCommand::Top(args) => {
                let ctx = docs_context(&args.common)?;
                let mut payload = docs_top_payload(&ctx, &args.common, args.limit)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(args.common.format, args.common.out, &payload)?, 0))
            }
            DocsCommand::Dead(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_dead_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            DocsCommand::Duplicates(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_duplicates_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            DocsCommand::DedupeReport(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_duplicates_payload(&ctx, &common)?;
                payload["kind"] = serde_json::json!("docs_dedupe_report");
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            DocsCommand::PrunePlan(common) => {
                let ctx = docs_context(&common)?;
                let dead = docs_dead_payload(&ctx, &common)?;
                let duplicates = docs_duplicates_payload(&ctx, &common)?;
                let mut payload = serde_json::json!({
                    "schema_version": 1,
                    "kind": "docs_prune_plan",
                    "status": "ok",
                    "dead_pages": dead.get("rows").cloned().unwrap_or_else(|| serde_json::json!([])),
                    "duplicate_clusters": duplicates.get("rows").cloned().unwrap_or_else(|| serde_json::json!([])),
                });
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            DocsCommand::ShrinkReport(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_shrink_report_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                if common.allow_write {
                    let report_path = ctx
                        .repo_root
                        .join("ops/_generated.example/docs-shrink-report.json");
                    if let Some(parent) = report_path.parent() {
                        fs::create_dir_all(parent).map_err(|e| {
                            format!("failed to create {}: {e}", parent.display())
                        })?;
                    }
                    fs::write(
                        &report_path,
                        serde_json::to_string_pretty(&payload)
                            .map_err(|e| format!("encode docs shrink report failed: {e}"))?,
                    )
                    .map_err(|e| format!("write {} failed: {e}", report_path.display()))?;
                }
                let code = if payload["status"] == "pass" { 0 } else { 1 };
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::HealthDashboard(common) => {
                if !common.allow_write {
                    return Err("docs health-dashboard requires --allow-write".to_string());
                }
                let ctx = docs_context(&common)?;
                let mut payload = docs_generate_health_dashboard(&ctx.repo_root)?;
                payload["run_id"] = serde_json::json!(ctx.run_id.as_str());
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            DocsCommand::LifecycleSummaryTable(args) => {
                if !args.common.allow_write {
                    return Err("docs lifecycle-summary-table requires --allow-write".to_string());
                }
                let mut payload = docs_write_summary_table(
                    &args.input,
                    &args.output,
                    "profiles",
                    |mut rows| {
                        rows.sort_by_key(|row| row["profile"].as_str().unwrap_or_default().to_string());
                        let header = vec![
                            "<!-- Generated by bijux-dev-atlas docs lifecycle-summary-table --allow-write. Do not edit by hand. -->".to_string(),
                            String::new(),
                            "# Lifecycle Summary Table".to_string(),
                            String::new(),
                            "| Profile | Namespace | Upgrade | Rollback |".to_string(),
                            "| --- | --- | --- | --- |".to_string(),
                        ];
                        let table = render_summary_table(
                            &rows,
                            "",
                            "| none | none | not-run | not-run |",
                            |row| {
                                format!(
                                    "| {} | {} | {} | {} |",
                                    row["profile"].as_str().unwrap_or_default(),
                                    row["namespace"].as_str().unwrap_or_default(),
                                    row["upgrade_status"].as_str().unwrap_or("not-run"),
                                    row["rollback_status"].as_str().unwrap_or("not-run")
                                )
                            },
                        );
                        let mut lines = header;
                        lines.extend(table.lines().skip(2).map(str::to_string));
                        lines.join("\n")
                    },
                )?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(args.common.format, args.common.out, &payload)?, 0))
            }
            DocsCommand::DrillSummaryTable(args) => {
                if !args.common.allow_write {
                    return Err("docs drill-summary-table requires --allow-write".to_string());
                }
                let mut payload = docs_write_summary_table(
                    &args.input,
                    &args.output,
                    "drills",
                    |mut rows| {
                        rows.sort_by_key(|row| row["name"].as_str().unwrap_or_default().to_string());
                        let lines = vec![
                            "# Drill Summary".to_string(),
                            String::new(),
                            "| Drill | Status | Report |".to_string(),
                            "| --- | --- | --- |".to_string(),
                        ];
                        let table = render_summary_table(
                            &rows,
                            "",
                            "| (none) | n/a | n/a |",
                            |row| {
                                format!(
                                    "| {} | {} | `{}` |",
                                    row["name"].as_str().unwrap_or_default(),
                                    row["status"].as_str().unwrap_or_default(),
                                    row["report_path"].as_str().unwrap_or_default()
                                )
                            },
                        );
                        let mut out = lines;
                        out.extend(table.lines().skip(2).map(str::to_string));
                        out.join("\n")
                    },
                )?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(args.common.format, args.common.out, &payload)?, 0))
            }
            DocsCommand::Reference { command } => match command {
                crate::cli::DocsReferenceCommand::Generate(common) => {
                    let ctx = docs_context(&common)?;
                    let (changed, generated) = docs_reference_generate_or_check(&ctx.repo_root, true)?;
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "run_id": ctx.run_id.as_str(),
                        "status": "ok",
                        "text": "docs reference pages generated",
                        "changed": changed,
                        "generated": generated,
                        "duration_ms": started.elapsed().as_millis() as u64,
                    });
                    Ok((emit_payload(common.format, common.out, &payload)?, 0))
                }
                crate::cli::DocsReferenceCommand::Check(common) => {
                    let ctx = docs_context(&common)?;
                    let (changed, generated) = docs_reference_generate_or_check(&ctx.repo_root, false)?;
                    let code = if changed.is_empty() { 0 } else { 1 };
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "run_id": ctx.run_id.as_str(),
                        "status": if code == 0 { "ok" } else { "drift" },
                        "text": if code == 0 { "docs reference pages up to date" } else { "docs reference pages drift detected" },
                        "changed": changed,
                        "generated": generated,
                        "duration_ms": started.elapsed().as_millis() as u64,
                    });
                    Ok((emit_payload(common.format, common.out, &payload)?, code))
                }
            },
            DocsCommand::Generate { command } => match command {
                crate::cli::DocsGenerateCommand::Examples(common) => {
                    if !common.allow_write {
                        return Err("docs generate examples requires --allow-write".to_string());
                    }
                    let ctx = docs_context(&common)?;
                    let generated = docs_generate_all(&ctx.repo_root)?;
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "kind": "docs_generate_examples",
                        "status": "ok",
                        "generated": generated,
                        "duration_ms": started.elapsed().as_millis() as u64,
                    });
                    Ok((emit_payload(common.format, common.out, &payload)?, 0))
                }
                crate::cli::DocsGenerateCommand::CommandLists(common) => {
                    if !common.allow_write || !common.allow_subprocess {
                        return Err("docs generate command-lists requires --allow-write --allow-subprocess".to_string());
                    }
                    let ctx = docs_context(&common)?;
                    let generated = docs_generate_command_lists(&ctx.repo_root)?;
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "kind": "docs_generate_command_lists",
                        "status": "ok",
                        "generated": generated,
                        "duration_ms": started.elapsed().as_millis() as u64,
                    });
                    Ok((emit_payload(common.format, common.out, &payload)?, 0))
                }
                crate::cli::DocsGenerateCommand::SchemaSnippets(common) => {
                    if !common.allow_write {
                        return Err("docs generate schema-snippets requires --allow-write".to_string());
                    }
                    let ctx = docs_context(&common)?;
                    let generated = docs_generate_schema_snippets(&ctx.repo_root)?;
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "kind": "docs_generate_schema_snippets",
                        "status": "ok",
                        "generated": generated,
                        "duration_ms": started.elapsed().as_millis() as u64,
                    });
                    Ok((emit_payload(common.format, common.out, &payload)?, 0))
                }
                crate::cli::DocsGenerateCommand::OpenapiSnippets(common) => {
                    if !common.allow_write {
                        return Err("docs generate openapi-snippets requires --allow-write".to_string());
                    }
                    let ctx = docs_context(&common)?;
                    let generated = docs_generate_openapi_snippets(&ctx.repo_root)?;
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "kind": "docs_generate_openapi_snippets",
                        "status": "ok",
                        "generated": generated,
                        "duration_ms": started.elapsed().as_millis() as u64,
                    });
                    Ok((emit_payload(common.format, common.out, &payload)?, 0))
                }
                crate::cli::DocsGenerateCommand::OpsSnippets(common) => {
                    if !common.allow_write {
                        return Err("docs generate ops-snippets requires --allow-write".to_string());
                    }
                    let ctx = docs_context(&common)?;
                    let generated = docs_generate_ops_snippets(&ctx.repo_root)?;
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "kind": "docs_generate_ops_snippets",
                        "status": "ok",
                        "generated": generated,
                        "duration_ms": started.elapsed().as_millis() as u64,
                    });
                    Ok((emit_payload(common.format, common.out, &payload)?, 0))
                }
                crate::cli::DocsGenerateCommand::RealDataPages(common) => {
                    if !common.allow_write {
                        return Err(
                            "docs generate real-data-pages requires --allow-write".to_string()
                        );
                    }
                    let ctx = docs_context(&common)?;
                    let generated = docs_generate_real_data_pages(&ctx.repo_root)?;
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "kind": "docs_generate_real_data_pages",
                        "status": "ok",
                        "generated": generated,
                        "duration_ms": started.elapsed().as_millis() as u64,
                    });
                    Ok((emit_payload(common.format, common.out, &payload)?, 0))
                }
            },
            DocsCommand::Redirects { command } => match command {
                crate::cli::DocsRedirectsCommand::Sync(common) => {
                    if !common.allow_write {
                        return Err("docs redirects sync requires --allow-write".to_string());
                    }
                    let ctx = docs_context(&common)?;
                    let mut payload = docs_sync_redirects(&ctx.repo_root)?;
                    payload["run_id"] = serde_json::json!(ctx.run_id.as_str());
                    payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                    Ok((emit_payload(common.format, common.out, &payload)?, 0))
                }
            },
            DocsCommand::Merge { command } => match command {
                crate::cli::DocsMergeCommand::Validate(common) => {
                    let ctx = docs_context(&common)?;
                    let mut payload = docs_merge_validate_payload(&ctx, &common)?;
                    payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                    let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                        1
                    } else {
                        0
                    };
                    if code != 0 {
                        payload["error_code"] = serde_json::json!("DOCS_MERGE_ERROR");
                    }
                    Ok((emit_payload(common.format, common.out, &payload)?, code))
                }
            },
            DocsCommand::Spine { command } => match command {
                crate::cli::DocsSpineCommand::Validate(common) => {
                    let ctx = docs_context(&common)?;
                    let mut payload = docs_spine_validate_payload(&ctx, &common)?;
                    payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                    let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                        1
                    } else {
                        0
                    };
                    Ok((emit_payload(common.format, common.out, &payload)?, code))
                }
                crate::cli::DocsSpineCommand::Report(common) => {
                    let ctx = docs_context(&common)?;
                    let mut payload = docs_spine_report_payload(&ctx, &common)?;
                    payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                    let code = if payload["orphans"].as_array().is_some_and(|v| !v.is_empty()) {
                        1
                    } else {
                        0
                    };
                    Ok((emit_payload(common.format, common.out, &payload)?, code))
                }
            },
            DocsCommand::Toc { command } => match command {
                crate::cli::DocsTocCommand::Verify(common) => {
                    let ctx = docs_context(&common)?;
                    let mut payload = docs_links_payload(&ctx, &common)?;
                    payload["kind"] = serde_json::json!("docs_toc_verify");
                    payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                    let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                        1
                    } else {
                        0
                    };
                    if code != 0 {
                        payload["error_code"] = serde_json::json!("DOCS_TOC_ERROR");
                    }
                    Ok((emit_payload(common.format, common.out, &payload)?, code))
                }
            },
            DocsCommand::Links(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_links_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                if code != 0 {
                    payload["error_code"] = serde_json::json!("DOCS_LINK_ERROR");
                }
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::NavIntegrity(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_nav_integrity_payload(&ctx)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::ExternalLinks(args) => {
                let ctx = docs_context(&args.common)?;
                let mut payload =
                    docs_external_links_payload(&ctx, &args.common, &args.allowlist)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                if code != 0 {
                    payload["error_code"] = serde_json::json!("DOCS_EXTERNAL_LINK_ERROR");
                }
                Ok((emit_payload(args.common.format, args.common.out, &payload)?, code))
            }
            DocsCommand::Lint(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_lint_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::IncludesCheck(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_includes_check_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::Grep(args) => {
                let ctx = docs_context(&args.common)?;
                let mut payload = docs_grep_payload(&ctx, &args.common, &args.pattern)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((
                    emit_payload(args.common.format, args.common.out, &payload)?,
                    0,
                ))
            }
            DocsCommand::Where(common) => {
                let ctx = docs_context(&common)?;
                let site_paths =
                    bijux_dev_atlas::docs::site_output::parse_mkdocs_site_paths(&ctx.repo_root)?;
                let pages_url = std::env::var("BIJUX_DOCS_SITE_URL")
                    .unwrap_or_else(|_| "https://bijux.github.io/bijux-atlas/".to_string());
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": ctx.run_id.as_str(),
                    "text": format!("docs site: {pages_url}"),
                    "rows": [{
                        "pages_url": pages_url,
                        "site_dir": site_paths.site_dir.display().to_string(),
                        "docs_dir": site_paths.docs_dir.display().to_string()
                    }],
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            DocsCommand::DeployPlan(common) => {
                let ctx = docs_context(&common)?;
                let site_paths =
                    bijux_dev_atlas::docs::site_output::parse_mkdocs_site_paths(&ctx.repo_root)?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": ctx.run_id.as_str(),
                    "text": "docs deploy plan",
                    "rows": [{
                        "workflow": ".github/workflows/docs-deploy.yml",
                        "build_command": "bijux-dev-atlas docs build --allow-subprocess --allow-write --strict",
                        "site_dir": site_paths.site_dir.display().to_string(),
                        "pages_url": std::env::var("BIJUX_DOCS_SITE_URL").unwrap_or_else(|_| "https://bijux.github.io/bijux-atlas/".to_string()),
                        "trigger_push_main": true,
                        "trigger_tag": "v*",
                        "trigger_manual": true
                    }],
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            DocsCommand::PagesSmoke(args) => {
                if !args.common.allow_network {
                    return Err("docs pages-smoke requires --allow-network".to_string());
                }
                let ctx = docs_context(&args.common)?;
                let url = args.url.clone().unwrap_or_else(|| {
                    let base = std::env::var("BIJUX_DOCS_SITE_URL")
                        .unwrap_or_else(|_| "https://bijux.github.io/bijux-atlas/".to_string());
                    format!("{}/reference/build-info/", base.trim_end_matches('/'))
                });
                let response = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(20))
                    .build()
                    .map_err(|e| format!("failed to build http client: {e}"))?
                    .get(&url)
                    .send()
                    .map_err(|e| format!("failed to fetch {url}: {e}"))?;
                let status = response.status().as_u16();
                let body = response
                    .text()
                    .map_err(|e| format!("failed to read response body from {url}: {e}"))?;
                let marker_found = body.contains(&args.marker);
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": ctx.run_id.as_str(),
                    "text": if status < 400 && marker_found { "docs pages smoke passed" } else { "docs pages smoke failed" },
                    "rows": [{
                        "url": url,
                        "http_status": status,
                        "marker": args.marker,
                        "marker_found": marker_found
                    }],
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                let code = if status < 400 && marker_found { 0 } else { 1 };
                Ok((emit_payload(args.common.format, args.common.out, &payload)?, code))
            }
            DocsCommand::UxSmoke(common) => {
                if !common.allow_subprocess || !common.allow_write {
                    return Err("docs ux-smoke requires --allow-subprocess --allow-write".to_string());
                }
                let ctx = docs_context(&common)?;
                docs_build_or_serve_subprocess(
                    &["build".to_string()],
                    &common,
                    "docs build",
                )?;
                let site_paths =
                    bijux_dev_atlas::docs::site_output::parse_mkdocs_site_paths(&ctx.repo_root)?;
                let site_dir = ctx.repo_root.join(&site_paths.site_dir);
                let breadcrumbs_path = ctx
                    .repo_root
                    .join("docs/_internal/generated/breadcrumbs.json");
                let breadcrumb_map: std::collections::BTreeSet<String> = if breadcrumbs_path.exists()
                {
                    let value: serde_json::Value = serde_json::from_str(
                        &fs::read_to_string(&breadcrumbs_path).unwrap_or_default(),
                    )
                    .unwrap_or_default();
                    value
                        .get("rows")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|row| row.get("path").and_then(|v| v.as_str()).map(str::to_string))
                        .collect()
                } else {
                    std::collections::BTreeSet::new()
                };
                let samples = [
                    "index.html",
                    "ops/tutorials/real-data/index.html",
                    "_generated/real-data-runs-table/index.html",
                    "_internal/generated/real-data-runs-overview/index.html",
                ];
                let mut rows = Vec::new();
                let mut failures = Vec::new();
                let mut warnings = Vec::new();
                for sample in samples {
                    let page = site_dir.join(sample);
                    if !page.exists() {
                        failures.push(format!("missing sample page `{}`", page.display()));
                        rows.push(serde_json::json!({
                            "sample": sample,
                            "exists": false,
                            "has_top_nav": false,
                            "has_side_nav": false,
                            "has_breadcrumb": false
                        }));
                        continue;
                    }
                    let text = fs::read_to_string(&page).unwrap_or_default();
                    let has_top_nav =
                        text.contains("md-header") || text.contains("md-tabs") || text.contains("md-tabs__item");
                    let has_side_nav = text.contains("md-nav") || text.contains("md-sidebar");
                    let expected_doc_path = if sample == "index.html" {
                        "docs/index.md".to_string()
                    } else {
                        "docs/".to_string()
                            + sample.trim_end_matches("index.html").trim_end_matches('/')
                            + ".md"
                    };
                    let has_breadcrumb = text.contains("md-path")
                        || text.contains("md-path__item")
                        || breadcrumb_map.contains(&expected_doc_path);
                    if !has_top_nav {
                        failures.push(format!("sample `{sample}` missing top navigation markers"));
                    }
                    if !has_side_nav {
                        failures.push(format!("sample `{sample}` missing side navigation markers"));
                    }
                    if !has_breadcrumb {
                        warnings.push(format!("sample `{sample}` missing breadcrumb path markers"));
                    }
                    rows.push(serde_json::json!({
                        "sample": sample,
                        "exists": true,
                        "has_top_nav": has_top_nav,
                        "has_side_nav": has_side_nav,
                        "has_breadcrumb": has_breadcrumb
                    }));
                }
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": ctx.run_id.as_str(),
                    "text": if failures.is_empty() { "docs ux smoke passed" } else { "docs ux smoke failed" },
                    "rows": rows,
                    "sample_set": samples,
                    "site_dir": site_dir.display().to_string(),
                    "errors": failures,
                    "warnings": warnings,
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::SiteDir(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = bijux_dev_atlas::docs::site_output::site_output_report(&ctx.repo_root)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["status"].as_str() == Some("pass") { 0 } else { 1 };
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::Build(common) => {
                let mut build_args = vec!["build".to_string()];
                if common.strict {
                    build_args.push("--strict".to_string());
                }
                let ctx = docs_context(&common)?;
                let generated = docs_verify_generated(&ctx.repo_root)?;
                if generated["status"] != "ok" {
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "kind": "docs_build_preflight",
                        "status": "failed",
                        "text": "generated tutorial snippets are stale; run `bijux-dev-atlas docs generate examples --allow-write --allow-subprocess`",
                        "missing": generated["missing"],
                        "missing_header": generated["missing_header"],
                        "stale": generated["stale"],
                        "duration_ms": started.elapsed().as_millis() as u64
                    });
                    return Ok((emit_payload(common.format, common.out, &payload)?, 1));
                }
                let (mut payload, code) =
                    docs_build_or_serve_subprocess(&build_args, &common, "docs build")?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::Serve(args) => {
                if !args.common.allow_network {
                    return Err("docs serve requires --allow-network".to_string());
                }
                let (mut payload, code) = docs_build_or_serve_subprocess(
                    &[
                        "serve".to_string(),
                        "--dev-addr".to_string(),
                        format!("{}:{}", args.host, args.port),
                    ],
                    &args.common,
                    "docs serve",
                )?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((
                    emit_payload(args.common.format, args.common.out, &payload)?,
                    code,
                ))
            }
            DocsCommand::Clean(common) => {
                if !common.allow_write {
                    return Err("docs clean requires --allow-write".to_string());
                }
                let ctx = docs_context(&common)?;
                let target = ctx.artifacts_root.join("atlas-dev").join("docs");
                if target.exists() {
                    fs::remove_dir_all(&target)
                        .map_err(|e| format!("failed to remove {}: {e}", target.display()))?;
                }
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": ctx.run_id.as_str(),
                    "text": format!("docs clean removed {}", target.display()),
                    "rows": [{"path": target.display().to_string()}],
                    "capabilities":{"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "options":{"strict": common.strict, "include_drafts": common.include_drafts},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            DocsCommand::Doctor(common) => {
                let ctx = docs_context(&common)?;
                let validate = docs_validate_payload(&ctx, &common)?;
                let links = docs_links_payload(&ctx, &common)?;
                let lint = docs_lint_payload(&ctx, &common)?;
                let mut rows = Vec::<serde_json::Value>::new();
                rows.push(serde_json::json!({"name":"validate","errors":validate["errors"].as_array().map(|v| v.len()).unwrap_or(0)}));
                rows.push(serde_json::json!({"name":"links","errors":links["errors"].as_array().map(|v| v.len()).unwrap_or(0)}));
                rows.push(serde_json::json!({"name":"lint","errors":lint["errors"].as_array().map(|v| v.len()).unwrap_or(0)}));
                let mut build_status = "skipped";
                let mut site_output = serde_json::json!({
                    "status": "skipped",
                    "counts": {"file_count": 0, "minimum_file_count": 0},
                    "checks": []
                });
                if common.allow_subprocess && common.allow_write {
                    let (_payload, code) = docs_build_or_serve_subprocess(
                        &["build".to_string()],
                        &common,
                        "docs build",
                    )?;
                    build_status = if code == 0 { "ok" } else { "failed" };
                    site_output = bijux_dev_atlas::docs::site_output::site_output_report(&ctx.repo_root)?;
                }
                let site_output_status = site_output["status"].as_str().unwrap_or(
                    if build_status == "skipped" {
                        "skipped"
                    } else {
                        "missing"
                    },
                );
                rows.push(serde_json::json!({"name":"build","status":build_status}));
                rows.push(serde_json::json!({
                    "name":"site_output",
                    "status": site_output_status,
                    "file_count": site_output["counts"]["file_count"].as_u64().unwrap_or(0),
                    "minimum_file_count": site_output["counts"]["minimum_file_count"].as_u64().unwrap_or(0)
                }));
                let errors = validate["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + links["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + lint["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + usize::from(build_status == "failed")
                    + usize::from(site_output_status == "fail");
                let closure_summary = serde_json::json!({
                    "report_id": "docs-build-closure-summary",
                    "version": 1,
                    "inputs": {
                        "links_report": "docs links",
                        "site_output_report": "docs-site-output"
                    },
                    "summary": {
                        "error_count": errors,
                        "build_status": build_status
                    },
                    "evidence": {
                        "links_errors": links["errors"].as_array().map(|v| v.len()).unwrap_or(0),
                        "site_output_status": site_output_status
                    },
                    "checks": [
                        {"name": "links", "status": if links["errors"].as_array().is_some_and(|v| v.is_empty()) { "pass" } else { "fail" }},
                        {"name": "site_output", "status": site_output_status}
                    ],
                    "status": if links["errors"].as_array().is_some_and(|v| v.is_empty())
                        && site_output_status != "fail"
                    {
                        "pass"
                    } else {
                        "fail"
                    }
                });
                if common.allow_write {
                    let out_dir = docs_gate_artifact_dir(&ctx);
                    let closure_summary_path = out_dir.join("docs-build-closure-summary.json");
                    let site_output_path = out_dir.join("site-output.json");
                    let manifest_path = out_dir.join("report-manifest.json");
                    let closure_index_json_path =
                        ctx.repo_root.join("docs/_internal/generated/closure-index.json");
                    let closure_index_md_path =
                        ctx.repo_root.join("docs/_internal/generated/closure-index.md");
                    let closure_index = bijux_dev_atlas::docs::site_output::closure_index_report();
                    let closure_index_markdown =
                        bijux_dev_atlas::docs::site_output::closure_index_markdown(&closure_index)?;
                    bijux_dev_atlas::docs::site_output::validate_named_report(
                        &ctx.repo_root,
                        "docs-site-output.schema.json",
                        &site_output,
                    )?;
                    bijux_dev_atlas::docs::site_output::validate_named_report(
                        &ctx.repo_root,
                        "closure-summary.schema.json",
                        &closure_summary,
                    )?;
                    let manifest = bijux_dev_atlas::docs::site_output::report_manifest(&[
                        ("closure-index", "docs/_internal/generated/closure-index.json"),
                        ("docs-build-closure-summary", "docs-build-closure-summary.json"),
                        ("docs-site-output", "site-output.json"),
                    ]);
                    write_docs_gate_artifact(&site_output_path, &site_output)?;
                    write_docs_gate_artifact(&closure_summary_path, &closure_summary)?;
                    write_docs_gate_artifact(&manifest_path, &manifest)?;
                    write_docs_gate_artifact(&closure_index_json_path, &closure_index)?;
                    if let Some(parent) = closure_index_md_path.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
                    }
                    fs::write(&closure_index_md_path, closure_index_markdown)
                        .map_err(|e| format!("write {} failed: {e}", closure_index_md_path.display()))?;
                }
                let payload = serde_json::json!({
                    "schema_version":1,
                    "run_id":ctx.run_id.as_str(),
                    "text": if errors==0 {
                        format!("docs: 5 checks collected, 0 failed, build={build_status}")
                    } else {
                        format!("docs: 5 checks collected, {errors} failed, build={build_status}")
                    },
                    "rows":rows,
                    "counts":{"errors":errors},
                    "site_output": site_output,
                    "artifacts": {
                        "closure_summary": docs_gate_artifact_dir(&ctx).join("docs-build-closure-summary.json").display().to_string(),
                        "site_output": docs_gate_artifact_dir(&ctx).join("site-output.json").display().to_string(),
                        "report_manifest": docs_gate_artifact_dir(&ctx).join("report-manifest.json").display().to_string(),
                        "closure_index": ctx.repo_root.join("docs/_internal/generated/closure-index.json").display().to_string()
                    },
                    "capabilities":{"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "options":{"strict": common.strict, "include_drafts": common.include_drafts},
                    "duration_ms": started.elapsed().as_millis() as u64,
                    "error_code": if errors == 0 { serde_json::Value::Null } else { serde_json::Value::String("DOCS_NAV_ERROR".to_string()) }
                });
                Ok((
                    emit_payload(common.format, common.out, &payload)?,
                    if errors == 0 { 0 } else { 1 },
                ))
            }
            DocsCommand::VerifyGenerated(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_verify_generated(&ctx.repo_root)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["status"] == "ok" { 0 } else { 1 };
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
        }
    })();
    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                if code == 0 {
                    let _ = writeln!(io::stdout(), "{rendered}");
                } else {
                    let _ = writeln!(io::stderr(), "{rendered}");
                }
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas docs failed: {err}");
            1
        }
    }
}

fn docs_shrink_report_payload(
    ctx: &DocsContext,
    common: &DocsCommonArgs,
) -> Result<serde_json::Value, String> {
    let inventory = docs_inventory_payload(ctx, common)?;
    let mut counts = BTreeMap::<String, usize>::new();
    for row in inventory["rows"].as_array().into_iter().flatten() {
        if let Some(path) = row["path"].as_str() {
            let p = std::path::Path::new(path);
            if p.components().next().and_then(|c| c.as_os_str().to_str()) == Some("docs") {
                let mut it = p.components();
                let _ = it.next();
                if let Some(dir) = it.next().and_then(|c| c.as_os_str().to_str()) {
                    if dir != "_generated" && dir != "_drafts" {
                        *counts.entry(dir.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    let mut top_directories = counts
        .iter()
        .map(|(dir, count)| serde_json::json!({"directory": dir, "markdown_count": count}))
        .collect::<Vec<_>>();
    top_directories.sort_by(|a, b| {
        b["markdown_count"]
            .as_u64()
            .cmp(&a["markdown_count"].as_u64())
            .then(a["directory"].as_str().cmp(&b["directory"].as_str()))
    });
    let max_md_per_dir = counts.values().copied().max().unwrap_or(0);
    let status = if max_md_per_dir > 40 { "fail" } else { "pass" };
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "status": status,
        "max_md_per_dir": max_md_per_dir,
        "top_directories": top_directories,
        "budget": { "max_md_per_dir": 40 },
        "text": if status == "pass" { "docs shrink report passed" } else { "docs shrink report failed" }
    }))
}
