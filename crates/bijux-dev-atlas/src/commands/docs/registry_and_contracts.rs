pub(crate) fn crate_doc_contract_status(
    repo_root: &Path,
) -> (Vec<serde_json::Value>, Vec<String>, Vec<String>) {
    let required_common = [
        "README.md",
        "ARCHITECTURE.md",
        "CONTRACT.md",
        "TESTING.md",
        "ERROR_TAXONOMY.md",
        "EXAMPLES.md",
        "BENCHMARKS.md",
        "VERSIONING.md",
    ];
    let mut rows = Vec::<serde_json::Value>::new();
    let mut errors = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();

    for crate_root in workspace_crate_roots(repo_root) {
        let crate_name = crate_root
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("unknown");
        let root_md = read_dir_entries(&crate_root)
            .into_iter()
            .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("md"))
            .collect::<Vec<_>>();
        let docs_dir = crate_root.join("docs");
        let docs_md = if docs_dir.exists() {
            walk_files_local(&docs_dir)
                .into_iter()
                .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("md"))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        if docs_md.len() > 10 {
            warnings.push(format!(
                "CRATE_DOCS_DIR_BUDGET_WARN: `{crate_name}` has {} docs under crate/docs (budget=10)",
                docs_md.len()
            ));
        }

        let root_names = root_md
            .iter()
            .filter_map(|p| p.file_name().and_then(|v| v.to_str()))
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        let mut allowed_root = BTreeSet::from([
            "README.md".to_string(),
            "ARCHITECTURE.md".to_string(),
            "CONTRACT.md".to_string(),
            "TESTING.md".to_string(),
            "ERROR_TAXONOMY.md".to_string(),
            "EXAMPLES.md".to_string(),
            "BENCHMARKS.md".to_string(),
            "VERSIONING.md".to_string(),
        ]);
        if crate_name.contains("-model") {
            allowed_root.insert("DATA_MODEL.md".to_string());
        }
        if crate_name.contains("-policies") {
            allowed_root.insert("EXTENSION_GUIDE.md".to_string());
        }
        if crate_name.ends_with("-adapters") {
            allowed_root.insert("ADAPTER_BOUNDARY.md".to_string());
        }
        if crate_name.ends_with("-cli") || crate_name.ends_with("dev-atlas") {
            allowed_root.insert("COMMAND_SURFACE.md".to_string());
        }
        if root_names.len() > 10 {
            errors.push(format!(
                "CRATE_DOC_BUDGET_ERROR: `{crate_name}` has {} root docs (budget=10)",
                root_names.len()
            ));
        }
        for root_name in &root_names {
            if !allowed_root.contains(root_name) {
                warnings.push(format!(
                    "CRATE_DOC_ALLOWED_TYPE_WARN: `{crate_name}` has non-canonical root doc `{root_name}`"
                ));
            }
        }

        for required in &required_common {
            if !root_names.contains(*required) {
                errors.push(format!(
                    "CRATE_DOC_REQUIRED_ERROR: `{crate_name}` missing `{required}`"
                ));
            }
        }

        if crate_name.contains("-model") && !root_names.contains("DATA_MODEL.md") {
            errors.push(format!(
                "CRATE_DOC_REQUIRED_ERROR: `{crate_name}` missing `DATA_MODEL.md`"
            ));
        }
        if crate_name.contains("-policies") && !root_names.contains("EXTENSION_GUIDE.md") {
            errors.push(format!(
                "CRATE_DOC_REQUIRED_ERROR: `{crate_name}` missing `EXTENSION_GUIDE.md`"
            ));
        }
        if (crate_name.ends_with("-cli") || crate_name.ends_with("dev-atlas"))
            && !root_names.contains("COMMAND_SURFACE.md")
        {
            errors.push(format!(
                "CRATE_DOC_REQUIRED_ERROR: `{crate_name}` missing `COMMAND_SURFACE.md`"
            ));
        }
        if crate_name.ends_with("-adapters") && !root_names.contains("ADAPTER_BOUNDARY.md") {
            errors.push(format!(
                "CRATE_DOC_REQUIRED_ERROR: `{crate_name}` missing `ADAPTER_BOUNDARY.md`"
            ));
        }

        if crate_name.contains("-core") {
            let architecture = crate_root.join("ARCHITECTURE.md");
            let has_invariants = fs::read_to_string(&architecture)
                .ok()
                .is_some_and(|text| text.contains("## Invariants"));
            if !has_invariants {
                warnings.push(format!(
                    "CRATE_DOC_INVARIANTS_WARN: `{crate_name}` ARCHITECTURE.md should include `## Invariants`"
                ));
            }
        }

        let readme_path = crate_root.join("README.md");
        if let Ok(readme) = fs::read_to_string(&readme_path) {
            if !readme.contains("docs/") {
                warnings.push(format!(
                    "CRATE_DOC_LINK_WARN: `{crate_name}` README.md should link to crate docs/"
                ));
            }
        }
        let index_path = crate_root.join("docs/INDEX.md");
        if let Ok(index) = fs::read_to_string(&index_path) {
            for expected in ["architecture.md", "public-api.md", "testing.md"] {
                if !index.contains(expected) {
                    warnings.push(format!(
                        "CRATE_DOC_CROSSLINK_WARN: `{crate_name}` docs/INDEX.md should reference `{expected}`"
                    ));
                }
            }
        }

        let mut diagram_count = 0usize;
        let mut rust_fences = 0usize;
        let mut rust_fences_tagged = 0usize;
        let mut docs_with_owner = 0usize;
        let mut docs_with_last_reviewed = 0usize;
        for doc in root_md.iter().chain(docs_md.iter()) {
            let text = fs::read_to_string(doc).unwrap_or_default();
            diagram_count += text.matches("![").count();
            docs_with_owner += usize::from(text.contains("- Owner:"));
            docs_with_last_reviewed += usize::from(text.contains("Last Reviewed:"));
            for line in text.lines() {
                if line.trim_start().starts_with("```") {
                    rust_fences += usize::from(line.trim() == "```");
                    rust_fences_tagged += usize::from(line.trim().starts_with("```rust"));
                }
            }
        }
        if diagram_count > 20 {
            warnings.push(format!(
                "CRATE_DOC_DIAGRAM_BUDGET_WARN: `{crate_name}` has {diagram_count} diagrams (budget=20)"
            ));
        }
        if rust_fences > rust_fences_tagged {
            warnings.push(format!(
                "CRATE_DOC_EXAMPLE_TAG_WARN: `{crate_name}` has untagged code fences; prefer ```rust for examples"
            ));
        }
        let total_docs = root_md.len() + docs_md.len();
        if docs_with_owner < total_docs {
            warnings.push(format!(
                "CRATE_DOC_OWNER_METADATA_WARN: `{crate_name}` owner metadata present in {docs_with_owner}/{total_docs} docs"
            ));
        }
        if docs_with_owner == 0 && total_docs > 0 {
            errors.push(format!(
                "CRATE_DOC_OWNER_ESCALATION_ERROR: `{crate_name}` has no owner metadata in crate docs"
            ));
        }
        if docs_with_last_reviewed == 0 {
            warnings.push(format!(
                "CRATE_DOC_FRESHNESS_WARN: `{crate_name}` has no `Last Reviewed:` metadata in crate docs"
            ));
        }

        rows.push(serde_json::json!({
            "crate": crate_name,
            "root_doc_count": root_names.len(),
            "docs_dir_count": docs_md.len(),
            "required": required_common,
            "has": root_names,
            "diagram_count": diagram_count,
            "owner_metadata_docs": docs_with_owner,
            "freshness_docs": docs_with_last_reviewed
        }));
    }
    let mut concept_index = BTreeMap::<String, Vec<String>>::new();
    for crate_root in workspace_crate_roots(repo_root) {
        let crate_name = crate_root
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("unknown")
            .to_string();
        let docs_dir = crate_root.join("docs");
        if !docs_dir.exists() {
            continue;
        }
        for file in walk_files_local(&docs_dir) {
            if file.extension().and_then(|v| v.to_str()) != Some("md") {
                continue;
            }
            let Some(name) = file.file_name().and_then(|v| v.to_str()) else {
                continue;
            };
            let concept = name.to_ascii_lowercase();
            if matches!(
                concept.as_str(),
                "index.md"
                    | "architecture.md"
                    | "public-api.md"
                    | "testing.md"
                    | "effects.md"
                    | "effect-boundary-map.md"
            ) {
                continue;
            }
            concept_index
                .entry(concept)
                .or_default()
                .push(crate_name.clone());
        }
    }
    for (concept, crates) in concept_index {
        let distinct = crates.into_iter().collect::<BTreeSet<_>>();
        if distinct.len() > 1 {
            warnings.push(format!(
                "CRATE_DOC_DUPLICATE_CONCEPT_WARN: `{concept}` appears across crates: {}",
                distinct.into_iter().collect::<Vec<_>>().join(", ")
            ));
        }
    }
    rows.sort_by(|a, b| a["crate"].as_str().cmp(&b["crate"].as_str()));
    errors.sort();
    errors.dedup();
    warnings.sort();
    warnings.dedup();
    (rows, errors, warnings)
}

