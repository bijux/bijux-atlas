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

#[derive(serde::Deserialize)]
struct DocsConceptRegistry {
    concepts: Vec<DocsConceptRow>,
}

#[derive(serde::Deserialize)]
struct DocsConceptRow {
    id: String,
    canonical: String,
    #[serde(default)]
    pointers: Vec<String>,
}

fn load_docs_concepts(repo_root: &std::path::Path) -> Result<Vec<DocsConceptRow>, String> {
    let path = repo_root.join("docs/_internal/style/concepts.yml");
    let text =
        fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let payload: DocsConceptRegistry = serde_yaml::from_str(&text)
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;
    Ok(payload.concepts)
}

fn render_concept_registry_markdown(rows: &[DocsConceptRow]) -> String {
    let mut out = String::from("# Concept Registry\n\n");
    out.push_str("Generated from `docs/_internal/style/concepts.yml`.\n\n");
    out.push_str("| Concept ID | Canonical Page | Pointer Pages |\n|---|---|---|\n");
    for row in rows {
        let pointers = if row.pointers.is_empty() {
            "`none`".to_string()
        } else {
            row.pointers
                .iter()
                .map(|pointer| format!("`{pointer}`"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        out.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            row.id, row.canonical, pointers
        ));
    }
    out
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

fn docs_generate_health_dashboard(repo_root: &std::path::Path) -> Result<serde_json::Value, String> {
    let docs_root = repo_root.join("docs");
    let output_path = docs_root.join("_internal/generated/docs-health-dashboard.md");
    let allowlist_path = repo_root.join("configs/docs/external-link-allowlist.json");
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
        "- Owner: `docs-governance`".to_string(),
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

include!("docs_command_router_registry.inc.rs");

fn write_generated_docs_file(
    repo_root: &std::path::Path,
    rel_path: &str,
    body: String,
) -> Result<String, String> {
    let path = repo_root.join(rel_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {} failed: {e}", parent.display()))?;
    }
    let header =
        "<!-- autogenerated: bijux-dev-atlas docs generate -->\n<!-- do not edit by hand -->\n\n";
    fs::write(&path, format!("{header}{body}"))
        .map_err(|e| format!("write {} failed: {e}", path.display()))?;
    Ok(rel_path.to_string())
}

fn docs_generate_command_lists(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
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
    let content = format!(
        "# Generated Command Lists\n\n## bijux-dev-atlas\n\n```text\n{}\n```\n\n## make wrappers\n\n```text\n{}\n```\n",
        String::from_utf8(root_help.stdout).map_err(|e| e.to_string())?,
        String::from_utf8(make_list.stdout).map_err(|e| e.to_string())?,
    );
    Ok(vec![write_generated_docs_file(
        repo_root,
        "docs/_generated/command-lists.md",
        content,
    )?])
}

fn docs_generate_schema_snippets(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let schema_path =
        repo_root.join("crates/bijux-atlas-server/docs/generated/runtime-startup-config.schema.json");
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
    let content = format!(
        "# Generated Schema Snippets\n\nRuntime startup schema source: `crates/bijux-atlas-server/docs/generated/runtime-startup-config.schema.json`\n\nRequired fields:\n\n{}\n",
        required
            .iter()
            .map(|item| format!("- `{item}`"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    Ok(vec![write_generated_docs_file(
        repo_root,
        "docs/_generated/schema-snippets.md",
        content,
    )?])
}

fn docs_generate_openapi_snippets(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let openapi_path = repo_root.join("configs/openapi/v1/openapi.generated.json");
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
    let content = format!(
        "# Generated OpenAPI Snippets\n\nOpenAPI source: `configs/openapi/v1/openapi.generated.json`\n\nSample endpoint paths:\n\n{}\n",
        sample
            .iter()
            .map(|path| format!("- `{path}`"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    Ok(vec![write_generated_docs_file(
        repo_root,
        "docs/_generated/openapi-snippets.md",
        content,
    )?])
}

fn docs_generate_ops_snippets(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
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
    let content = format!(
        "# Generated Ops Snippets\n\nKnown values files:\n\n{}\n",
        values_files
            .iter()
            .map(|path| format!("- `{path}`"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    Ok(vec![write_generated_docs_file(
        repo_root,
        "docs/_generated/ops-snippets.md",
        content,
    )?])
}

fn docs_generate_examples(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let content = [
        "# Generated Examples Index".to_string(),
        "".to_string(),
        "- [Command lists](command-lists.md)".to_string(),
        "- [Schema snippets](schema-snippets.md)".to_string(),
        "- [OpenAPI snippets](openapi-snippets.md)".to_string(),
        "- [Ops snippets](ops-snippets.md)".to_string(),
        "".to_string(),
    ]
    .join("\n");
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
    generated.extend(docs_generate_examples(repo_root)?);
    Ok(generated)
}

fn docs_verify_generated(repo_root: &std::path::Path) -> Result<serde_json::Value, String> {
    let required = [
        "docs/_generated/examples.md",
        "docs/_generated/command-lists.md",
        "docs/_generated/schema-snippets.md",
        "docs/_generated/openapi-snippets.md",
        "docs/_generated/ops-snippets.md",
    ];
    let missing = required
        .iter()
        .filter(|rel| !repo_root.join(rel).exists())
        .map(|rel| (*rel).to_string())
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "docs_verify_generated",
        "status": if missing.is_empty() { "ok" } else { "failed" },
        "required": required,
        "missing": missing,
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
            DocsCommand::Registry { command } => run_docs_registry_command(&started, command),
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
                rows.push(serde_json::json!({"name":"build","status":build_status}));
                rows.push(serde_json::json!({
                    "name":"site_output",
                    "status": site_output["status"].as_str().unwrap_or("unknown"),
                    "file_count": site_output["counts"]["file_count"].as_u64().unwrap_or(0),
                    "minimum_file_count": site_output["counts"]["minimum_file_count"].as_u64().unwrap_or(0)
                }));
                let errors = validate["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + links["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + lint["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + usize::from(build_status == "failed")
                    + usize::from(site_output["status"].as_str() == Some("fail"));
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
                        "site_output_status": site_output["status"].as_str().unwrap_or("unknown")
                    },
                    "checks": [
                        {"name": "links", "status": if links["errors"].as_array().is_some_and(|v| v.is_empty()) { "pass" } else { "fail" }},
                        {"name": "site_output", "status": site_output["status"].as_str().unwrap_or("skipped")}
                    ],
                    "status": if links["errors"].as_array().is_some_and(|v| v.is_empty())
                        && site_output["status"].as_str() != Some("fail")
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
