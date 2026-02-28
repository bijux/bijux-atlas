fn markdown_links(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut links = Vec::new();
    let mut idx = 0usize;
    while idx + 3 < bytes.len() {
        if bytes[idx] == b'[' {
            if let Some(close_bracket) = text[idx..].find("](") {
                let open_paren = idx + close_bracket + 1;
                if let Some(close_paren_rel) = text[open_paren + 1..].find(')') {
                    let target = &text[open_paren + 1..open_paren + 1 + close_paren_rel];
                    links.push(target.to_string());
                    idx = open_paren + 1 + close_paren_rel + 1;
                    continue;
                }
            }
        }
        idx += 1;
    }
    links
}

fn validate_markdown_links(
    ctx: &RunContext,
    source: &std::path::Path,
    contract_id: &str,
    test_id: &str,
    violations: &mut Vec<Violation>,
) {
    let contents = match std::fs::read_to_string(source) {
        Ok(contents) => contents,
        Err(err) => {
            let rel = source
                .strip_prefix(&ctx.repo_root)
                .map(|value| value.display().to_string())
                .unwrap_or_else(|_| source.display().to_string());
            push_docs_violation(
                violations,
                contract_id,
                test_id,
                Some(rel),
                format!("read failed: {err}"),
            );
            return;
        }
    };
    let base_dir = source.parent().unwrap_or(source);
    for target in markdown_links(&contents) {
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
        let resolved = base_dir.join(clean);
        if !resolved.exists() {
            let rel = source
                .strip_prefix(&ctx.repo_root)
                .map(|value| value.display().to_string())
                .unwrap_or_else(|_| source.display().to_string());
            push_docs_violation(
                violations,
                contract_id,
                test_id,
                Some(rel),
                format!("broken relative link target: {target}"),
            );
        }
    }
}

fn test_docs_011_section_index_links_resolve(ctx: &RunContext) -> TestResult {
    let payload = match docs_sections_payload(ctx, "DOC-011", "docs.links.section_indexes_resolve") {
        Ok(payload) => payload,
        Err(result) => return result,
    };
    let section_map = match payload["sections"].as_object() {
        Some(map) => map,
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-011".to_string(),
                test_id: "docs.links.section_indexes_resolve".to_string(),
                file: Some("docs/sections.json".to_string()),
                line: None,
                message: "`sections` object is required".to_string(),
                evidence: None,
            }])
        }
    };
    let mut violations = Vec::new();
    let docs_root = docs_root_path(ctx);
    for (name, config) in section_map {
        if !config["requires_index"].as_bool().unwrap_or(false) {
            continue;
        }
        validate_markdown_links(
            ctx,
            &docs_root.join(name).join("INDEX.md"),
            "DOC-011",
            "docs.links.section_indexes_resolve",
            &mut violations,
        );
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_docs_012_root_entrypoint_links_resolve(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for relative in ["docs/index.md", "docs/START_HERE.md"] {
        validate_markdown_links(
            ctx,
            &ctx.repo_root.join(relative),
            "DOC-012",
            "docs.links.root_entrypoints_resolve",
            &mut violations,
        );
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_docs_024_no_absolute_local_paths(ctx: &RunContext) -> TestResult {
    let pages = match docs_entrypoint_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    for (relative, _) in pages {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-024",
                    "docs.links.no_absolute_local_paths",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        for target in markdown_links(&contents) {
            if target.starts_with("/Users/") || target.starts_with("file://") {
                push_docs_violation(
                    &mut violations,
                    "DOC-024",
                    "docs.links.no_absolute_local_paths",
                    Some(relative.clone()),
                    format!("absolute local file link is forbidden: {target}"),
                );
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_docs_025_no_raw_http_links(ctx: &RunContext) -> TestResult {
    let pages = match docs_entrypoint_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    for (relative, _) in pages {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-025",
                    "docs.links.no_raw_http",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        for target in markdown_links(&contents) {
            if target.starts_with("http://") {
                push_docs_violation(
                    &mut violations,
                    "DOC-025",
                    "docs.links.no_raw_http",
                    Some(relative.clone()),
                    format!("raw http link is forbidden: {target}"),
                );
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_docs_028_section_indexes_unique_local_pages(ctx: &RunContext) -> TestResult {
    let payload = match docs_sections_payload(ctx, "DOC-028", "docs.index.section_indexes_unique_local_pages") {
        Ok(payload) => payload,
        Err(result) => return result,
    };
    let section_map = match payload["sections"].as_object() {
        Some(map) => map,
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-028".to_string(),
                test_id: "docs.index.section_indexes_unique_local_pages".to_string(),
                file: Some("docs/sections.json".to_string()),
                line: None,
                message: "`sections` object is required".to_string(),
                evidence: None,
            }])
        }
    };
    let docs_root = docs_root_path(ctx);
    let mut violations = Vec::new();
    for (name, config) in section_map {
        if name.starts_with('_') || !config["requires_index"].as_bool().unwrap_or(false) {
            continue;
        }
        let index_path = docs_root.join(name).join("INDEX.md");
        let contents = match std::fs::read_to_string(&index_path) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-028",
                    "docs.index.section_indexes_unique_local_pages",
                    Some(format!("docs/{name}/INDEX.md")),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        let mut counts = std::collections::BTreeMap::<String, usize>::new();
        for target in markdown_links(&contents) {
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
            let resolved = index_path.parent().unwrap_or(&index_path).join(clean);
            let normalized = match std::fs::canonicalize(&resolved) {
                Ok(path) => path,
                Err(_) => resolved,
            };
            let rel = match normalized.strip_prefix(&ctx.repo_root) {
                Ok(path) => path.display().to_string(),
                Err(_) => continue,
            };
            let expected_prefix = format!("docs/{name}/");
            if rel.starts_with(&expected_prefix) && rel.ends_with(".md") && rel != format!("docs/{name}/INDEX.md") {
                *counts.entry(rel).or_insert(0) += 1;
            }
        }
        for (rel, count) in counts {
            if count > 1 {
                push_docs_violation(
                    &mut violations,
                    "DOC-028",
                    "docs.index.section_indexes_unique_local_pages",
                    Some(format!("docs/{name}/INDEX.md")),
                    format!("section index links `{rel}` {count} times"),
                );
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_docs_029_root_entrypoints_unique_local_pages(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for relative in ["docs/index.md", "docs/START_HERE.md"] {
        let path = ctx.repo_root.join(relative);
        let contents = match std::fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-029",
                    "docs.index.root_entrypoints_unique_local_pages",
                    Some(relative.to_string()),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        let mut counts = std::collections::BTreeMap::<String, usize>::new();
        for target in markdown_links(&contents) {
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
                path.parent().unwrap_or(&path).join(clean)
            };
            let normalized = match std::fs::canonicalize(&resolved) {
                Ok(path) => path,
                Err(_) => resolved,
            };
            let rel = match normalized.strip_prefix(&ctx.repo_root) {
                Ok(path) => path.display().to_string(),
                Err(_) => continue,
            };
            if rel.ends_with(".md") {
                *counts.entry(rel).or_insert(0) += 1;
            }
        }
        for (target, count) in counts {
            if count > 1 {
                push_docs_violation(
                    &mut violations,
                    "DOC-029",
                    "docs.index.root_entrypoints_unique_local_pages",
                    Some(relative.to_string()),
                    format!("root entrypoint links `{target}` {count} times"),
                );
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}