fn is_vendored_docs_registry_path(path: &str) -> bool {
    path.starts_with("configs/docs/node_modules/")
}

fn tags_for_path(path: &str) -> Vec<String> {
    let mut out = BTreeSet::new();
    for segment in path.split('/') {
        if segment.is_empty() || segment == "docs" || segment == "crates" {
            continue;
        }
        let tag = segment
            .trim_end_matches(".md")
            .replace('_', "-")
            .to_ascii_lowercase();
        if tag.len() >= 3 {
            out.insert(tag);
        }
    }
    out.into_iter().take(8).collect()
}

pub(crate) fn search_synonyms(repo_root: &Path) -> Vec<serde_json::Value> {
    let path = repo_root.join("docs/metadata/search-synonyms.json");
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| v.get("synonyms").and_then(|s| s.as_array().cloned()))
        .unwrap_or_default()
}

fn canonical_reference_checks(
    repo_root: &Path,
    docs: &[serde_json::Value],
) -> (Vec<String>, Vec<String>, serde_json::Value) {
    let contract_path = repo_root.join("docs/metadata/reference-canonicals.json");
    if !contract_path.exists() {
        return (
            Vec::new(),
            vec![
                "DOCS_CANONICAL_CONTRACT_WARN: docs/metadata/reference-canonicals.json is missing"
                    .to_string(),
            ],
            serde_json::json!({"categories": {}, "total_entries": 0}),
        );
    }
    let text = match fs::read_to_string(&contract_path) {
        Ok(v) => v,
        Err(e) => {
            return (
                vec![format!(
                    "DOCS_CANONICAL_CONTRACT_ERROR: failed reading `{}`: {e}",
                    contract_path.display()
                )],
                Vec::new(),
                serde_json::json!({"categories": {}, "total_entries": 0}),
            );
        }
    };
    let contract: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            return (
                vec![format!(
                    "DOCS_CANONICAL_CONTRACT_ERROR: invalid json in `{}`: {e}",
                    contract_path.display()
                )],
                Vec::new(),
                serde_json::json!({"categories": {}, "total_entries": 0}),
            );
        }
    };

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut category_counts = BTreeMap::<String, usize>::new();
    let mut total_entries = 0usize;
    let mut seen_ids = BTreeSet::<String>::new();
    for category in ["major_subsystems", "commands", "schemas", "policies"] {
        let entries = contract[category].as_array().cloned().unwrap_or_default();
        category_counts.insert(category.to_string(), entries.len());
        for entry in entries {
            total_entries += 1;
            let id = entry["id"].as_str().unwrap_or_default();
            let path = entry["path"].as_str().unwrap_or_default();
            if id.is_empty() || path.is_empty() {
                errors.push(format!(
                    "DOCS_CANONICAL_CONTRACT_ERROR: `{category}` entry must include `id` and `path`"
                ));
                continue;
            }
            if !seen_ids.insert(format!("{category}:{id}")) {
                errors.push(format!(
                    "DOCS_CANONICAL_DUPLICATE_ID: `{category}` id `{id}` is duplicated"
                ));
            }
            if !repo_root.join(path).exists() {
                errors.push(format!(
                    "DOCS_CANONICAL_REFERENCE_MISSING_FILE: `{category}` id `{id}` missing `{path}`"
                ));
            }
            let matches = docs
                .iter()
                .filter(|doc| doc["path"].as_str() == Some(path))
                .count();
            if matches != 1 {
                errors.push(format!(
                    "DOCS_CANONICAL_REFERENCE_COUNT_ERROR: `{category}` id `{id}` expected exactly one registry entry for `{path}`, found {matches}"
                ));
            }
        }
        if category_counts.get(category).copied().unwrap_or(0) == 0 {
            warnings.push(format!(
                "DOCS_CANONICAL_CATEGORY_WARN: `{category}` has no canonical references"
            ));
        }
    }
    (
        errors,
        warnings,
        serde_json::json!({"categories": category_counts, "total_entries": total_entries}),
    )
}

