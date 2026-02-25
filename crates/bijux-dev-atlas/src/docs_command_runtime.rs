// SPDX-License-Identifier: Apache-2.0

use crate::docs_commands::{
    crate_doc_contract_status, docs_inventory_payload, docs_markdown_files, docs_registry_payload,
    has_required_section, load_quality_policy, registry_validate_payload, search_synonyms,
    workspace_crate_roots,
};
use crate::*;
use std::collections::{BTreeMap, BTreeSet};

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

fn docs_build_or_serve_subprocess(
    args: &[String],
    common: &DocsCommonArgs,
    label: &str,
) -> Result<(serde_json::Value, i32), String> {
    if !common.allow_subprocess {
        return Err(format!("{label} requires --allow-subprocess"));
    }
    if label == "docs build" && !common.allow_write {
        return Err("docs build requires --allow-write".to_string());
    }
    let ctx = docs_context(common)?;
    let output_dir = ctx
        .artifacts_root
        .join("dist")
        .join("docs-site")
        .join(ctx.run_id.as_str());
    if label == "docs build" {
        fs::create_dir_all(&output_dir)
            .map_err(|e| format!("failed to create {}: {e}", output_dir.display()))?;
    }
    let mut cmd = ProcessCommand::new("mkdocs");
    cmd.args(args).current_dir(&ctx.repo_root);
    if label == "docs build" {
        cmd.args([
            "--site-dir",
            output_dir.to_str().unwrap_or("artifacts/dist/docs-site"),
        ]);
    }
    let out = cmd
        .output()
        .map_err(|e| format!("failed to run mkdocs: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let code = out.status.code().unwrap_or(1);
    let mut files = Vec::<serde_json::Value>::new();
    if label == "docs build" && output_dir.exists() {
        for path in walk_files_local(&output_dir) {
            let Ok(bytes) = fs::read(&path) else { continue };
            let rel = path
                .strip_prefix(&output_dir)
                .unwrap_or(&path)
                .display()
                .to_string();
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            files.push(serde_json::json!({
                "path": rel,
                "sha256": format!("{:x}", hasher.finalize()),
                "bytes": bytes.len()
            }));
        }
        files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
        let index_path = ctx
            .artifacts_root
            .join("dist")
            .join("docs-site")
            .join(ctx.run_id.as_str())
            .join("build.index.json");
        if common.allow_write {
            if let Some(parent) = index_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(
                &index_path,
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": 1,
                    "run_id": ctx.run_id.as_str(),
                    "files": files
                }))
                .unwrap_or_default(),
            );
        }
    }
    Ok((
        serde_json::json!({
            "schema_version":1,
            "run_id": ctx.run_id.as_str(),
            "error_code": if code == 0 { serde_json::Value::Null } else { serde_json::Value::String("DOCS_BUILD_ERROR".to_string()) },
            "text": format!("{label} {}", if code==0 {"ok"} else {"failed"}),
            "rows":[{"command": args, "exit_code": code, "stdout": stdout, "stderr": stderr, "site_dir": output_dir.display().to_string()}],
            "artifacts": {"site_dir": output_dir.display().to_string(), "build_index": ctx.artifacts_root.join("dist").join("docs-site").join(ctx.run_id.as_str()).join("build.index.json").display().to_string(), "files": files},
            "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
            "options": {"strict": common.strict, "include_drafts": common.include_drafts}
        }),
        code,
    ))
}

