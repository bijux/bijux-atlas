fn test_docs_026_unique_section_links(ctx: &RunContext) -> TestResult {
    let targets = match docs_root_index_targets(ctx, "DOC-026", "docs.index.unique_section_links") {
        Ok(targets) => targets,
        Err(result) => return result,
    };
    let mut seen = std::collections::BTreeSet::new();
    let mut violations = Vec::new();
    for target in targets {
        if !target.ends_with("/index.md") {
            continue;
        }
        if !seen.insert(target.clone()) {
            push_docs_violation(
                &mut violations,
                "DOC-026",
                "docs.index.unique_section_links",
                Some("docs/index.md".to_string()),
                format!("duplicate section index link is forbidden: {target}"),
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_docs_027_section_indexes_list_local_pages(ctx: &RunContext) -> TestResult {
    let payload = match docs_sections_payload(ctx, "DOC-027", "docs.index.section_indexes_list_local_pages") {
        Ok(payload) => payload,
        Err(result) => return result,
    };
    let section_map = match payload["sections"].as_object() {
        Some(map) => map,
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-027".to_string(),
                test_id: "docs.index.section_indexes_list_local_pages".to_string(),
                file: Some("docs/_internal/registry/sections.json".to_string()),
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
        if name.starts_with('_') {
            continue;
        }
        let index_path = docs_root.join(name).join("INDEX.md");
        let contents = match std::fs::read_to_string(&index_path) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-027",
                    "docs.index.section_indexes_list_local_pages",
                    Some(format!("docs/{name}/index.md")),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        let has_local_markdown_link = markdown_links(&contents).into_iter().any(|target| {
            !target.starts_with("http://")
                && !target.starts_with("https://")
                && !target.starts_with('#')
                && !target.starts_with("mailto:")
                && target.ends_with(".md")
        });
        if !has_local_markdown_link {
            push_docs_violation(
                &mut violations,
                "DOC-027",
                "docs.index.section_indexes_list_local_pages",
                Some(format!("docs/{name}/index.md")),
                "required section index must link at least one local markdown page",
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}
