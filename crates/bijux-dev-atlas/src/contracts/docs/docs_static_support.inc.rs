fn docs_root_path(ctx: &RunContext) -> std::path::PathBuf {
    ctx.repo_root.join("docs")
}

const DOCS_ENTRYPOINT_WORD_BUDGET: usize = 700;
const DOCS_HOME_LINE_BUDGET: usize = 80;
const DOCS_SECTION_INDEX_LINK_BUDGET: usize = 12;

fn parse_docs_field(contents: &str, labels: &[&str]) -> Option<String> {
    for line in contents.lines().take(12) {
        let trimmed = line.trim();
        for label in labels {
            let prefix = format!("- {label}:");
            if let Some(value) = trimmed.strip_prefix(&prefix) {
                let normalized = value.trim().trim_matches('`').trim().to_string();
                if !normalized.is_empty() {
                    return Some(normalized);
                }
            }
        }
    }
    None
}

fn parse_docs_frontmatter(contents: &str) -> Option<serde_yaml::Value> {
    let mut lines = contents.lines();
    if lines.next()? != "---" {
        return None;
    }
    let mut yaml = String::new();
    for line in lines {
        if line == "---" {
            return serde_yaml::from_str(&yaml).ok();
        }
        yaml.push_str(line);
        yaml.push('\n');
    }
    None
}

fn strip_docs_frontmatter(contents: &str) -> &str {
    let mut lines = contents.lines();
    if lines.next() != Some("---") {
        return contents;
    }
    let mut offset = 4usize;
    for line in lines {
        offset += line.len() + 1;
        if line == "---" {
            return &contents[offset..];
        }
    }
    contents
}

fn docs_frontmatter_string(contents: &str, key: &str) -> Option<String> {
    let value = parse_docs_frontmatter(contents)?;
    value.get(key)?.as_str().map(|value| value.to_string())
}

fn docs_frontmatter_bool(contents: &str, key: &str) -> Option<bool> {
    let value = parse_docs_frontmatter(contents)?;
    value.get(key)?.as_bool()
}

fn docs_frontmatter_list(contents: &str, key: &str) -> Option<Vec<String>> {
    let value = parse_docs_frontmatter(contents)?;
    Some(
        value
            .get(key)?
            .as_sequence()?
            .iter()
            .filter_map(|item| item.as_str().map(|value| value.to_string()))
            .collect(),
    )
}

fn looks_like_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(idx, byte)| matches!(idx, 4 | 7) || byte.is_ascii_digit())
}

fn docs_section_owners_payload(
    ctx: &RunContext,
    contract_id: &str,
    test_id: &str,
) -> Result<serde_json::Value, TestResult> {
    let owners_path = docs_root_path(ctx).join("owners.json");
    let contents = match std::fs::read_to_string(&owners_path) {
        Ok(contents) => contents,
        Err(err) => {
            return Err(TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("docs/owners.json".to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }]))
        }
    };
    match serde_json::from_str(&contents) {
        Ok(payload) => Ok(payload),
        Err(err) => Err(TestResult::Fail(vec![Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some("docs/owners.json".to_string()),
            line: None,
            message: format!("invalid json: {err}"),
            evidence: None,
        }])),
    }
}

fn docs_entrypoint_pages(ctx: &RunContext) -> Result<Vec<(String, bool)>, TestResult> {
    let mut pages = vec![("docs/index.md".to_string(), true)];
    let sections_payload = docs_sections_payload(ctx, "DOC-013", "docs.metadata.entrypoint_owner")?;
    let sections = match sections_payload["sections"].as_object() {
        Some(map) => map,
        None => {
            return Err(TestResult::Fail(vec![Violation {
                contract_id: "DOC-013".to_string(),
                test_id: "docs.metadata.entrypoint_owner".to_string(),
                file: Some("docs/sections.json".to_string()),
                line: None,
                message: "`sections` object is required".to_string(),
                evidence: None,
            }]))
        }
    };
    let mut section_names = sections.keys().cloned().collect::<Vec<_>>();
    section_names.sort();
    for name in section_names {
        let requires_index = sections[&name]["requires_index"].as_bool().unwrap_or(false);
        if requires_index {
            pages.push((format!("docs/{name}/index.md"), false));
        }
    }
    Ok(pages)
}

fn docs_spine_pages(ctx: &RunContext) -> Result<Vec<String>, TestResult> {
    let mut pages = vec![
        "docs/index.md".to_string(),
        "docs/start-here.md".to_string(),
        "docs/glossary.md".to_string(),
    ];
    let entrypoints = docs_entrypoint_pages(ctx)?;
    for (path, is_root_entrypoint) in entrypoints {
        if !is_root_entrypoint && !path.starts_with("docs/_") {
            pages.push(path);
        }
    }
    pages.sort();
    pages.dedup();
    Ok(pages)
}

