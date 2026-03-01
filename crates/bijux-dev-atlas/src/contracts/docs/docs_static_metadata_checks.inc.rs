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
        if parse_docs_field(&contents, &["Owner"]).is_none()
            && docs_frontmatter_string(&contents, "owner").is_none()
        {
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
        let actual = parse_docs_field(&contents, &["Owner"])
            .or_else(|| docs_frontmatter_string(&contents, "owner"));
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

fn test_docs_037_spine_frontmatter_required(ctx: &RunContext) -> TestResult {
    let pages = match docs_spine_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    let required = ["title", "audience", "type", "stability"];
    let mut violations = Vec::new();
    for relative in pages {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(&mut violations, "DOC-037", "docs.metadata.frontmatter_required", Some(relative), format!("read failed: {err}"));
                continue;
            }
        };
        let Some(frontmatter) = parse_docs_frontmatter(&contents) else {
            push_docs_violation(
                &mut violations,
                "DOC-037",
                "docs.metadata.frontmatter_required",
                Some(relative),
                "spine page must declare YAML frontmatter",
            );
            continue;
        };
        for key in required {
            if frontmatter.get(key).is_none() {
                push_docs_violation(
                    &mut violations,
                    "DOC-037",
                    "docs.metadata.frontmatter_required",
                    Some(relative.clone()),
                    format!("spine page frontmatter is missing `{key}`"),
                );
            }
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_docs_038_spine_frontmatter_values(ctx: &RunContext) -> TestResult {
    let pages = match docs_spine_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    let allowed_audiences = ["user", "operator", "contributor"];
    let allowed_types = ["concept", "how-to", "reference", "runbook", "adr", "internal"];
    let allowed_stability = ["draft", "stable", "deprecated"];
    let mut violations = Vec::new();
    for relative in pages {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(&mut violations, "DOC-038", "docs.metadata.frontmatter_values", Some(relative), format!("read failed: {err}"));
                continue;
            }
        };
        if let Some(value) = docs_frontmatter_string(&contents, "audience") {
            if !allowed_audiences.contains(&value.as_str()) {
                push_docs_violation(&mut violations, "DOC-038", "docs.metadata.frontmatter_values", Some(relative.clone()), format!("unsupported audience `{value}`"));
            }
        }
        if let Some(value) = docs_frontmatter_string(&contents, "type") {
            if !allowed_types.contains(&value.as_str()) {
                push_docs_violation(&mut violations, "DOC-038", "docs.metadata.frontmatter_values", Some(relative.clone()), format!("unsupported type `{value}`"));
            }
            if relative.ends_with("/index.md")
                && !relative.starts_with("docs/_internal/")
                && value != "concept"
                && value != "reference"
            {
                push_docs_violation(&mut violations, "DOC-038", "docs.metadata.frontmatter_values", Some(relative.clone()), "section index pages must use `concept` or `reference` type");
            }
        }
        if let Some(value) = docs_frontmatter_string(&contents, "stability") {
            if !allowed_stability.contains(&value.as_str()) {
                push_docs_violation(&mut violations, "DOC-038", "docs.metadata.frontmatter_values", Some(relative), format!("unsupported stability `{value}`"));
            }
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_docs_039_stable_spine_metadata(ctx: &RunContext) -> TestResult {
    let pages = match docs_spine_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    for relative in pages {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(&mut violations, "DOC-039", "docs.metadata.stable_spine_requirements", Some(relative), format!("read failed: {err}"));
                continue;
            }
        };
        if docs_frontmatter_string(&contents, "stability").as_deref() != Some("stable") {
            continue;
        }
        for key in ["owner", "last_reviewed"] {
            if docs_frontmatter_string(&contents, key).is_none() {
                push_docs_violation(&mut violations, "DOC-039", "docs.metadata.stable_spine_requirements", Some(relative.clone()), format!("stable spine page is missing `{key}`"));
            }
        }
        if docs_frontmatter_list(&contents, "tags").is_none_or(|items| items.is_empty()) {
            push_docs_violation(&mut violations, "DOC-039", "docs.metadata.stable_spine_requirements", Some(relative.clone()), "stable spine page must include non-empty `tags`");
        }
        if docs_frontmatter_list(&contents, "related").is_none_or(|items| items.len() < 2) {
            push_docs_violation(&mut violations, "DOC-039", "docs.metadata.stable_spine_requirements", Some(relative), "stable spine page must include at least two `related` links");
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_docs_040_reference_spine_sources(ctx: &RunContext) -> TestResult {
    let pages = match docs_spine_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    for relative in pages {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(&mut violations, "DOC-040", "docs.metadata.reference_sources", Some(relative), format!("read failed: {err}"));
                continue;
            }
        };
        if docs_frontmatter_string(&contents, "type").as_deref() != Some("reference") {
            continue;
        }
        if docs_frontmatter_list(&contents, "source").is_none_or(|items| items.is_empty()) {
            push_docs_violation(&mut violations, "DOC-040", "docs.metadata.reference_sources", Some(relative), "reference spine page must include a non-empty `source` list");
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_docs_041_internal_frontmatter_boundary(ctx: &RunContext) -> TestResult {
    let relative = "docs/_internal/index.md".to_string();
    let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
        Ok(contents) => contents,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-041".to_string(),
                test_id: "docs.metadata.internal_boundary".to_string(),
                file: Some(relative),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }])
        }
    };
    let mut violations = Vec::new();
    if docs_frontmatter_bool(&contents, "internal") != Some(true) {
        push_docs_violation(&mut violations, "DOC-041", "docs.metadata.internal_boundary", Some("docs/_internal/index.md".to_string()), "docs/_internal/index.md must declare `internal: true` in frontmatter");
    }
    if docs_frontmatter_string(&contents, "audience").as_deref() == Some("user") {
        push_docs_violation(&mut violations, "DOC-041", "docs.metadata.internal_boundary", Some("docs/_internal/index.md".to_string()), "internal docs may not use `audience: user`");
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_docs_042_stable_review_freshness(ctx: &RunContext) -> TestResult {
    let pages = match docs_spine_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    for relative in pages {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(&mut violations, "DOC-042", "docs.metadata.review_freshness", Some(relative), format!("read failed: {err}"));
                continue;
            }
        };
        if docs_frontmatter_string(&contents, "stability").as_deref() != Some("stable") {
            continue;
        }
        let Some(last_reviewed) = docs_frontmatter_string(&contents, "last_reviewed") else {
            continue;
        };
        if !looks_like_iso_date(&last_reviewed) {
            push_docs_violation(
                &mut violations,
                "DOC-042",
                "docs.metadata.review_freshness",
                Some(relative),
                format!("last_reviewed must be ISO date, found `{last_reviewed}`"),
            );
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_docs_043_how_to_verification_flag(ctx: &RunContext) -> TestResult {
    let pages = match docs_spine_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    for relative in pages {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(&mut violations, "DOC-043", "docs.metadata.how_to_verification", Some(relative), format!("read failed: {err}"));
                continue;
            }
        };
        if docs_frontmatter_string(&contents, "type").as_deref() != Some("how-to") {
            continue;
        }
        if docs_frontmatter_bool(&contents, "verification") != Some(true) {
            push_docs_violation(
                &mut violations,
                "DOC-043",
                "docs.metadata.how_to_verification",
                Some(relative),
                "how-to spine page must declare `verification: true`",
            );
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_docs_044_frontmatter_schema_exists(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join("configs/docs/frontmatter.schema.json");
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-044".to_string(),
                test_id: "docs.metadata.frontmatter_schema".to_string(),
                file: Some("configs/docs/frontmatter.schema.json".to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }])
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(&contents) {
        Ok(parsed) => parsed,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-044".to_string(),
                test_id: "docs.metadata.frontmatter_schema".to_string(),
                file: Some("configs/docs/frontmatter.schema.json".to_string()),
                line: None,
                message: format!("invalid json: {err}"),
                evidence: None,
            }])
        }
    };
    let required = parsed
        .get("required")
        .and_then(|value| value.as_array())
        .map(|items| {
            items.iter()
                .filter_map(|item| item.as_str())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for key in ["title", "audience", "type", "stability"] {
        if !required.contains(&key) {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-044".to_string(),
                test_id: "docs.metadata.frontmatter_schema".to_string(),
                file: Some("configs/docs/frontmatter.schema.json".to_string()),
                line: None,
                message: format!("frontmatter schema must require `{key}`"),
                evidence: None,
            }]);
        }
    }
    TestResult::Pass
}

fn test_docs_045_reader_utility_frontmatter(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for relative in docs_reader_utility_pages() {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-045",
                    "docs.metadata.reader_utility_frontmatter",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        let Some(frontmatter) = parse_docs_frontmatter(&contents) else {
            push_docs_violation(
                &mut violations,
                "DOC-045",
                "docs.metadata.reader_utility_frontmatter",
                Some(relative),
                "reader utility page must declare YAML frontmatter",
            );
            continue;
        };
        for key in ["title", "audience", "type", "stability", "owner", "last_reviewed"] {
            if frontmatter.get(key).is_none() {
                push_docs_violation(
                    &mut violations,
                    "DOC-045",
                    "docs.metadata.reader_utility_frontmatter",
                    Some(relative.clone()),
                    format!("reader utility page frontmatter is missing `{key}`"),
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
        let expected = format!("{name}/index.md");
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


include!("docs_static_metadata_quality_checks.inc.rs");
