fn has_required_section(text: &str, section: &str) -> bool {
    let needle = format!("## {section}");
    text.lines().any(|line| line.trim() == needle)
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

fn docs_verify_contracts_payload(
    ctx: &DocsContext,
    common: &DocsCommonArgs,
) -> Result<serde_json::Value, String> {
    let mut errors = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    let mut scanned_files = 0usize;
    let mut runtime_examples = 0usize;
    let mut dev_examples = 0usize;
    let scripts_areas = format!("{}/{}", "scripts", "areas");
    let x_task = ["x", "task"].join("");
    let forbidden = [x_task, scripts_areas];
    let policy = load_quality_policy(&ctx.repo_root);
    let tags_allowlist_path = ctx
        .repo_root
        .join("configs/sources/repository/docs/tag-vocabulary.json");
    let allowed_tags = if tags_allowlist_path.exists() {
        let value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&tags_allowlist_path)
                .map_err(|e| format!("failed to read {}: {e}", tags_allowlist_path.display()))?,
        )
        .map_err(|e| format!("invalid {}: {e}", tags_allowlist_path.display()))?;
        value["allowed_tags"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str())
            .map(str::to_string)
            .collect::<std::collections::BTreeSet<_>>()
    } else {
        std::collections::BTreeSet::new()
    };
    let expected_frontmatter_order = [
        "title",
        "audience",
        "type",
        "stability",
        "owner",
        "last_reviewed",
        "tags",
        "related",
    ];
    let review_window_stable_days = 365i64;
    let review_window_experimental_days = 180i64;
    let canonical_index_links = [
        "01-introduction/index.md",
        "02-getting-started/index.md",
        "03-user-guide/index.md",
        "04-operations/index.md",
        "05-architecture/index.md",
        "06-development/index.md",
        "07-reference/index.md",
        "08-contracts/index.md",
    ];

    for file in docs_markdown_files(&ctx.docs_root, common.include_drafts) {
        scanned_files += 1;
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let text = fs::read_to_string(&file).map_err(|e| format!("failed to read {rel}: {e}"))?;
        runtime_examples += text.matches("bijux atlas ").count();
        dev_examples += text.matches("bijux dev atlas ").count();
        for needle in &forbidden {
            if text.contains(needle) {
                errors.push(format!(
                    "DOCS_CONTRACT_ERROR: forbidden `{needle}` reference in `{rel}`"
                ));
            }
        }
        if rel.starts_with("docs/_assets/") {
            continue;
        }
        let frontmatter = parse_frontmatter_contract_fields(&text);
        let owner = frontmatter
            .values
            .get("owner")
            .map(String::as_str)
            .unwrap_or_default()
            .trim();
        let stability = frontmatter
            .values
            .get("stability")
            .map(String::as_str)
            .unwrap_or_default()
            .trim();
        let last_reviewed = frontmatter
            .values
            .get("last_reviewed")
            .map(String::as_str)
            .unwrap_or_default()
            .trim();

        let enforce_contract_surface = rel == "docs/index.md"
            || rel == "docs/01-introduction/index.md"
            || rel == "docs/02-getting-started/index.md"
            || rel == "docs/03-user-guide/index.md"
            || rel == "docs/04-operations/index.md"
            || rel == "docs/05-architecture/index.md"
            || rel == "docs/06-development/index.md"
            || rel == "docs/07-reference/index.md";

        if enforce_contract_surface && owner.is_empty() {
            errors.push(format!(
                "DOCS_OWNER_REQUIRED: `{rel}` must declare front matter owner"
            ));
        }
        if enforce_contract_surface
            && !matches!(
                stability,
                "stable" | "experimental" | "deprecated" | "internal"
            )
        {
            errors.push(format!(
                "DOCS_STABILITY_INVALID: `{rel}` uses unsupported stability `{stability}`"
            ));
        }
        if enforce_contract_surface && matches!(stability, "stable" | "experimental") {
            if let Some(age_days) = date_diff_days(last_reviewed, &policy.reference_date) {
                let max_age = if stability == "stable" {
                    review_window_stable_days
                } else {
                    review_window_experimental_days
                };
                if age_days > max_age {
                    errors.push(format!(
                        "DOCS_REVIEW_WINDOW_EXCEEDED: `{rel}` stability=`{stability}` last_reviewed={last_reviewed} age_days={age_days} budget_days={max_age}"
                    ));
                }
            } else {
                errors.push(format!(
                    "DOCS_LAST_REVIEWED_INVALID: `{rel}` must use YYYY-MM-DD in `last_reviewed`"
                ));
            }
        }
        let lower = text.to_ascii_lowercase();
        if enforce_contract_surface
            && lower.contains("production-ready")
            && stability == "experimental"
        {
            errors.push(format!(
                "DOCS_PRODUCTION_CLAIM_MISMATCH: `{rel}` claims production-ready while stability is experimental"
            ));
        }
        if enforce_contract_surface
            && stability == "stable"
            && (text.contains("TODO") || text.contains("TBD"))
        {
            errors.push(format!(
                "DOCS_STABLE_PLACEHOLDER_FORBIDDEN: `{rel}` contains TODO/TBD while stability=stable"
            ));
        }
        if enforce_contract_surface && !frontmatter.keys.is_empty() {
            let filtered = frontmatter
                .keys
                .iter()
                .filter(|k| expected_frontmatter_order.contains(&k.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            let expected = expected_frontmatter_order
                .iter()
                .filter(|k| filtered.iter().any(|seen| seen == *k))
                .map(|k| (*k).to_string())
                .collect::<Vec<_>>();
            if filtered != expected {
                errors.push(format!(
                    "DOCS_FRONTMATTER_ORDER_ERROR: `{rel}` must follow key order `{}`",
                    expected_frontmatter_order.join(", ")
                ));
            }
        }
        if enforce_contract_surface && !allowed_tags.is_empty() {
            for tag in &frontmatter.tags {
                if !allowed_tags.contains(tag) {
                    errors.push(format!(
                        "DOCS_TAG_VOCABULARY_ERROR: `{rel}` uses disallowed tag `{tag}`"
                    ));
                }
            }
        }
        if rel == "docs/index.md" {
            for needle in canonical_index_links {
                if !text.contains(needle) {
                    errors.push(format!(
                        "DOCS_INDEX_CANONICAL_LINK_REQUIRED: `docs/index.md` missing canonical link `{needle}`"
                    ));
                }
            }
        }
    }
    if runtime_examples == 0 {
        warnings
            .push("DOCS_CONTRACT_ERROR: no `bijux atlas ...` examples found in docs/".to_string());
    }
    if dev_examples == 0 {
        warnings.push(
            "DOCS_CONTRACT_ERROR: no `bijux dev atlas ...` examples found in docs/".to_string(),
        );
    }
    if common.strict {
        errors.append(&mut warnings);
    }
    let text = if errors.is_empty() {
        "docs verify-contracts passed".to_string()
    } else {
        "docs verify-contracts failed".to_string()
    };
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": text,
        "errors": errors,
        "warnings": warnings,
        "summary": {
            "files_scanned": scanned_files,
            "runtime_examples": runtime_examples,
            "dev_examples": dev_examples
        },
        "capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},
        "options": {"strict": common.strict, "include_drafts": common.include_drafts}
    }))
}

