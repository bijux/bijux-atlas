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

fn schema_required_properties(schema: &serde_json::Value) -> std::collections::BTreeSet<String> {
    schema
        .get("required")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect()
}

fn validate_docs_registry_schema(ctx: &RunContext) -> TestResult {
    const CONTRACT_ID: &str = "DOC-061";
    const TEST_ID: &str = "docs.schema.registry_validation";
    const FILE: &str = "docs/_internal/registry/registry.json";
    const SCHEMA_FILE: &str = "configs/schema/docs-registry.schema.json";

    let value = match std::fs::read_to_string(ctx.repo_root.join(FILE))
        .map_err(|err| err.to_string())
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).map_err(|err| err.to_string()))
    {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: CONTRACT_ID.to_string(),
                test_id: TEST_ID.to_string(),
                file: Some(FILE.to_string()),
                line: None,
                message: format!("parse failed: {err}"),
                evidence: None,
            }])
        }
    };
    let schema = match std::fs::read_to_string(ctx.repo_root.join(SCHEMA_FILE))
        .map_err(|err| err.to_string())
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).map_err(|err| err.to_string()))
    {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: CONTRACT_ID.to_string(),
                test_id: TEST_ID.to_string(),
                file: Some(SCHEMA_FILE.to_string()),
                line: None,
                message: format!("schema parse failed: {err}"),
                evidence: None,
            }])
        }
    };

    let Some(object) = value.as_object() else {
        return TestResult::Fail(vec![Violation {
            contract_id: CONTRACT_ID.to_string(),
            test_id: TEST_ID.to_string(),
            file: Some(FILE.to_string()),
            line: None,
            message: "docs registry must be a JSON object".to_string(),
            evidence: None,
        }]);
    };
    let allowed_properties = schema
        .get("properties")
        .and_then(|value| value.as_object())
        .map(|props| props.keys().cloned().collect::<std::collections::BTreeSet<_>>())
        .unwrap_or_default();
    let mut violations = Vec::new();
    for required in schema_required_properties(&schema) {
        if !object.contains_key(&required) {
            violations.push(Violation {
                contract_id: CONTRACT_ID.to_string(),
                test_id: TEST_ID.to_string(),
                file: Some(FILE.to_string()),
                line: None,
                message: format!("missing required property `{required}`"),
                evidence: None,
            });
        }
    }
    for key in object.keys() {
        if !allowed_properties.contains(key) {
            violations.push(Violation {
                contract_id: CONTRACT_ID.to_string(),
                test_id: TEST_ID.to_string(),
                file: Some(FILE.to_string()),
                line: None,
                message: format!("unexpected property `{key}`"),
                evidence: None,
            });
        }
    }
    if object
        .get("schema_version")
        .and_then(|value| value.as_u64())
        .filter(|version| *version >= 1)
        .is_none()
    {
        violations.push(Violation {
            contract_id: CONTRACT_ID.to_string(),
            test_id: TEST_ID.to_string(),
            file: Some(FILE.to_string()),
            line: None,
            message: "schema_version must be an integer >= 1".to_string(),
            evidence: None,
        });
    }

    let Some(documents) = object.get("documents").and_then(|value| value.as_array()) else {
        violations.push(Violation {
            contract_id: CONTRACT_ID.to_string(),
            test_id: TEST_ID.to_string(),
            file: Some(FILE.to_string()),
            line: None,
            message: "property `documents` must be an array".to_string(),
            evidence: None,
        });
        return TestResult::Fail(violations);
    };
    if documents.is_empty() {
        violations.push(Violation {
            contract_id: CONTRACT_ID.to_string(),
            test_id: TEST_ID.to_string(),
            file: Some(FILE.to_string()),
            line: None,
            message: "property `documents` must not be empty".to_string(),
            evidence: None,
        });
    }
    for (index, document) in documents.iter().enumerate() {
        let Some(row) = document.as_object() else {
            violations.push(Violation {
                contract_id: CONTRACT_ID.to_string(),
                test_id: TEST_ID.to_string(),
                file: Some(FILE.to_string()),
                line: None,
                message: format!("documents[{index}] must be an object"),
                evidence: None,
            });
            continue;
        };
        for field in [
            "area",
            "audience",
            "doc_type",
            "last_reviewed",
            "owner",
            "path",
            "stability",
            "title",
            "topic",
        ] {
            if row
                .get(field)
                .and_then(|value| value.as_str())
                .map(|value| !value.trim().is_empty())
                != Some(true)
            {
                violations.push(Violation {
                    contract_id: CONTRACT_ID.to_string(),
                    test_id: TEST_ID.to_string(),
                    file: Some(FILE.to_string()),
                    line: None,
                    message: format!("documents[{index}].{field} must be a non-empty string"),
                    evidence: None,
                });
            }
        }
        if let Some(last_reviewed) = row.get("last_reviewed").and_then(|value| value.as_str()) {
            if !looks_like_iso_date(last_reviewed) {
                violations.push(Violation {
                    contract_id: CONTRACT_ID.to_string(),
                    test_id: TEST_ID.to_string(),
                    file: Some(FILE.to_string()),
                    line: None,
                    message: format!("documents[{index}].last_reviewed must be an ISO date"),
                    evidence: None,
                });
            }
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn validate_docs_sections_schema(ctx: &RunContext) -> TestResult {
    const CONTRACT_ID: &str = "DOC-062";
    const TEST_ID: &str = "docs.schema.sections_validation";
    const FILE: &str = "docs/_internal/registry/sections.json";
    const SCHEMA_FILE: &str = "configs/schema/docs-sections.schema.json";

    let value = match std::fs::read_to_string(ctx.repo_root.join(FILE))
        .map_err(|err| err.to_string())
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).map_err(|err| err.to_string()))
    {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: CONTRACT_ID.to_string(),
                test_id: TEST_ID.to_string(),
                file: Some(FILE.to_string()),
                line: None,
                message: format!("parse failed: {err}"),
                evidence: None,
            }])
        }
    };
    let schema = match std::fs::read_to_string(ctx.repo_root.join(SCHEMA_FILE))
        .map_err(|err| err.to_string())
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).map_err(|err| err.to_string()))
    {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: CONTRACT_ID.to_string(),
                test_id: TEST_ID.to_string(),
                file: Some(SCHEMA_FILE.to_string()),
                line: None,
                message: format!("schema parse failed: {err}"),
                evidence: None,
            }])
        }
    };

    let Some(object) = value.as_object() else {
        return TestResult::Fail(vec![Violation {
            contract_id: CONTRACT_ID.to_string(),
            test_id: TEST_ID.to_string(),
            file: Some(FILE.to_string()),
            line: None,
            message: "docs sections file must be a JSON object".to_string(),
            evidence: None,
        }]);
    };
    let allowed_properties = schema
        .get("properties")
        .and_then(|value| value.as_object())
        .map(|props| props.keys().cloned().collect::<std::collections::BTreeSet<_>>())
        .unwrap_or_default();
    let mut violations = Vec::new();
    for required in schema_required_properties(&schema) {
        if !object.contains_key(&required) {
            violations.push(Violation {
                contract_id: CONTRACT_ID.to_string(),
                test_id: TEST_ID.to_string(),
                file: Some(FILE.to_string()),
                line: None,
                message: format!("missing required property `{required}`"),
                evidence: None,
            });
        }
    }
    for key in object.keys() {
        if !allowed_properties.contains(key) {
            violations.push(Violation {
                contract_id: CONTRACT_ID.to_string(),
                test_id: TEST_ID.to_string(),
                file: Some(FILE.to_string()),
                line: None,
                message: format!("unexpected property `{key}`"),
                evidence: None,
            });
        }
    }
    if object
        .get("schema_version")
        .and_then(|value| value.as_u64())
        .filter(|version| *version >= 1)
        .is_none()
    {
        violations.push(Violation {
            contract_id: CONTRACT_ID.to_string(),
            test_id: TEST_ID.to_string(),
            file: Some(FILE.to_string()),
            line: None,
            message: "schema_version must be an integer >= 1".to_string(),
            evidence: None,
        });
    }
    let Some(sections) = object.get("sections").and_then(|value| value.as_object()) else {
        violations.push(Violation {
            contract_id: CONTRACT_ID.to_string(),
            test_id: TEST_ID.to_string(),
            file: Some(FILE.to_string()),
            line: None,
            message: "property `sections` must be an object".to_string(),
            evidence: None,
        });
        return TestResult::Fail(violations);
    };
    if sections.is_empty() {
        violations.push(Violation {
            contract_id: CONTRACT_ID.to_string(),
            test_id: TEST_ID.to_string(),
            file: Some(FILE.to_string()),
            line: None,
            message: "property `sections` must not be empty".to_string(),
            evidence: None,
        });
    }
    for (name, section) in sections {
        let Some(row) = section.as_object() else {
            violations.push(Violation {
                contract_id: CONTRACT_ID.to_string(),
                test_id: TEST_ID.to_string(),
                file: Some(FILE.to_string()),
                line: None,
                message: format!("section `{name}` must be an object"),
                evidence: None,
            });
            continue;
        };
        for field in ["requires_index", "root_entrypoint"] {
            if row.get(field).and_then(|value| value.as_bool()).is_none() {
                violations.push(Violation {
                    contract_id: CONTRACT_ID.to_string(),
                    test_id: TEST_ID.to_string(),
                    file: Some(FILE.to_string()),
                    line: None,
                    message: format!("section `{name}` field `{field}` must be a boolean"),
                    evidence: None,
                });
            }
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
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
    for seed in ["docs/index.md", "docs/index.md", "docs/start-here.md"] {
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
    let mut fingerprints = Vec::<(String, std::collections::BTreeSet<String>)>::new();
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
        by_title
            .entry(normalized)
            .or_default()
            .push(relative.clone());
        let token_set = contents
            .split(|ch: char| !ch.is_ascii_alphanumeric())
            .filter(|token| token.len() >= 5)
            .map(|token| token.to_ascii_lowercase())
            .collect::<std::collections::BTreeSet<_>>();
        fingerprints.push((relative, token_set));
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
    fingerprints.sort_by(|a, b| a.0.cmp(&b.0));
    let mut analyzed_pairs = Vec::new();
    for left in 0..fingerprints.len() {
        for right in (left + 1)..fingerprints.len() {
            let (left_path, left_tokens) = &fingerprints[left];
            let (right_path, right_tokens) = &fingerprints[right];
            if left_tokens.is_empty() || right_tokens.is_empty() {
                continue;
            }
            let overlap = left_tokens.intersection(right_tokens).count();
            if overlap == 0 {
                continue;
            }
            let union = left_tokens.union(right_tokens).count();
            if union == 0 {
                continue;
            }
            let similarity = overlap as f64 / union as f64;
            if similarity < 0.35 {
                continue;
            }
            analyzed_pairs.push(serde_json::json!({
                "left": left_path,
                "right": right_path,
                "shared_token_count": overlap,
                "similarity": ((similarity * 1000.0).round() / 1000.0),
            }));
        }
    }
    analyzed_pairs.sort_by(|a, b| {
        b["similarity"]
            .as_f64()
            .partial_cmp(&a["similarity"].as_f64())
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a["left"].as_str().cmp(&b["left"].as_str()))
            .then(a["right"].as_str().cmp(&b["right"].as_str()))
    });
    serde_json::json!({
        "schema_version": 1,
        "kind": "docs_duplication",
        "status": if duplicates.is_empty() { "pass" } else { "warn" },
        "duplicate_titles": duplicates,
        "analyzed_pairs": analyzed_pairs.into_iter().take(50).collect::<Vec<_>>()
    })
}


include!("docs_static_artifact_reports_checks.inc.rs");
