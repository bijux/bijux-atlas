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

pub(crate) fn docs_lint_payload(
    ctx: &DocsContext,
    common: &DocsCommonArgs,
) -> Result<serde_json::Value, String> {
    let policy = load_quality_policy(&ctx.repo_root);
    let mut errors = Vec::<String>::new();
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
            let mut referenced = false;
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

