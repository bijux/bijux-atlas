fn markdown_links_for_reports(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut links = Vec::new();
    let mut index = 0usize;
    while index + 3 < bytes.len() {
        if bytes[index] != b'[' {
            index += 1;
            continue;
        }
        let Some(close_bracket_rel) = text[index..].find("](") else {
            index += 1;
            continue;
        };
        let open_paren = index + close_bracket_rel + 1;
        let Some(close_paren_rel) = text[open_paren + 1..].find(')') else {
            index += 1;
            continue;
        };
        let target = &text[open_paren + 1..open_paren + 1 + close_paren_rel];
        links.push(target.to_string());
        index = open_paren + 1 + close_paren_rel + 1;
    }
    links
}

fn markdown_targets_in_docs(
    ctx: &RunContext,
    source: &std::path::Path,
    contents: &str,
) -> Vec<(String, std::path::PathBuf)> {
    let mut out = Vec::new();
    for target in markdown_links_for_reports(contents) {
        if target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with('#')
            || target.starts_with("mailto:")
        {
            continue;
        }
        let clean = target.split('#').next().unwrap_or(&target);
        if clean.is_empty() {
            continue;
        }
        let resolved = if clean.starts_with('/') {
            ctx.repo_root.join(clean.trim_start_matches('/'))
        } else {
            source.parent().unwrap_or(source).join(clean)
        };
        if resolved.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        out.push((target, resolved));
    }
    out
}

fn parse_markdown_field(contents: &str, field: &str) -> Option<String> {
    let needle = format!("{field}:");
    for line in contents.lines().take(32) {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix(&needle) {
            let normalized = value.trim().trim_matches('"').trim_matches('\'').trim();
            if !normalized.is_empty() {
                return Some(normalized.to_string());
            }
        }
    }
    parse_docs_field(contents, &[field])
}

