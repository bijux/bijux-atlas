fn docs_root_path(ctx: &RunContext) -> std::path::PathBuf {
    ctx.repo_root.join("docs")
}

const DOCS_ENTRYPOINT_WORD_BUDGET: usize = 700;

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
            pages.push((format!("docs/{name}/INDEX.md"), false));
        }
    }
    Ok(pages)
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

