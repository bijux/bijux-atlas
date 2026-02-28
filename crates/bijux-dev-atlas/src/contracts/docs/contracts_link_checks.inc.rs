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
