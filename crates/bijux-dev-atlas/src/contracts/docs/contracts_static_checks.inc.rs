fn docs_root_path(ctx: &RunContext) -> std::path::PathBuf {
    ctx.repo_root.join("docs")
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

fn test_docs_001_allowed_root_dirs(ctx: &RunContext) -> TestResult {
    let docs_root = docs_root_path(ctx);
    let entries = match std::fs::read_dir(&docs_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-001".to_string(),
                test_id: "docs.surface.allowed_root_dirs".to_string(),
                file: Some("docs".to_string()),
                line: None,
                message: format!("read docs root failed: {err}"),
                evidence: None,
            }])
        }
    };
    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if !path.is_dir() {
            continue;
        }
        let allowed = DOCS_ALLOWED_ROOT_DIRS
            .iter()
            .chain(DOCS_ALLOWED_ROOT_DIRS_TAIL.iter())
            .any(|candidate| *candidate == name);
        if !allowed {
            push_docs_violation(
                &mut violations,
                "DOC-001",
                "docs.surface.allowed_root_dirs",
                Some(format!("docs/{name}")),
                "unexpected top-level docs directory",
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

fn test_docs_002_allowed_root_markdown(ctx: &RunContext) -> TestResult {
    let docs_root = docs_root_path(ctx);
    let entries = match std::fs::read_dir(&docs_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-002".to_string(),
                test_id: "docs.surface.allowed_root_markdown".to_string(),
                file: Some("docs".to_string()),
                line: None,
                message: format!("read docs root failed: {err}"),
                evidence: None,
            }])
        }
    };
    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() || !name.ends_with(".md") {
            continue;
        }
        if !DOCS_ALLOWED_ROOT_MARKDOWN.iter().any(|candidate| *candidate == name) {
            push_docs_violation(
                &mut violations,
                "DOC-002",
                "docs.surface.allowed_root_markdown",
                Some(format!("docs/{name}")),
                "unexpected top-level docs markdown file",
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

fn test_docs_005_no_whitespace_names(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let docs_root = docs_root_path(ctx);
    let mut stack = vec![docs_root.clone()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(err) => {
                let rel = dir
                    .strip_prefix(&ctx.repo_root)
                    .map(|value| value.display().to_string())
                    .unwrap_or_else(|_| dir.display().to_string());
                push_docs_violation(
                    &mut violations,
                    "DOC-005",
                    "docs.naming.no_whitespace",
                    Some(rel),
                    format!("read dir failed: {err}"),
                );
                continue;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.chars().any(|ch| ch.is_whitespace()) {
                let rel = path
                    .strip_prefix(&ctx.repo_root)
                    .map(|value| value.display().to_string())
                    .unwrap_or_else(|_| path.display().to_string());
                push_docs_violation(
                    &mut violations,
                    "DOC-005",
                    "docs.naming.no_whitespace",
                    Some(rel),
                    "docs names may not contain whitespace",
                );
            }
            if path.is_dir() {
                stack.push(path);
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

fn test_docs_006_index_exists(ctx: &RunContext) -> TestResult {
    if docs_root_path(ctx).join("index.md").is_file() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "DOC-006".to_string(),
            test_id: "docs.index.exists".to_string(),
            file: Some("docs/index.md".to_string()),
            line: None,
            message: "docs/index.md is required as the docs entrypoint".to_string(),
            evidence: None,
        }])
    }
}

fn test_docs_007_allowed_root_files(ctx: &RunContext) -> TestResult {
    let docs_root = docs_root_path(ctx);
    let entries = match std::fs::read_dir(&docs_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-007".to_string(),
                test_id: "docs.surface.allowed_root_files".to_string(),
                file: Some("docs".to_string()),
                line: None,
                message: format!("read docs root failed: {err}"),
                evidence: None,
            }])
        }
    };
    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() || name.ends_with(".md") {
            continue;
        }
        if !DOCS_ALLOWED_ROOT_FILES.iter().any(|candidate| *candidate == name) {
            push_docs_violation(
                &mut violations,
                "DOC-007",
                "docs.surface.allowed_root_files",
                Some(format!("docs/{name}")),
                "unexpected top-level docs file",
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

fn test_docs_008_section_owner_coverage(ctx: &RunContext) -> TestResult {
    let owners_path = docs_root_path(ctx).join("owners.json");
    let contents = match std::fs::read_to_string(&owners_path) {
        Ok(contents) => contents,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-008".to_string(),
                test_id: "docs.owners.section_coverage".to_string(),
                file: Some("docs/owners.json".to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }])
        }
    };
    let payload: serde_json::Value = match serde_json::from_str(&contents) {
        Ok(payload) => payload,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-008".to_string(),
                test_id: "docs.owners.section_coverage".to_string(),
                file: Some("docs/owners.json".to_string()),
                line: None,
                message: format!("invalid json: {err}"),
                evidence: None,
            }])
        }
    };
    let section_map = match payload["section_owners"].as_object() {
        Some(map) => map,
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-008".to_string(),
                test_id: "docs.owners.section_coverage".to_string(),
                file: Some("docs/owners.json".to_string()),
                line: None,
                message: "`section_owners` object is required".to_string(),
                evidence: None,
            }])
        }
    };
    let docs_root = docs_root_path(ctx);
    let entries = match std::fs::read_dir(&docs_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-008".to_string(),
                test_id: "docs.owners.section_coverage".to_string(),
                file: Some("docs".to_string()),
                line: None,
                message: format!("read docs root failed: {err}"),
                evidence: None,
            }])
        }
    };
    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !section_map.contains_key(&name) {
            push_docs_violation(
                &mut violations,
                "DOC-008",
                "docs.owners.section_coverage",
                Some(format!("docs/{name}")),
                "top-level docs section is missing an owner mapping",
            );
            continue;
        }
        let owner = &section_map[&name];
        if owner.as_str().is_none() || owner.as_str() == Some("") {
            push_docs_violation(
                &mut violations,
                "DOC-008",
                "docs.owners.section_coverage",
                Some(format!("docs/{name}")),
                "top-level docs section owner must be a non-empty string",
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