pub(crate) fn docs_registry_payload(ctx: &DocsContext) -> serde_json::Value {
    let mut docs = Vec::new();
    for file in scan_registry_markdown_files(&ctx.repo_root) {
        let Ok(rel) = file.strip_prefix(&ctx.repo_root) else {
            continue;
        };
        let rel_path = rel.display().to_string();
        let (owner, stability) = parse_owner_and_stability(&file);
        let crate_name = crate_association(&rel_path);
        docs.push(serde_json::json!({
            "path": rel_path,
            "doc_type": infer_doc_type(&rel.display().to_string()),
            "owner": owner,
            "crate": crate_name,
            "stability": stability,
            "last_reviewed": "2026-02-25",
            "review_due": "2026-08-24",
            "lifecycle": infer_lifecycle(&rel.display().to_string()),
            "tags": tags_for_path(&rel.display().to_string()),
            "keywords": tags_for_path(&rel.display().to_string()),
            "doc_version": "v1",
            "topic": rel.file_stem().and_then(|v| v.to_str()).unwrap_or("unknown")
        }));
    }
    docs.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    serde_json::json!({
        "schema_version": 1,
        "project_version": "v0.1.0",
        "generated_by": "bijux dev atlas docs registry build",
        "generated_from": "docs and crate docs",
        "documents": docs
    })
}