fn markdown_h1(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("# ") {
            let value = rest.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn docs_broken_links_report(ctx: &RunContext) -> serde_json::Value {
    let mut rows = Vec::new();
    for path in docs_markdown_files(ctx) {
        let Ok(contents) = std::fs::read_to_string(&path) else {
            continue;
        };
        let source = match path.strip_prefix(&ctx.repo_root) {
            Ok(value) => value.display().to_string(),
            Err(_) => continue,
        };
        for (target, resolved) in markdown_targets_in_docs(ctx, &path, &contents) {
            if !resolved.exists() {
                rows.push(serde_json::json!({
                    "source": source,
                    "target": target,
                    "resolved_path": match resolved.strip_prefix(&ctx.repo_root) {
                        Ok(value) => value.display().to_string(),
                        Err(_) => resolved.display().to_string(),
                    }
                }));
            }
        }
    }
    rows.sort_by(|a, b| {
        a["source"]
            .as_str()
            .cmp(&b["source"].as_str())
            .then(a["target"].as_str().cmp(&b["target"].as_str()))
    });
    serde_json::json!({
        "schema_version": 1,
        "kind": "docs_broken_links",
        "broken_links": rows
    })
}

fn docs_orphans_report(ctx: &RunContext) -> serde_json::Value {
    let mut all = std::collections::BTreeSet::<String>::new();
    for path in docs_markdown_files(ctx) {
        if let Ok(rel) = path.strip_prefix(&ctx.repo_root) {
            all.insert(rel.display().to_string());
        }
    }
    let mut reachable = std::collections::BTreeSet::<String>::new();
    let mut queue = std::collections::VecDeque::<String>::new();
    for seed in ["docs/index.md", "docs/INDEX.md", "docs/START_HERE.md"] {
        if all.contains(seed) && reachable.insert(seed.to_string()) {
            queue.push_back(seed.to_string());
        }
    }
    while let Some(current) = queue.pop_front() {
        let path = ctx.repo_root.join(&current);
        let Ok(contents) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (_, target_path) in markdown_targets_in_docs(ctx, &path, &contents) {
            let Ok(rel) = target_path.strip_prefix(&ctx.repo_root) else {
                continue;
            };
            let rel = rel.display().to_string();
            if all.contains(&rel) && reachable.insert(rel.clone()) {
                queue.push_back(rel);
            }
        }
    }
    let orphans = all
        .iter()
        .filter(|path| !reachable.contains(*path))
        .cloned()
        .collect::<Vec<_>>();
    serde_json::json!({
        "schema_version": 1,
        "kind": "docs_orphans",
        "reachable_count": reachable.len(),
        "total_markdown_files": all.len(),
        "orphans": orphans
    })
}

fn docs_metadata_coverage_report(ctx: &RunContext) -> serde_json::Value {
    let mut rows = Vec::new();
    let mut title_count = 0usize;
    let mut owner_count = 0usize;
    let mut status_count = 0usize;
    let mut audience_count = 0usize;
    for path in docs_markdown_files(ctx) {
        let Ok(contents) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Some(relative) = path
            .strip_prefix(&ctx.repo_root)
            .ok()
            .map(|value| value.display().to_string())
        else {
            continue;
        };
        let title = parse_markdown_field(&contents, "title").is_some();
        let owner = parse_markdown_field(&contents, "owner").is_some();
        let status = parse_markdown_field(&contents, "status").is_some();
        let audience = parse_markdown_field(&contents, "audience").is_some();
        title_count += usize::from(title);
        owner_count += usize::from(owner);
        status_count += usize::from(status);
        audience_count += usize::from(audience);
        rows.push(serde_json::json!({
            "path": relative,
            "title": title,
            "owner": owner,
            "status": status,
            "audience": audience
        }));
    }
    rows.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let total = rows.len();
    serde_json::json!({
        "schema_version": 1,
        "kind": "docs_metadata_coverage",
        "total_markdown_files": total,
        "title_coverage": title_count,
        "owner_coverage": owner_count,
        "status_coverage": status_count,
        "audience_coverage": audience_count,
        "files": rows
    })
}

fn docs_duplication_report(ctx: &RunContext) -> serde_json::Value {
    let mut by_title = std::collections::BTreeMap::<String, Vec<String>>::new();
    for path in docs_markdown_files(ctx) {
        let Ok(contents) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Some(h1) = markdown_h1(&contents) else {
            continue;
        };
        let normalized = h1.to_lowercase();
        let Some(relative) = path
            .strip_prefix(&ctx.repo_root)
            .ok()
            .map(|value| value.display().to_string())
        else {
            continue;
        };
        by_title.entry(normalized).or_default().push(relative);
    }
    let mut duplicates = Vec::new();
    for (title, mut files) in by_title {
        files.sort();
        if files.len() > 1 {
            duplicates.push(serde_json::json!({
                "normalized_title": title,
                "count": files.len(),
                "files": files
            }));
        }
    }
    serde_json::json!({
        "schema_version": 1,
        "kind": "docs_duplication",
        "duplicate_titles": duplicates
    })
}

fn test_docs_032_broken_links_report_generated(ctx: &RunContext) -> TestResult {
    let payload = docs_broken_links_report(ctx);
    write_docs_report_artifact(
        ctx,
        "DOC-032",
        "docs.reports.broken_links_generated",
        "broken-links.json",
        &payload,
    )
}

fn test_docs_033_orphans_report_generated(ctx: &RunContext) -> TestResult {
    let payload = docs_orphans_report(ctx);
    write_docs_report_artifact(
        ctx,
        "DOC-033",
        "docs.reports.orphans_generated",
        "orphans.json",
        &payload,
    )
}

fn test_docs_034_metadata_coverage_report_generated(ctx: &RunContext) -> TestResult {
    let payload = docs_metadata_coverage_report(ctx);
    write_docs_report_artifact(
        ctx,
        "DOC-034",
        "docs.reports.metadata_coverage_generated",
        "metadata-coverage.json",
        &payload,
    )
}

fn test_docs_035_duplication_report_generated(ctx: &RunContext) -> TestResult {
    let payload = docs_duplication_report(ctx);
    write_docs_report_artifact(
        ctx,
        "DOC-035",
        "docs.reports.duplication_generated",
        "duplication-report.json",
        &payload,
    )
}