#[derive(Default)]
struct FrontmatterContractFields {
    keys: Vec<String>,
    values: std::collections::BTreeMap<String, String>,
    tags: Vec<String>,
}

fn parse_frontmatter_contract_fields(text: &str) -> FrontmatterContractFields {
    let mut result = FrontmatterContractFields::default();
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return result;
    }
    let mut active_key: Option<String> = None;
    for line in lines {
        let trimmed = line.trim_end();
        if trimmed.trim() == "---" {
            break;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let key = key.trim().to_string();
            if !key.is_empty() {
                result.keys.push(key.clone());
                result.values.insert(
                    key.clone(),
                    value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string(),
                );
                active_key = Some(key);
            }
            continue;
        }
        if trimmed.trim_start().starts_with("- ") && active_key.as_deref() == Some("tags") {
            result.tags.push(
                trimmed
                    .trim_start()
                    .trim_start_matches("- ")
                    .trim()
                    .to_string(),
            );
        }
    }
    result
}

pub(crate) fn docs_lint_payload(
    ctx: &DocsContext,
    common: &DocsCommonArgs,
) -> Result<serde_json::Value, String> {
    let policy = load_quality_policy(&ctx.repo_root);
    let mkdocs_text = fs::read_to_string(ctx.repo_root.join("mkdocs.yml")).unwrap_or_default();
    let mut errors = Vec::<String>::new();
    let docs_lint_root = ctx.repo_root.join("configs/sources/repository/docs");
    let required_lint_files = [
        "configs/sources/repository/docs/.vale.ini",
        "configs/sources/repository/docs/.markdownlint-cli2.jsonc",
        "configs/sources/repository/docs/cspell.config.json",
        "configs/sources/repository/docs/package.json",
        "configs/sources/repository/docs/package-lock.json",
    ];
    for rel in required_lint_files {
        if !ctx.repo_root.join(rel).exists() {
            errors.push(format!(
                "DOCS_LINT_CONFIG_MISSING: required lint config `{rel}` is missing"
            ));
        }
    }
    let package_json_path = docs_lint_root.join("package.json");
    if package_json_path.exists() {
        let package_text = fs::read_to_string(&package_json_path)
            .map_err(|e| format!("failed to read {}: {e}", package_json_path.display()))?;
        let package_json: serde_json::Value = serde_json::from_str(&package_text)
            .map_err(|e| format!("failed to parse {}: {e}", package_json_path.display()))?;
        let scripts = package_json
            .get("scripts")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        for key in ["markdownlint", "cspell"] {
            if !scripts.contains_key(key) {
                errors.push(format!(
                    "DOCS_LINT_CONFIG_INVALID: `configs/sources/repository/docs/package.json` missing script `{key}`"
                ));
            }
        }
    }
    let vale_ini_path = docs_lint_root.join(".vale.ini");
    if vale_ini_path.exists() {
        let vale_text = fs::read_to_string(&vale_ini_path)
            .map_err(|e| format!("failed to read {}: {e}", vale_ini_path.display()))?;
        if !vale_text.contains("StylesPath") {
            errors.push(
                "DOCS_LINT_CONFIG_INVALID: `configs/sources/repository/docs/.vale.ini` must declare `StylesPath`"
                    .to_string(),
            );
        }
        let styles_dir = docs_lint_root.join(".vale");
        if !styles_dir.exists() {
            errors.push(
                "DOCS_LINT_CONFIG_INVALID: `configs/sources/repository/docs/.vale` directory must exist".to_string(),
            );
        }
    }
    let adr_filename_re = Regex::new(r"^ADR-\d{4}-[a-z0-9-]+\.md$").map_err(|e| e.to_string())?;
    let schema_ref_re =
        Regex::new(r"`([^`\n]*schema[^`\n]*\.(json|ya?ml))`").map_err(|e| e.to_string())?;
    for file in docs_markdown_files(&ctx.docs_root, common.include_drafts) {
        let rel = file
            .strip_prefix(&ctx.docs_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        if rel.contains(' ') {
            errors.push(format!("docs filename must not contain spaces: `{rel}`"));
        }
        let name = file
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        let stem = name.strip_suffix(".md").unwrap_or(name);
        let is_canonical_upper_doc = !stem.is_empty()
            && stem
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_' || c == '-');
        let is_adr_filename = adr_filename_re.is_match(name);
        if !matches!(name, "README.md" | "INDEX.md")
            && !is_canonical_upper_doc
            && !is_adr_filename
            && name.chars().any(|c| c.is_ascii_uppercase())
        {
            errors.push(format!(
                "docs filename should use lowercase intent-based naming: `{rel}`"
            ));
        }
        let rel_lower = rel.to_ascii_lowercase();
        for word in &policy.naming.forbidden_words {
            if rel_lower.contains(word) {
                errors.push(format!(
                    "docs filename uses forbidden transitional term `{word}`: `{rel}`"
                ));
            }
        }
        let text = fs::read_to_string(&file).map_err(|e| format!("failed to read {rel}: {e}"))?;
        if policy.markdown.require_h1
            && !text.lines().any(|line| line.trim_start().starts_with("# "))
        {
            errors.push(format!("DOCS_MARKDOWN_H1_REQUIRED: `{rel}`"));
        }
        for section in &policy.markdown.require_sections {
            if !has_required_section(&text, section) {
                errors.push(format!(
                    "DOCS_SECTION_REQUIRED: `{rel}` missing `## {section}`"
                ));
            }
        }
        for (idx, line) in text.lines().enumerate() {
            if line.ends_with(' ') || line.contains('\t') {
                errors.push(format!(
                    "{rel}:{} formatting lint failure (tab/trailing-space)",
                    idx + 1
                ));
            }
            if line.len() > policy.markdown.max_line_length
                && !line.trim_start().starts_with("http")
            {
                errors.push(format!(
                    "DOCS_MARKDOWN_LINE_LENGTH: `{rel}` line {} exceeds {} chars",
                    idx + 1,
                    policy.markdown.max_line_length
                ));
            }
            for term in &policy.terminology.forbidden_terms {
                if line
                    .to_ascii_lowercase()
                    .contains(&term.to_ascii_lowercase())
                {
                    errors.push(format!(
                        "DOCS_TERMINOLOGY_ERROR: `{rel}` line {} contains forbidden term `{term}`",
                        idx + 1
                    ));
                }
            }
            for cap in schema_ref_re.captures_iter(line) {
                let schema_ref = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
                if !schema_ref.is_empty() && !ctx.repo_root.join(schema_ref).exists() {
                    errors.push(format!(
                        "DOCS_SCHEMA_REF_ERROR: `{rel}` line {} references missing schema `{schema_ref}`",
                        idx + 1
                    ));
                }
            }
        }
        let fence_count = text
            .lines()
            .filter(|line| line.trim_start().starts_with("```"))
            .count();
        if fence_count % 2 != 0 {
            errors.push(format!(
                "DOCS_EXAMPLE_FENCE_ERROR: `{rel}` has unbalanced fenced code blocks"
            ));
        }
        let mut table_width: Option<usize> = None;
        for (idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('|') && trimmed.ends_with('|') {
                let width = trimmed.matches('|').count() - 1;
                if let Some(expected) = table_width {
                    if width != expected {
                        errors.push(format!(
                            "DOCS_TABLE_CONSISTENCY_ERROR: `{rel}` line {} has {width} columns expected {expected}",
                            idx + 1
                        ));
                    }
                } else {
                    table_width = Some(width);
                }
            } else {
                table_width = None;
            }
        }
    }
    for root in &policy.diagrams.roots {
        let base = ctx.repo_root.join(root);
        if !base.exists() {
            continue;
        }
        for file in walk_files_local(&base) {
            let ext = file
                .extension()
                .and_then(|v| v.to_str())
                .map(|v| format!(".{}", v.to_ascii_lowercase()))
                .unwrap_or_default();
            if !policy
                .diagrams
                .extensions
                .iter()
                .any(|allowed| allowed == &ext)
            {
                continue;
            }
            let Ok(rel) = file.strip_prefix(&ctx.repo_root) else {
                continue;
            };
            let rels = rel.display().to_string();
            let mut referenced = rel
                .strip_prefix("docs/")
                .ok()
                .and_then(|path| path.to_str())
                .is_some_and(|path| mkdocs_text.contains(path));
            for doc in docs_markdown_files(&ctx.docs_root, common.include_drafts) {
                let text = fs::read_to_string(&doc).unwrap_or_default();
                if text.contains(&rels) {
                    referenced = true;
                    break;
                }
            }
            if !referenced {
                errors.push(format!(
                    "DOCS_DIAGRAM_ORPHAN_ERROR: `{rels}` is not referenced by any markdown document"
                ));
            }
        }
    }
    errors.sort();
    errors.dedup();
    Ok(
        serde_json::json!({"schema_version":1,"run_id":ctx.run_id.as_str(),"text": if errors.is_empty() {"docs lint passed"} else {"docs lint failed"},"rows":[],"errors":errors,"warnings":[],"capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},"options": {"strict": common.strict, "include_drafts": common.include_drafts}}),
    )
}

fn docs_grep_payload(
    ctx: &DocsContext,
    common: &DocsCommonArgs,
    pattern: &str,
) -> Result<serde_json::Value, String> {
    let mut rows = Vec::<serde_json::Value>::new();
    for file in docs_markdown_files(&ctx.docs_root, common.include_drafts) {
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let text = fs::read_to_string(&file).map_err(|e| format!("failed to read {rel}: {e}"))?;
        for (idx, line) in text.lines().enumerate() {
            if line.contains(pattern) {
                rows.push(serde_json::json!({"file": rel, "line": idx + 1, "text": line.trim()}));
            }
        }
    }
    rows.sort_by(|a, b| {
        a["file"]
            .as_str()
            .cmp(&b["file"].as_str())
            .then(a["line"].as_u64().cmp(&b["line"].as_u64()))
    });
    Ok(
        serde_json::json!({"schema_version":1,"run_id":ctx.run_id.as_str(),"text": format!("{} matches", rows.len()),"rows":rows,"capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},"options": {"strict": common.strict, "include_drafts": common.include_drafts}}),
    )
}