fn parse_ymd_date(s: &str) -> Option<(i32, i32, i32)> {
    let parts: Vec<_> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let y = parts[0].parse().ok()?;
    let m = parts[1].parse().ok()?;
    let d = parts[2].parse().ok()?;
    Some((y, m, d))
}

fn days_from_civil(y: i32, m: i32, d: i32) -> i64 {
    let y = y - i32::from(m <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = m + if m > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era * 146_097 + doe - 719_468) as i64
}

fn date_diff_days(older: &str, newer: &str) -> Option<i64> {
    let (y1, m1, d1) = parse_ymd_date(older)?;
    let (y2, m2, d2) = parse_ymd_date(newer)?;
    Some(days_from_civil(y2, m2, d2) - days_from_civil(y1, m1, d1))
}

pub(crate) fn has_required_section(text: &str, section: &str) -> bool {
    let needle = format!("## {section}");
    text.lines().any(|line| line.trim() == needle)
}

pub(crate) fn registry_validate_payload(ctx: &DocsContext) -> Result<serde_json::Value, String> {
    let registry_path = ctx.repo_root.join("docs/registry.json");
    if !registry_path.exists() {
        return Ok(serde_json::json!({
            "schema_version": 1,
            "errors": [],
            "warnings": ["DOCS_REGISTRY_MISSING: docs/registry.json is missing"],
            "summary": {"errors": 0, "warnings": 1}
        }));
    }
    let text = fs::read_to_string(&registry_path)
        .map_err(|e| format!("failed to read {}: {e}", registry_path.display()))?;
    let registry: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("invalid docs registry json: {e}"))?;
    let docs = registry["documents"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let policy = load_quality_policy(&ctx.repo_root);
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut seen_paths = BTreeSet::new();
    let mut seen_topics = BTreeMap::<String, usize>::new();
    let scanned = scan_registry_markdown_files(&ctx.repo_root)
        .into_iter()
        .filter_map(|p| {
            p.strip_prefix(&ctx.repo_root)
                .ok()
                .map(|r| r.display().to_string())
        })
        .collect::<BTreeSet<_>>();
    for entry in &docs {
        let Some(path) = entry["path"].as_str() else {
            errors.push("DOCS_REGISTRY_INVALID_ENTRY: missing path".to_string());
            continue;
        };
        if !seen_paths.insert(path.to_string()) {
            errors.push(format!("DOCS_REGISTRY_DUPLICATE_PATH: `{path}`"));
        }
        if !ctx.repo_root.join(path).exists() {
            errors.push(format!("DOCS_REGISTRY_MISSING_FILE: `{path}`"));
        }
        if let Some(topic) = entry["topic"].as_str() {
            *seen_topics.entry(topic.to_string()).or_default() += 1;
        }
        if let Some(last_reviewed) = entry["last_reviewed"].as_str() {
            if let Some(age_days) = date_diff_days(last_reviewed, "2026-02-25") {
                if age_days > policy.stale_days {
                    warnings.push(format!(
                        "DOCS_REGISTRY_OUTDATED: `{path}` last_reviewed={last_reviewed} age_days={age_days}"
                    ));
                }
            }
        } else {
            warnings.push(format!("DOCS_REGISTRY_MISSING_LAST_REVIEWED: `{path}`"));
        }
        if entry["owner"].as_str().unwrap_or("unknown") == "unknown" {
            errors.push(format!(
                "DOCS_REGISTRY_OWNER_REQUIRED: `{path}` requires owner metadata"
            ));
        }
    }
    for path in scanned.difference(&seen_paths) {
        errors.push(format!("DOCS_REGISTRY_ORPHAN_DOC: `{path}` not registered"));
    }
    for path in seen_paths.difference(&scanned) {
        errors.push(format!("DOCS_REGISTRY_ORPHAN_ENTRY: `{path}` has no file"));
    }
    for (topic, count) in seen_topics {
        if count > 1 {
            warnings.push(format!(
                "DOCS_REGISTRY_DUPLICATE_TOPIC: `{topic}` appears {count} times"
            ));
        }
    }
    let mut per_crate = BTreeMap::<String, usize>::new();
    for entry in &docs {
        let bucket = entry["crate"].as_str().unwrap_or("docs-root").to_string();
        let path = entry["path"].as_str().unwrap_or_default();
        if is_vendored_docs_registry_path(path) {
            continue;
        }
        *per_crate.entry(bucket).or_default() += 1;
    }
    for (bucket, count) in per_crate {
        let budget = policy
            .area_budgets
            .get(&bucket)
            .copied()
            .unwrap_or(policy.default_area_budget);
        if count > budget {
            errors.push(format!(
                "DOCS_REGISTRY_DOC_BUDGET_ERROR: `{bucket}` has {count} docs (budget={budget})"
            ));
        }
    }
    let registered = docs
        .iter()
        .filter_map(|v| v["path"].as_str())
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let mut incoming = BTreeMap::<String, usize>::new();
    let link_re = Regex::new(r"\[[^\]]+\]\(([^)]+)\)").map_err(|e| e.to_string())?;
    for file in scan_registry_markdown_files(&ctx.repo_root) {
        let source = file
            .strip_prefix(&ctx.repo_root)
            .ok()
            .map(|v| v.display().to_string())
            .unwrap_or_default();
        let text = fs::read_to_string(&file).unwrap_or_default();
        for cap in link_re.captures_iter(&text) {
            let target = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
            if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
                || target.starts_with('#')
            {
                continue;
            }
            let path_part = target.split('#').next().unwrap_or_default();
            if path_part.is_empty() {
                continue;
            }
            let resolved = file.parent().unwrap_or(&ctx.repo_root).join(path_part);
            if let Ok(rel) = resolved.strip_prefix(&ctx.repo_root) {
                let rels = rel.display().to_string();
                if registered.contains(&rels) {
                    *incoming.entry(rels).or_default() += 1;
                }
            }
        }
        let _ = source;
    }
    for path in &registered {
        if is_vendored_docs_registry_path(path) {
            continue;
        }
        let basename = Path::new(path)
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        if matches!(basename, "README.md" | "INDEX.md") {
            continue;
        }
        if incoming.get(path).copied().unwrap_or(0) == 0 {
            warnings.push(format!(
                "DOCS_REGISTRY_UNUSED_DOC_WARN: `{path}` has no inbound doc links"
            ));
        }
    }
    let root_md = read_dir_entries(&ctx.repo_root)
        .into_iter()
        .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    for file in root_md {
        let name = file
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        if !matches!(
            name,
            "README.md" | "CONTRIBUTING.md" | "SECURITY.md" | "CHANGELOG.md"
        ) {
            errors.push(format!(
                "DOCS_REGISTRY_ROOT_DOC_FORBIDDEN: allowed root docs are README/CONTRIBUTING/SECURITY/CHANGELOG, found `{}`",
                name
            ));
        }
    }
    for file in walk_files_local(&ctx.repo_root) {
        if file.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let Ok(rel) = file.strip_prefix(&ctx.repo_root) else {
            continue;
        };
        let rels = rel.to_string_lossy().to_string();
        if rels.starts_with("artifacts/") || rels.contains("/target/") {
            continue;
        }
        if !is_allowed_doc_location(&rels) {
            errors.push(format!(
                "DOCS_REGISTRY_DOC_LOCATION_FORBIDDEN: `{}` is outside allowed documentation directories",
                rels
            ));
        }
    }
    let (crate_rows, crate_errors, crate_warnings) = crate_doc_contract_status(&ctx.repo_root);
    errors.extend(crate_errors);
    warnings.extend(crate_warnings);
    let (canonical_errors, canonical_warnings, canonical_summary) =
        canonical_reference_checks(&ctx.repo_root, &docs);
    errors.extend(canonical_errors);
    warnings.extend(canonical_warnings);
    warnings.sort();
    warnings.dedup();
    errors.sort();
    errors.dedup();
    let pruning = warnings
        .iter()
        .filter(|w| {
            w.starts_with("DOCS_REGISTRY_OUTDATED:")
                || w.starts_with("DOCS_REGISTRY_UNUSED_DOC_WARN:")
                || w.starts_with("DOCS_REGISTRY_DUPLICATE_TOPIC:")
        })
        .cloned()
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "schema_version": 1,
        "errors": errors,
        "warnings": warnings,
        "crate_docs": crate_rows,
        "pruning_suggestions": pruning,
        "canonical_references": canonical_summary,
        "summary": {
            "registered": docs.len(),
            "errors": errors.len(),
            "warnings": warnings.len()
        }
    }))
}
