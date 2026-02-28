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
    let payload = match docs_section_owners_payload(ctx, "DOC-008", "docs.owners.section_coverage") {
        Ok(payload) => payload,
        Err(result) => return result,
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

fn docs_sections_payload(
    ctx: &RunContext,
    contract_id: &str,
    test_id: &str,
) -> Result<serde_json::Value, TestResult> {
    let sections_path = docs_root_path(ctx).join("sections.json");
    let contents = match std::fs::read_to_string(&sections_path) {
        Ok(contents) => contents,
        Err(err) => {
            return Err(TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("docs/sections.json".to_string()),
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
            file: Some("docs/sections.json".to_string()),
            line: None,
            message: format!("invalid json: {err}"),
            evidence: None,
        }])),
    }
}

fn test_docs_009_section_manifest_complete(ctx: &RunContext) -> TestResult {
    let payload = match docs_sections_payload(ctx, "DOC-009", "docs.sections.manifest_complete") {
        Ok(payload) => payload,
        Err(result) => return result,
    };
    let section_map = match payload["sections"].as_object() {
        Some(map) => map,
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-009".to_string(),
                test_id: "docs.sections.manifest_complete".to_string(),
                file: Some("docs/sections.json".to_string()),
                line: None,
                message: "`sections` object is required".to_string(),
                evidence: None,
            }])
        }
    };
    let docs_root = docs_root_path(ctx);
    let entries = match std::fs::read_dir(&docs_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-009".to_string(),
                test_id: "docs.sections.manifest_complete".to_string(),
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
                "DOC-009",
                "docs.sections.manifest_complete",
                Some(format!("docs/{name}")),
                "top-level docs section is missing from docs/sections.json",
            );
        }
    }
    for name in section_map.keys() {
        if !docs_root.join(name).is_dir() {
            push_docs_violation(
                &mut violations,
                "DOC-009",
                "docs.sections.manifest_complete",
                Some(format!("docs/{name}")),
                "docs/sections.json references a missing top-level docs section",
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

fn test_docs_010_section_index_policy(ctx: &RunContext) -> TestResult {
    let payload = match docs_sections_payload(ctx, "DOC-010", "docs.sections.index_policy") {
        Ok(payload) => payload,
        Err(result) => return result,
    };
    let section_map = match payload["sections"].as_object() {
        Some(map) => map,
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-010".to_string(),
                test_id: "docs.sections.index_policy".to_string(),
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
        let expects_index = config["requires_index"].as_bool().unwrap_or(false);
        let has_index = docs_root.join(name).join("INDEX.md").is_file();
        if expects_index != has_index {
            push_docs_violation(
                &mut violations,
                "DOC-010",
                "docs.sections.index_policy",
                Some(format!("docs/{name}")),
                if expects_index {
                    "section requires INDEX.md but the file is missing".to_string()
                } else {
                    "section forbids INDEX.md but the file exists".to_string()
                },
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

fn test_docs_013_entrypoint_owner(ctx: &RunContext) -> TestResult {
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
                    "DOC-013",
                    "docs.metadata.entrypoint_owner",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        if parse_docs_field(&contents, &["Owner"]).is_none() {
            push_docs_violation(
                &mut violations,
                "DOC-013",
                "docs.metadata.entrypoint_owner",
                Some(relative),
                "docs entrypoint page must declare `- Owner:` metadata near the top",
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

fn test_docs_014_entrypoint_stability(ctx: &RunContext) -> TestResult {
    let pages = match docs_entrypoint_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    let allowed = ["stable", "evolving", "deprecated"];
    let mut violations = Vec::new();
    for (relative, _) in pages {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-014",
                    "docs.metadata.entrypoint_stability",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        if let Some(value) = parse_docs_field(&contents, &["Stability", "Status"]) {
            if !allowed.iter().any(|candidate| *candidate == value) {
                push_docs_violation(
                    &mut violations,
                    "DOC-014",
                    "docs.metadata.entrypoint_stability",
                    Some(relative),
                    format!("unsupported stability value: {value}"),
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

fn test_docs_015_deprecated_replacement(ctx: &RunContext) -> TestResult {
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
                    "DOC-015",
                    "docs.metadata.deprecated_replacement",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        let is_deprecated = parse_docs_field(&contents, &["Stability", "Status"]).as_deref() == Some("deprecated");
        if is_deprecated {
            let lowered = contents.to_lowercase();
            if !lowered.contains("replacement") && !lowered.contains("canonical page") {
                push_docs_violation(
                    &mut violations,
                    "DOC-015",
                    "docs.metadata.deprecated_replacement",
                    Some(relative),
                    "deprecated docs entrypoint page must name a replacement path",
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

fn test_docs_016_section_owner_alignment(ctx: &RunContext) -> TestResult {
    let owners_payload = match docs_section_owners_payload(ctx, "DOC-016", "docs.metadata.section_owner_alignment") {
        Ok(payload) => payload,
        Err(result) => return result,
    };
    let section_owners = match owners_payload["section_owners"].as_object() {
        Some(map) => map,
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-016".to_string(),
                test_id: "docs.metadata.section_owner_alignment".to_string(),
                file: Some("docs/owners.json".to_string()),
                line: None,
                message: "`section_owners` object is required".to_string(),
                evidence: None,
            }])
        }
    };
    let pages = match docs_entrypoint_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    for (relative, is_root_entrypoint) in pages {
        if is_root_entrypoint {
            continue;
        }
        let section = relative
            .trim_start_matches("docs/")
            .split('/')
            .next()
            .unwrap_or_default()
            .to_string();
        let expected = match section_owners.get(&section).and_then(|value| value.as_str()) {
            Some(value) if !value.is_empty() => value,
            _ => {
                push_docs_violation(
                    &mut violations,
                    "DOC-016",
                    "docs.metadata.section_owner_alignment",
                    Some("docs/owners.json".to_string()),
                    format!("missing section owner mapping for {section}"),
                );
                continue;
            }
        };
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-016",
                    "docs.metadata.section_owner_alignment",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        let actual = parse_docs_field(&contents, &["Owner"]);
        match actual.as_deref() {
            Some(value) if value == expected => {}
            Some(value) => push_docs_violation(
                &mut violations,
                "DOC-016",
                "docs.metadata.section_owner_alignment",
                Some(relative),
                format!("entrypoint owner `{value}` does not match docs/owners.json `{expected}`"),
            ),
            None => push_docs_violation(
                &mut violations,
                "DOC-016",
                "docs.metadata.section_owner_alignment",
                Some(relative),
                "docs entrypoint page must declare `- Owner:` metadata near the top",
            ),
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_docs_017_root_entrypoint_flags(ctx: &RunContext) -> TestResult {
    let payload = match docs_sections_payload(ctx, "DOC-017", "docs.sections.root_entrypoint_flags") {
        Ok(payload) => payload,
        Err(result) => return result,
    };
    let section_map = match payload["sections"].as_object() {
        Some(map) => map,
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-017".to_string(),
                test_id: "docs.sections.root_entrypoint_flags".to_string(),
                file: Some("docs/sections.json".to_string()),
                line: None,
                message: "`sections` object is required".to_string(),
                evidence: None,
            }])
        }
    };
    let mut violations = Vec::new();
    for (name, config) in section_map {
        if !config["requires_index"].as_bool().unwrap_or(false) {
            continue;
        }
        if config.get("root_entrypoint").and_then(|value| value.as_bool()).is_none() {
            push_docs_violation(
                &mut violations,
                "DOC-017",
                "docs.sections.root_entrypoint_flags",
                Some("docs/sections.json".to_string()),
                format!("indexed section `{name}` must declare boolean `root_entrypoint`"),
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

fn test_docs_018_root_section_coverage(ctx: &RunContext) -> TestResult {
    let payload = match docs_sections_payload(ctx, "DOC-018", "docs.index.root_section_coverage") {
        Ok(payload) => payload,
        Err(result) => return result,
    };
    let section_map = match payload["sections"].as_object() {
        Some(map) => map,
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-018".to_string(),
                test_id: "docs.index.root_section_coverage".to_string(),
                file: Some("docs/sections.json".to_string()),
                line: None,
                message: "`sections` object is required".to_string(),
                evidence: None,
            }])
        }
    };
    let targets = match docs_root_index_targets(ctx, "DOC-018", "docs.index.root_section_coverage") {
        Ok(targets) => targets,
        Err(result) => return result,
    };
    let target_set = targets.into_iter().collect::<std::collections::BTreeSet<_>>();
    let mut violations = Vec::new();
    for (name, config) in section_map {
        if !config["requires_index"].as_bool().unwrap_or(false) {
            continue;
        }
        if !config["root_entrypoint"].as_bool().unwrap_or(false) {
            continue;
        }
        let expected = format!("{name}/INDEX.md");
        if !target_set.contains(&expected) {
            push_docs_violation(
                &mut violations,
                "DOC-018",
                "docs.index.root_section_coverage",
                Some("docs/index.md".to_string()),
                format!("docs/index.md must link `{expected}`"),
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

fn test_docs_019_entrypoint_word_budget(ctx: &RunContext) -> TestResult {
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
                    "DOC-019",
                    "docs.quality.entrypoint_word_budget",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        let word_count = contents.split_whitespace().count();
        if word_count > DOCS_ENTRYPOINT_WORD_BUDGET {
            push_docs_violation(
                &mut violations,
                "DOC-019",
                "docs.quality.entrypoint_word_budget",
                Some(relative),
                format!(
                    "entrypoint page word count {word_count} exceeds budget {DOCS_ENTRYPOINT_WORD_BUDGET}"
                ),
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

fn test_docs_020_no_placeholders(ctx: &RunContext) -> TestResult {
    let pages = match docs_entrypoint_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    let markers = ["TODO", "TBD", "XXX"];
    let mut violations = Vec::new();
    for (relative, _) in pages {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-020",
                    "docs.quality.no_placeholders",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        let is_stable = parse_docs_field(&contents, &["Stability", "Status"]).as_deref() == Some("stable");
        if !is_stable {
            continue;
        }
        for marker in markers {
            if contents.contains(marker) {
                push_docs_violation(
                    &mut violations,
                    "DOC-020",
                    "docs.quality.no_placeholders",
                    Some(relative.clone()),
                    format!("stable entrypoint page contains placeholder marker `{marker}`"),
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

fn test_docs_021_no_tabs(ctx: &RunContext) -> TestResult {
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
                    "DOC-021",
                    "docs.format.no_tabs",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        if contents.contains('\t') {
            push_docs_violation(
                &mut violations,
                "DOC-021",
                "docs.format.no_tabs",
                Some(relative),
                "entrypoint page contains raw tab characters",
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

fn test_docs_022_no_trailing_whitespace(ctx: &RunContext) -> TestResult {
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
                    "DOC-022",
                    "docs.format.no_trailing_whitespace",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        if contents.lines().any(|line| line.ends_with(' ') || line.ends_with('\t')) {
            push_docs_violation(
                &mut violations,
                "DOC-022",
                "docs.format.no_trailing_whitespace",
                Some(relative),
                "entrypoint page contains trailing whitespace",
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

fn test_docs_023_single_h1(ctx: &RunContext) -> TestResult {
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
                    "DOC-023",
                    "docs.structure.single_h1",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        let h1_count = contents.lines().filter(|line| line.starts_with("# ")).count();
        if h1_count != 1 {
            push_docs_violation(
                &mut violations,
                "DOC-023",
                "docs.structure.single_h1",
                Some(relative),
                format!("entrypoint page must contain exactly one H1 heading, found {h1_count}"),
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

fn test_docs_026_unique_section_links(ctx: &RunContext) -> TestResult {
    let targets = match docs_root_index_targets(ctx, "DOC-026", "docs.index.unique_section_links") {
        Ok(targets) => targets,
        Err(result) => return result,
    };
    let mut seen = std::collections::BTreeSet::new();
    let mut violations = Vec::new();
    for target in targets {
        if !target.ends_with("/INDEX.md") {
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
                    Some(format!("docs/{name}/INDEX.md")),
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
                Some(format!("docs/{name}/INDEX.md")),
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
