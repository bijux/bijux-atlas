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