fn docs_reader_utility_pages() -> Vec<String> {
    vec![
        "docs/site-map.md".to_string(),
        "docs/what-to-read-next.md".to_string(),
    ]
}

fn docs_count_named_files(ctx: &RunContext, file_name: &str) -> usize {
    docs_markdown_files(ctx)
        .into_iter()
        .filter(|path| path.file_name().and_then(|value| value.to_str()) == Some(file_name))
        .count()
}

fn docs_required_section_indexes(ctx: &RunContext) -> Result<Vec<String>, TestResult> {
    let mut indexes = Vec::new();
    for (path, is_root_entrypoint) in docs_entrypoint_pages(ctx)? {
        if !is_root_entrypoint && !path.starts_with("docs/_") {
            indexes.push(path);
        }
    }
    indexes.sort();
    indexes.dedup();
    Ok(indexes)
}

fn read_mkdocs_yaml(
    ctx: &RunContext,
    contract_id: &str,
    test_id: &str,
) -> Result<String, TestResult> {
    let path = ctx.repo_root.join("mkdocs.yml");
    match std::fs::read_to_string(&path) {
        Ok(contents) => Ok(contents),
        Err(err) => Err(TestResult::Fail(vec![Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some("mkdocs.yml".to_string()),
            line: None,
            message: format!("read failed: {err}"),
            evidence: None,
        }])),
    }
}

fn docs_operator_golden_path_pages() -> Vec<String> {
    vec![
        "docs/operations/run-locally.md".to_string(),
        "docs/operations/deploy-kind.md".to_string(),
        "docs/operations/deploy-kubernetes-minimal.md".to_string(),
    ]
}

fn docs_published_markdown_files(ctx: &RunContext) -> Vec<String> {
    let mut files = Vec::new();
    for path in docs_markdown_files(ctx) {
        let Ok(relative) = path.strip_prefix(&ctx.repo_root) else {
            continue;
        };
        let relative = relative.display().to_string();
        if relative.starts_with("docs/_internal/") || relative.starts_with("docs/_drafts/") {
            continue;
        }
        files.push(relative);
    }
    files.sort();
    files
}

fn docs_first_h1(contents: &str) -> Option<String> {
    let mut in_fence = false;
    for line in contents.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some(title) = line.strip_prefix("# ") {
            return Some(title.trim().to_string());
        }
    }
    None
}

fn docs_h1_count(contents: &str) -> usize {
    let mut in_fence = false;
    let mut count = 0usize;
    for line in contents.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if line.starts_with("# ") {
            count += 1;
        }
    }
    count
}

fn docs_root_index_targets(ctx: &RunContext, contract_id: &str, test_id: &str) -> Result<Vec<String>, TestResult> {
    let path = ctx.repo_root.join("docs/index.md");
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) => {
            return Err(TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("docs/index.md".to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }]))
        }
    };
    Ok(markdown_links(&contents))
}

fn push_docs_violation(
    violations: &mut Vec<Violation>,
    contract_id: &str,
    test_id: &str,
    file: impl Into<Option<String>>,
    message: impl Into<String>,
) {
    violations.push(Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file: file.into(),
        line: None,
        message: message.into(),
        evidence: None,
    });
}

fn docs_markdown_files(ctx: &RunContext) -> Vec<std::path::PathBuf> {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                walk(&path, out);
                continue;
            }
            if kind.is_file() && path.extension().and_then(|value| value.to_str()) == Some("md") {
                out.push(path);
            }
        }
    }

    let mut files = Vec::new();
    walk(&docs_root_path(ctx), &mut files);
    files.sort();
    files
}

fn write_docs_report_artifact(
    ctx: &RunContext,
    contract_id: &str,
    test_id: &str,
    artifact_name: &str,
    payload: &serde_json::Value,
) -> TestResult {
    let Some(root) = &ctx.artifacts_root else {
        return TestResult::Pass;
    };
    let path = root.join(artifact_name);
    let encoded = match serde_json::to_string_pretty(payload) {
        Ok(encoded) => encoded,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("docs".to_string()),
                line: None,
                message: format!("encode report failed: {err}"),
                evidence: None,
            }]);
        }
    };
    match std::fs::write(&path, encoded) {
        Ok(()) => TestResult::Pass,
        Err(err) => TestResult::Fail(vec![Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some(path.display().to_string()),
            line: None,
            message: format!("write failed: {err}"),
            evidence: None,
        }]),
    }
}