pub(crate) fn run_docs_command(quiet: bool, command: DocsCommand) -> i32 {
    let run = (|| -> Result<(String, i32), String> {
        let started = std::time::Instant::now();
        match command {
            DocsCommand::Check(common) => {
                if !common.allow_subprocess {
                    return Err("docs check requires --allow-subprocess".to_string());
                }
                let ctx = docs_context(&common)?;
                let validate = docs_validate_payload(&ctx, &common)?;
                let links = docs_links_payload(&ctx, &common)?;
                let lint = docs_lint_payload(&ctx, &common)?;
                let (build_payload, build_code) =
                    docs_build_or_serve_subprocess(&["build".to_string()], &common, "docs build")?;
                let errors = validate["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + links["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + lint["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + usize::from(build_code != 0);
                let payload = serde_json::json!({
                    "schema_version":1,
                    "run_id":ctx.run_id.as_str(),
                    "text": if errors == 0 { "docs check passed" } else { "docs check failed" },
                    "rows":[
                        {"name":"validate","errors": validate["errors"].as_array().map(|v| v.len()).unwrap_or(0)},
                        {"name":"links","errors": links["errors"].as_array().map(|v| v.len()).unwrap_or(0)},
                        {"name":"lint","errors": lint["errors"].as_array().map(|v| v.len()).unwrap_or(0)},
                        {"name":"build","exit_code": build_code}
                    ],
                    "checks": {"validate": validate, "links": links, "lint": lint, "build": build_payload},
                    "counts":{"errors": errors},
                    "capabilities":{"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "options":{"strict": common.strict, "include_drafts": common.include_drafts},
                    "duration_ms": started.elapsed().as_millis() as u64,
                    "error_code": if errors == 0 { serde_json::Value::Null } else { serde_json::Value::String("DOCS_BUILD_ERROR".to_string()) }
                });
                Ok((
                    emit_payload(common.format, common.out, &payload)?,
                    if errors == 0 { 0 } else { 1 },
                ))
            }
            DocsCommand::VerifyContracts(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_verify_contracts_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                if code != 0 {
                    payload["error_code"] = serde_json::json!("DOCS_CONTRACT_ERROR");
                }
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::Validate(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_validate_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                if code != 0 {
                    payload["error_code"] = serde_json::json!("DOCS_NAV_ERROR");
                }
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::Inventory(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_inventory_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            DocsCommand::Registry { command } => match command {
                crate::cli::DocsRegistryCommand::Build(common) => {
                    let ctx = docs_context(&common)?;
                    let payload = docs_registry_payload(&ctx);
                    if common.allow_write {
                        let path = ctx.repo_root.join("docs/registry.json");
                        fs::write(
                            &path,
                            serde_json::to_string_pretty(&payload)
                                .map_err(|e| format!("registry encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("failed to write {}: {e}", path.display()))?;
                    }
                    let generated = docs_registry_payload(&ctx);
                    let docs_rows = generated["documents"]
                        .as_array()
                        .cloned()
                        .unwrap_or_default();
                    let mut search_index = Vec::new();
                    let mut graph = Vec::new();
                    let mut topic_index = BTreeMap::<String, Vec<String>>::new();
                    let mut crate_slice = BTreeMap::<String, Vec<serde_json::Value>>::new();
                    for doc in &docs_rows {
                        let path = doc["path"].as_str().unwrap_or_default().to_string();
                        let tags = doc["tags"].as_array().cloned().unwrap_or_default();
                        let keywords = doc["keywords"].as_array().cloned().unwrap_or_default();
                        search_index.push(serde_json::json!({
                            "path": path,
                            "topic": doc["topic"],
                            "tags": tags,
                            "keywords": keywords
                        }));
                        graph.push(serde_json::json!({
                            "from": path,
                            "crate": doc["crate"],
                            "doc_type": doc["doc_type"]
                        }));
                        if let Some(topic) = doc["topic"].as_str() {
                            topic_index
                                .entry(topic.to_string())
                                .or_default()
                                .push(path.clone());
                        }
                        if let Some(crate_name) = doc["crate"].as_str() {
                            crate_slice
                                .entry(crate_name.to_string())
                                .or_default()
                                .push(doc.clone());
                        }
                    }
                    let crate_coverage = crate_slice
                        .iter()
                        .map(|(name, rows)| {
                            serde_json::json!({
                                "crate": name,
                                "doc_count": rows.len()
                            })
                        })
                        .collect::<Vec<_>>();
                    if common.allow_write {
                        let generated_dir = ctx.repo_root.join("docs/_generated");
                        fs::create_dir_all(&generated_dir).map_err(|e| {
                            format!("failed to create {}: {e}", generated_dir.display())
                        })?;
                        fs::write(
                            generated_dir.join("search-index.json"),
                            serde_json::to_string_pretty(&serde_json::json!({
                                "schema_version": 1,
                                "entries": search_index,
                                "synonyms": search_synonyms(&ctx.repo_root)
                            }))
                            .map_err(|e| format!("search index encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("write search index failed: {e}"))?;
                        let sitemap = docs_rows
                            .iter()
                            .filter_map(|row| row["path"].as_str().map(ToString::to_string))
                            .collect::<Vec<_>>();
                        fs::write(
                            generated_dir.join("sitemap.json"),
                            serde_json::to_string_pretty(&serde_json::json!({
                                "schema_version": 1,
                                "paths": sitemap
                            }))
                            .map_err(|e| format!("sitemap encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("write sitemap failed: {e}"))?;
                        fs::write(
                            generated_dir.join("topic-index.json"),
                            serde_json::to_string_pretty(&serde_json::json!({
                                "schema_version": 1,
                                "topics": topic_index
                            }))
                            .map_err(|e| format!("topic index encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("write topic index failed: {e}"))?;
                        let breadcrumbs = docs_rows
                            .iter()
                            .filter_map(|row| row["path"].as_str())
                            .map(|path| {
                                let crumbs = path
                                    .split('/')
                                    .scan(String::new(), |state, seg| {
                                        if !state.is_empty() {
                                            state.push('/');
                                        }
                                        state.push_str(seg);
                                        Some(state.clone())
                                    })
                                    .collect::<Vec<_>>();
                                serde_json::json!({"path": path, "breadcrumbs": crumbs})
                            })
                            .collect::<Vec<_>>();
                        fs::write(
                            generated_dir.join("breadcrumbs.json"),
                            serde_json::to_string_pretty(&serde_json::json!({
                                "schema_version": 1,
                                "rows": breadcrumbs
                            }))
                            .map_err(|e| format!("breadcrumbs encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("write breadcrumbs failed: {e}"))?;
                        fs::write(
                            generated_dir.join("docs-dependency-graph.json"),
                            serde_json::to_string_pretty(&serde_json::json!({
                                "schema_version": 1,
                                "edges": graph
                            }))
                            .map_err(|e| format!("graph encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("write dependency graph failed: {e}"))?;
                        fs::write(
                            generated_dir.join("crate-docs-slice.json"),
                            serde_json::to_string_pretty(&serde_json::json!({
                                "schema_version": 1,
                                "crates": crate_slice
                            }))
                            .map_err(|e| format!("crate docs slice encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("write crate docs slice failed: {e}"))?;
                        let docs_with_owner = docs_rows
                            .iter()
                            .filter(|row| {
                                row["owner"]
                                    .as_str()
                                    .is_some_and(|owner| !owner.is_empty() && owner != "unknown")
                            })
                            .count();
                        let docs_with_stability = docs_rows
                            .iter()
                            .filter(|row| row["stability"].as_str().is_some_and(|v| !v.is_empty()))
                            .count();
                        fs::write(
                            generated_dir.join("docs-test-coverage.json"),
                            serde_json::to_string_pretty(&serde_json::json!({
                                "schema_version": 1,
                                "documents_total": docs_rows.len(),
                                "owner_metadata_count": docs_with_owner,
                                "stability_metadata_count": docs_with_stability,
                                "owner_metadata_ratio": if docs_rows.is_empty() { 1.0 } else { docs_with_owner as f64 / docs_rows.len() as f64 },
                                "stability_metadata_ratio": if docs_rows.is_empty() { 1.0 } else { docs_with_stability as f64 / docs_rows.len() as f64 }
                            }))
                            .map_err(|e| format!("docs test coverage encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("write docs test coverage failed: {e}"))?;
                        fs::write(
                            generated_dir.join("crate-doc-coverage.json"),
                            serde_json::to_string_pretty(&serde_json::json!({
                                "schema_version": 1,
                                "rows": crate_coverage
                            }))
                            .map_err(|e| format!("crate coverage encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("write crate coverage failed: {e}"))?;
                        let (crate_doc_rows, crate_doc_errors, crate_doc_warnings) =
                            crate_doc_contract_status(&ctx.repo_root);
                        fs::write(
                            generated_dir.join("crate-doc-governance.json"),
                            serde_json::to_string_pretty(&serde_json::json!({
                                "schema_version": 1,
                                "rows": crate_doc_rows,
                                "errors": crate_doc_errors,
                                "warnings": crate_doc_warnings
                            }))
                            .map_err(|e| format!("crate doc governance encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("write crate doc governance failed: {e}"))?;
                        let mut crate_governance_md = String::from("# Crate Docs Governance\n\n");
                        crate_governance_md.push_str("| Crate | Root Docs | docs/ Files | Diagrams |\n|---|---:|---:|---:|\n");
                        for row in &crate_doc_rows {
                            crate_governance_md.push_str(&format!(
                                "| `{}` | {} | {} | {} |\n",
                                row["crate"].as_str().unwrap_or_default(),
                                row["root_doc_count"].as_u64().unwrap_or(0),
                                row["docs_dir_count"].as_u64().unwrap_or(0),
                                row["diagram_count"].as_u64().unwrap_or(0),
                            ));
                        }
                        fs::write(
                            generated_dir.join("crate-doc-governance.md"),
                            crate_governance_md,
                        )
                        .map_err(|e| format!("write crate doc governance page failed: {e}"))?;
                        let mut crate_api_table = String::from(
                            "# Crate Public API Table\n\n| Crate | Public API Doc |\n|---|---|\n",
                        );
                        for crate_root in workspace_crate_roots(&ctx.repo_root) {
                            let crate_name = crate_root
                                .file_name()
                                .and_then(|v| v.to_str())
                                .unwrap_or("unknown");
                            let public_api = crate_root.join("docs/public-api.md");
                            let public_api_value = if public_api.exists() {
                                format!(
                                    "`{}`",
                                    public_api
                                        .strip_prefix(&ctx.repo_root)
                                        .unwrap_or(&public_api)
                                        .display()
                                )
                            } else {
                                "`missing`".to_string()
                            };
                            crate_api_table
                                .push_str(&format!("| `{crate_name}` | {public_api_value} |\n"));
                        }
                        fs::write(
                            generated_dir.join("crate-doc-api-table.md"),
                            crate_api_table,
                        )
                        .map_err(|e| format!("write crate doc api table failed: {e}"))?;
                        let pruning_rows = crate_doc_warnings
                            .iter()
                            .filter(|w| {
                                w.starts_with("CRATE_DOC_DUPLICATE_CONCEPT_WARN:")
                                    || w.starts_with("CRATE_DOC_BUDGET_ERROR:")
                                    || w.starts_with("CRATE_DOC_ALLOWED_TYPE_WARN:")
                            })
                            .cloned()
                            .collect::<Vec<_>>();
                        fs::write(
                            generated_dir.join("crate-doc-pruning.json"),
                            serde_json::to_string_pretty(&serde_json::json!({
                                "schema_version": 1,
                                "rows": pruning_rows
                            }))
                            .map_err(|e| format!("crate doc pruning encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("write crate doc pruning failed: {e}"))?;
                        let mut inventory_md =
                            String::from("# Docs Inventory\n\nLicense: Apache-2.0\n\n");
                        inventory_md
                            .push_str("| Path | Type | Owner | Stability |\n|---|---|---|---|\n");
                        for row in &docs_rows {
                            inventory_md.push_str(&format!(
                                "| `{}` | `{}` | `{}` | `{}` |\n",
                                row["path"].as_str().unwrap_or_default(),
                                row["doc_type"].as_str().unwrap_or_default(),
                                row["owner"].as_str().unwrap_or_default(),
                                row["stability"].as_str().unwrap_or_default()
                            ));
                        }
                        fs::write(generated_dir.join("docs-inventory.md"), inventory_md)
                            .map_err(|e| format!("write docs inventory page failed: {e}"))?;
                        let mut topic_md = String::from("# Topic Index\n\n");
                        topic_md.push_str("| Topic | Paths |\n|---|---|\n");
                        for (topic, paths) in &topic_index {
                            topic_md.push_str(&format!(
                                "| `{}` | `{}` |\n",
                                topic,
                                paths.join(", ")
                            ));
                        }
                        fs::write(generated_dir.join("topic-index.md"), topic_md)
                            .map_err(|e| format!("write topic index page failed: {e}"))?;
                        let make_registry_path =
                            ctx.repo_root.join("configs/ops/make-target-registry.json");
                        if make_registry_path.exists() {
                            let make_registry_text = fs::read_to_string(&make_registry_path)
                                .map_err(|e| format!("read make target registry failed: {e}"))?;
                            let make_registry: serde_json::Value =
                                serde_json::from_str(&make_registry_text).map_err(|e| {
                                    format!("parse make target registry failed: {e}")
                                })?;
                            let mut generated_make = String::from("# Generated Make Targets\n\n");
                            generated_make.push_str(
                                "This file is generated by `bijux dev atlas docs registry build --allow-write`.\n\n",
                            );
                            generated_make
                                .push_str("| Target | Visibility | Defined In |\n|---|---|---|\n");
                            for row in make_registry["targets"].as_array().into_iter().flatten() {
                                let name = row["name"].as_str().unwrap_or_default();
                                let visibility = row["visibility"].as_str().unwrap_or_default();
                                let defined_in = row["defined_in"]
                                    .as_array()
                                    .map(|v| {
                                        v.iter()
                                            .filter_map(|s| s.as_str())
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    })
                                    .unwrap_or_default();
                                generated_make.push_str(&format!(
                                    "| `{name}` | `{visibility}` | `{defined_in}` |\n"
                                ));
                            }
                            fs::write(
                                ctx.repo_root.join("makefiles/GENERATED_TARGETS.md"),
                                generated_make,
                            )
                            .map_err(|e| format!("write generated make targets failed: {e}"))?;
                        }
                        let command_rows = docs_rows
                            .iter()
                            .filter(|row| {
                                row["path"].as_str().is_some_and(|p| {
                                    p.contains("COMMAND") || p.contains("CLI_COMMAND")
                                })
                            })
                            .cloned()
                            .collect::<Vec<_>>();
                        fs::write(
                            generated_dir.join("command-index.json"),
                            serde_json::to_string_pretty(&serde_json::json!({
                                "schema_version": 1,
                                "rows": command_rows
                            }))
                            .map_err(|e| format!("command index encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("write command index failed: {e}"))?;
                        let schema_rows = docs_rows
                            .iter()
                            .filter(|row| {
                                row["path"].as_str().is_some_and(|p| p.contains("SCHEMA"))
                            })
                            .cloned()
                            .collect::<Vec<_>>();
                        fs::write(
                            generated_dir.join("schema-index.json"),
                            serde_json::to_string_pretty(&serde_json::json!({
                                "schema_version": 1,
                                "rows": schema_rows
                            }))
                            .map_err(|e| format!("schema index encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("write schema index failed: {e}"))?;
                        let registry_checks = registry_validate_payload(&ctx)?;
                        fs::write(
                            generated_dir.join("docs-quality-dashboard.json"),
                            serde_json::to_string_pretty(&serde_json::json!({
                                "schema_version": 1,
                                "kind": "docs_quality_dashboard_v1",
                                "summary": registry_checks["summary"].clone(),
                                "canonical_references": registry_checks["canonical_references"].clone(),
                                "pruning_suggestions": registry_checks["pruning_suggestions"].clone()
                            }))
                            .map_err(|e| format!("docs quality dashboard encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("write docs quality dashboard failed: {e}"))?;
                    }
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "run_id": ctx.run_id.as_str(),
                        "text": "docs registry build completed",
                        "summary": {
                            "documents": docs_rows.len(),
                            "areas": docs_rows.iter().filter_map(|v| v["path"].as_str()).map(|v| v.split('/').nth(1).unwrap_or("root")).collect::<BTreeSet<_>>().len()
                        },
                        "coverage": {
                            "registered": docs_rows.len(),
                            "areas_covered": docs_rows.iter().filter_map(|v| v["path"].as_str()).map(|v| v.split('/').nth(1).unwrap_or("root")).collect::<BTreeSet<_>>().len()
                        },
                        "artifacts": {
                            "registry": "docs/registry.json",
                            "inventory_page": "docs/_generated/docs-inventory.md",
                            "search_index": "docs/_generated/search-index.json",
                            "sitemap": "docs/_generated/sitemap.json",
                            "topic_index": "docs/_generated/topic-index.json",
                            "breadcrumbs": "docs/_generated/breadcrumbs.json",
                            "dependency_graph": "docs/_generated/docs-dependency-graph.json",
                            "crate_docs_slice": "docs/_generated/crate-docs-slice.json",
                            "docs_test_coverage": "docs/_generated/docs-test-coverage.json",
                            "crate_doc_coverage": "docs/_generated/crate-doc-coverage.json",
                            "crate_doc_governance": "docs/_generated/crate-doc-governance.json",
                            "crate_doc_api_table": "docs/_generated/crate-doc-api-table.md",
                            "crate_doc_pruning": "docs/_generated/crate-doc-pruning.json",
                            "command_index": "docs/_generated/command-index.json",
                            "schema_index": "docs/_generated/schema-index.json",
                            "docs_quality_dashboard": "docs/_generated/docs-quality-dashboard.json",
                            "generated_make_targets": "makefiles/GENERATED_TARGETS.md"
                        },
                        "changes_summary": {
                            "message": "docs registry updated",
                            "ci_hint": "attach docs registry delta to job summary"
                        }
                    });
                    Ok((emit_payload(common.format, common.out, &payload)?, 0))
                }
                crate::cli::DocsRegistryCommand::Validate(common) => {
                    let ctx = docs_context(&common)?;
                    let mut payload = registry_validate_payload(&ctx)?;
                    payload["run_id"] = serde_json::json!(ctx.run_id.as_str());
                    payload["duration_ms"] =
                        serde_json::json!(started.elapsed().as_millis() as u64);
                    let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                        1
                    } else {
                        0
                    };
                    Ok((emit_payload(common.format, common.out, &payload)?, code))
                }
            },
            DocsCommand::Links(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_links_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                if code != 0 {
                    payload["error_code"] = serde_json::json!("DOCS_LINK_ERROR");
                }
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::Lint(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_lint_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::Grep(args) => {
                let ctx = docs_context(&args.common)?;
                let mut payload = docs_grep_payload(&ctx, &args.common, &args.pattern)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((
                    emit_payload(args.common.format, args.common.out, &payload)?,
                    0,
                ))
            }
            DocsCommand::Build(common) => {
                let (mut payload, code) =
                    docs_build_or_serve_subprocess(&["build".to_string()], &common, "docs build")?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::Serve(args) => {
                if !args.common.allow_network {
                    return Err("docs serve requires --allow-network".to_string());
                }
                let (mut payload, code) = docs_build_or_serve_subprocess(
                    &[
                        "serve".to_string(),
                        "--dev-addr".to_string(),
                        format!("{}:{}", args.host, args.port),
                    ],
                    &args.common,
                    "docs serve",
                )?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((
                    emit_payload(args.common.format, args.common.out, &payload)?,
                    code,
                ))
            }
            DocsCommand::Clean(common) => {
                if !common.allow_write {
                    return Err("docs clean requires --allow-write".to_string());
                }
                let ctx = docs_context(&common)?;
                let target = ctx.artifacts_root.join("atlas-dev").join("docs");
                if target.exists() {
                    fs::remove_dir_all(&target)
                        .map_err(|e| format!("failed to remove {}: {e}", target.display()))?;
                }
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": ctx.run_id.as_str(),
                    "text": format!("docs clean removed {}", target.display()),
                    "rows": [{"path": target.display().to_string()}],
                    "capabilities":{"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "options":{"strict": common.strict, "include_drafts": common.include_drafts},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            DocsCommand::Doctor(common) => {
                let ctx = docs_context(&common)?;
                let validate = docs_validate_payload(&ctx, &common)?;
                let links = docs_links_payload(&ctx, &common)?;
                let lint = docs_lint_payload(&ctx, &common)?;
                let mut rows = Vec::<serde_json::Value>::new();
                rows.push(serde_json::json!({"name":"validate","errors":validate["errors"].as_array().map(|v| v.len()).unwrap_or(0)}));
                rows.push(serde_json::json!({"name":"links","errors":links["errors"].as_array().map(|v| v.len()).unwrap_or(0)}));
                rows.push(serde_json::json!({"name":"lint","errors":lint["errors"].as_array().map(|v| v.len()).unwrap_or(0)}));
                let mut build_status = "skipped";
                if common.allow_subprocess && common.allow_write {
                    let (_payload, code) = docs_build_or_serve_subprocess(
                        &["build".to_string()],
                        &common,
                        "docs build",
                    )?;
                    build_status = if code == 0 { "ok" } else { "failed" };
                }
                rows.push(serde_json::json!({"name":"build","status":build_status}));
                let errors = validate["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + links["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + lint["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + usize::from(build_status == "failed");
                let payload = serde_json::json!({
                    "schema_version":1,
                    "run_id":ctx.run_id.as_str(),
                    "text": if errors==0 {
                        format!("docs: 4 checks collected, 0 failed, build={build_status}")
                    } else {
                        format!("docs: 4 checks collected, {errors} failed, build={build_status}")
                    },
                    "rows":rows,
                    "counts":{"errors":errors},
                    "capabilities":{"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "options":{"strict": common.strict, "include_drafts": common.include_drafts},
                    "duration_ms": started.elapsed().as_millis() as u64,
                    "error_code": if errors == 0 { serde_json::Value::Null } else { serde_json::Value::String("DOCS_NAV_ERROR".to_string()) }
                });
                Ok((
                    emit_payload(common.format, common.out, &payload)?,
                    if errors == 0 { 0 } else { 1 },
                ))
            }
        }
    })();
    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                if code == 0 {
                    println!("{rendered}");
                } else {
                    eprintln!("{rendered}");
                }
            }
            code
        }
        Err(err) => {
            eprintln!("bijux-dev-atlas docs failed: {err}");
            1
        }
    }
}
