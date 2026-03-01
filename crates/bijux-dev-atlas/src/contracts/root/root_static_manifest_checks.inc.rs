fn test_root_016_surface_manifest_complete(ctx: &RunContext) -> TestResult {
    let payload = match root_surface_manifest(ctx, "ROOT-016", "root.surface.manifest_complete") {
        Ok(payload) => payload,
        Err(result) => return result,
    };
    let entries = match payload["entries"].as_object() {
        Some(entries) => entries,
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-016".to_string(),
                test_id: "root.surface.manifest_complete".to_string(),
                file: Some("ops/inventory/root-surface.json".to_string()),
                line: None,
                message: "`entries` object is required".to_string(),
                evidence: None,
            }])
        }
    };
    let expected = ROOT_ALLOWED_VISIBLE
        .iter()
        .chain(ROOT_ALLOWED_VISIBLE_TAIL.iter())
        .map(|value| (*value).to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let manifest_entries = entries.keys().cloned().collect::<std::collections::BTreeSet<_>>();
    let mut violations = Vec::new();
    for name in expected.difference(&manifest_entries) {
        push_root_violation(
            &mut violations,
            "ROOT-016",
            "root.surface.manifest_complete",
            Some(name.clone()),
            "sealed root entry is missing from ops/inventory/root-surface.json",
        );
    }
    for name in manifest_entries.difference(&expected) {
        push_root_violation(
            &mut violations,
            "ROOT-016",
            "root.surface.manifest_complete",
            Some(name.clone()),
            "ops/inventory/root-surface.json references an undeclared root entry",
        );
    }
    for name in manifest_entries {
        let path = ctx.repo_root.join(&name);
        if !path.exists() {
            push_root_violation(
                &mut violations,
                "ROOT-016",
                "root.surface.manifest_complete",
                Some(name),
                "ops/inventory/root-surface.json references a missing repo root entry",
            );
            continue;
        }
        let declared_kind = entries
            .get(&name)
            .and_then(|value| value.get("kind"))
            .and_then(|value| value.as_str());
        let actual_kind = if path.is_symlink() {
            Some("symlink")
        } else if path.is_dir() {
            Some("dir")
        } else if path.is_file() {
            Some("file")
        } else {
            None
        };
        if declared_kind != actual_kind {
            push_root_violation(
                &mut violations,
                "ROOT-016",
                "root.surface.manifest_complete",
                Some(name),
                format!(
                    "ops/inventory/root-surface.json kind drift: declared {:?}, actual {:?}",
                    declared_kind, actual_kind
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

fn test_root_017_no_binary_artifacts(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let entries = match std::fs::read_dir(&ctx.repo_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-017".to_string(),
                test_id: "root.surface.no_binary_artifacts".to_string(),
                file: None,
                line: None,
                message: format!("read repo root failed: {err}"),
                evidence: None,
            }])
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let extension = match path.extension().and_then(|value| value.to_str()) {
            Some(extension) => extension,
            None => continue,
        };
        if ROOT_FORBIDDEN_BINARY_EXTENSIONS
            .iter()
            .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        {
            push_root_violation(
                &mut violations,
                "ROOT-017",
                "root.surface.no_binary_artifacts",
                Some(name),
                format!("root binary artifact extension is forbidden: .{extension}"),
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

fn test_root_018_no_env_files(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let entries = match std::fs::read_dir(&ctx.repo_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-018".to_string(),
                test_id: "root.surface.no_env_files".to_string(),
                file: None,
                line: None,
                message: format!("read repo root failed: {err}"),
                evidence: None,
            }])
        }
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == ".env" || (name.starts_with(".env.") && name.len() > 5) {
            push_root_violation(
                &mut violations,
                "ROOT-018",
                "root.surface.no_env_files",
                Some(name),
                "committed root .env files are forbidden",
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

fn test_root_019_directory_budget(ctx: &RunContext) -> TestResult {
    let expected = ROOT_ALLOWED_VISIBLE
        .iter()
        .chain(ROOT_ALLOWED_VISIBLE_TAIL.iter())
        .filter(|name| ctx.repo_root.join(name).is_dir())
        .map(|name| (*name).to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let mut visible_directories = Vec::new();
    let entries = match std::fs::read_dir(&ctx.repo_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-019".to_string(),
                test_id: "root.surface.directory_budget".to_string(),
                file: None,
                line: None,
                message: format!("read repo root failed: {err}"),
                evidence: None,
            }])
        }
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if ROOT_IGNORED_LOCAL.iter().any(|ignored| *ignored == name) {
            continue;
        }
        visible_directories.push(name);
    }
    visible_directories.sort();
    let mut violations = Vec::new();
    if visible_directories.len() > ROOT_DIRECTORY_BUDGET {
        push_root_violation(
            &mut violations,
            "ROOT-019",
            "root.surface.directory_budget",
            None,
            format!(
                "repo root directory budget exceeded: {} > {} ({})",
                visible_directories.len(),
                ROOT_DIRECTORY_BUDGET,
                visible_directories.join(", ")
            ),
        );
    }
    let actual = visible_directories.iter().cloned().collect::<std::collections::BTreeSet<_>>();
    for name in actual.difference(&expected) {
        push_root_violation(
            &mut violations,
            "ROOT-019",
            "root.surface.directory_budget",
            Some(name.clone()),
            "unexpected top-level directory is outside the approved root surface",
        );
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_root_020_single_segment_entries(ctx: &RunContext) -> TestResult {
    let raw = match read_root_text(
        ctx,
        "ops/inventory/root-surface.json",
        "ROOT-020",
        "root.surface.single_segment_entries",
    ) {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    let payload = match root_surface_manifest(ctx, "ROOT-020", "root.surface.single_segment_entries") {
        Ok(payload) => payload,
        Err(result) => return result,
    };
    let entries = match payload["entries"].as_object() {
        Some(entries) => entries,
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-020".to_string(),
                test_id: "root.surface.single_segment_entries".to_string(),
                file: Some("ops/inventory/root-surface.json".to_string()),
                line: None,
                message: "`entries` object is required".to_string(),
                evidence: None,
            }])
        }
    };
    let mut violations = Vec::new();
    for name in entries.keys() {
        if name.contains('/') || name.contains('\\') {
            push_root_violation(
                &mut violations,
                "ROOT-020",
                "root.surface.single_segment_entries",
                Some(name.clone()),
                "root manifest entries must be single-segment repo root names",
            );
        }
    }
    let mut in_entries = false;
    let mut manifest_entry_rows = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("\"entries\"") {
            in_entries = true;
            continue;
        }
        if in_entries && trimmed == "}," {
            in_entries = false;
            continue;
        }
        if !in_entries || !trimmed.starts_with('"') {
            continue;
        }
        if let Some(key) = trimmed.split('"').nth(1).map(str::to_string) {
            if entries.contains_key(&key) {
                manifest_entry_rows.push(key);
            }
        }
    }
    let mut sorted_entry_rows = manifest_entry_rows.clone();
    sorted_entry_rows.sort();
    if manifest_entry_rows != sorted_entry_rows {
        push_root_violation(
            &mut violations,
            "ROOT-020",
            "root.surface.single_segment_entries",
            Some("ops/inventory/root-surface.json".to_string()),
            "root manifest entries must be sorted lexicographically",
        );
    }
    if let Some(values) = payload["ssot_roots"].as_array() {
        let roots = values
            .iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>();
        let mut sorted_roots = roots.clone();
        sorted_roots.sort();
        if roots != sorted_roots {
            push_root_violation(
                &mut violations,
                "ROOT-020",
                "root.surface.single_segment_entries",
                Some("ops/inventory/root-surface.json".to_string()),
                "`ssot_roots` must be sorted lexicographically",
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

fn test_root_027_manifest_ssot_roots(ctx: &RunContext) -> TestResult {
    let payload = match root_surface_manifest(ctx, "ROOT-027", "root.surface.ssot_roots") {
        Ok(payload) => payload,
        Err(result) => return result,
    };
    let ssot_roots = match payload["ssot_roots"].as_array() {
        Some(values) => values
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<std::collections::BTreeSet<_>>(),
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-027".to_string(),
                test_id: "root.surface.ssot_roots".to_string(),
                file: Some("ops/inventory/root-surface.json".to_string()),
                line: None,
                message: "`ssot_roots` array is required".to_string(),
                evidence: None,
            }])
        }
    };
    let mut violations = Vec::new();
    for required in ["configs", "ops"] {
        if !ssot_roots.contains(required) {
            push_root_violation(
                &mut violations,
                "ROOT-027",
                "root.surface.ssot_roots",
                Some("ops/inventory/root-surface.json".to_string()),
                format!("missing ssot root declaration: {required}"),
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_028_manifest_docs_governed(ctx: &RunContext) -> TestResult {
    let payload = match root_surface_manifest(ctx, "ROOT-028", "root.surface.docs_governed") {
        Ok(payload) => payload,
        Err(result) => return result,
    };
    let ssot_roots = match payload["ssot_roots"].as_array() {
        Some(values) => values
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<std::collections::BTreeSet<_>>(),
        None => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-028".to_string(),
                test_id: "root.surface.docs_governed".to_string(),
                file: Some("ops/inventory/root-surface.json".to_string()),
                line: None,
                message: "`ssot_roots` array is required".to_string(),
                evidence: None,
            }])
        }
    };
    let mut violations = Vec::new();
    if !ctx.repo_root.join("docs").is_dir() {
        push_root_violation(
            &mut violations,
            "ROOT-028",
            "root.surface.docs_governed",
            Some("docs".to_string()),
            "docs/ must exist at the repo root",
        );
    }
    if !ssot_roots.contains("docs") {
        push_root_violation(
            &mut violations,
            "ROOT-028",
            "root.surface.docs_governed",
            Some("ops/inventory/root-surface.json".to_string()),
            "docs must be declared as a governed root in ssot_roots",
        );
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}
