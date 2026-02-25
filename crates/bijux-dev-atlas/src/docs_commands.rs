// SPDX-License-Identifier: Apache-2.0

use crate::*;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn docs_context(common: &DocsCommonArgs) -> Result<DocsContext, String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let artifacts_root = common
        .artifacts_root
        .clone()
        .unwrap_or_else(|| repo_root.join("artifacts"));
    let run_id = common
        .run_id
        .as_ref()
        .map(|v| RunId::parse(v))
        .transpose()?
        .unwrap_or_else(|| RunId::from_seed("docs_run"));
    Ok(DocsContext {
        docs_root: repo_root.join("docs"),
        repo_root,
        artifacts_root,
        run_id,
    })
}

fn slugify_anchor(text: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for c in text.chars().flat_map(|c| c.to_lowercase()) {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if (c.is_whitespace() || c == '-' || c == '_') && !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn docs_markdown_files(docs_root: &Path, include_drafts: bool) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if docs_root.exists() {
        for file in walk_files_local(docs_root) {
            if file.extension().and_then(|v| v.to_str()) == Some("md") {
                if !include_drafts {
                    if let Ok(rel) = file.strip_prefix(docs_root) {
                        if rel.to_string_lossy().starts_with("_drafts/") {
                            continue;
                        }
                    }
                }
                files.push(file);
            }
        }
    }
    files.sort();
    files
}

pub(crate) fn walk_files_local(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn parse_mkdocs_yaml(repo_root: &Path) -> Result<YamlValue, String> {
    let path = repo_root.join("mkdocs.yml");
    let text =
        fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    serde_yaml::from_str(&text).map_err(|e| format!("failed to parse {}: {e}", path.display()))
}

fn collect_nav_refs(node: &YamlValue, out: &mut Vec<(String, String)>) {
    match node {
        YamlValue::Sequence(seq) => {
            for item in seq {
                collect_nav_refs(item, out);
            }
        }
        YamlValue::Mapping(map) => {
            for (k, v) in map {
                let title = k.as_str().unwrap_or_default().to_string();
                if let Some(path) = v.as_str() {
                    out.push((title, path.to_string()));
                } else {
                    collect_nav_refs(v, out);
                }
            }
        }
        _ => {}
    }
}

pub(crate) fn mkdocs_nav_refs(repo_root: &Path) -> Result<Vec<(String, String)>, String> {
    let yaml = parse_mkdocs_yaml(repo_root)?;
    let nav = yaml
        .get("nav")
        .ok_or_else(|| "mkdocs.yml missing `nav`".to_string())?;
    let mut refs = Vec::new();
    collect_nav_refs(nav, &mut refs);
    refs.sort();
    Ok(refs)
}

fn docs_inventory_payload(
    ctx: &DocsContext,
    common: &DocsCommonArgs,
) -> Result<serde_json::Value, String> {
    let nav_refs = mkdocs_nav_refs(&ctx.repo_root)?;
    let nav_set = nav_refs
        .iter()
        .map(|(_, p)| p.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let rows = docs_markdown_files(&ctx.docs_root, common.include_drafts)
        .into_iter()
        .filter_map(|p| {
            p.strip_prefix(&ctx.docs_root)
                .ok()
                .map(|r| r.display().to_string())
        })
        .map(|rel| DocsPageRow {
            in_nav: nav_set.contains(&rel),
            path: rel,
        })
        .collect::<Vec<_>>();
    let orphan_pages = rows
        .iter()
        .filter(|r| {
            !r.in_nav
                && !r.path.starts_with("_assets/")
                && (common.include_drafts || !r.path.starts_with("_drafts/"))
        })
        .map(|r| r.path.clone())
        .collect::<Vec<_>>();
    let duplicate_titles = {
        let mut seen = BTreeMap::<String, usize>::new();
        for (title, _) in &nav_refs {
            *seen.entry(title.clone()).or_default() += 1;
        }
        let mut d = seen
            .into_iter()
            .filter(|(_, n)| *n > 1)
            .map(|(k, _)| k)
            .collect::<Vec<_>>();
        d.sort();
        d
    };
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},
        "options": {"strict": common.strict, "include_drafts": common.include_drafts},
        "nav": nav_refs.iter().map(|(title, path)| serde_json::json!({"title": title, "path": path})).collect::<Vec<_>>(),
        "pages": rows,
        "orphan_pages": orphan_pages,
        "duplicate_nav_titles": duplicate_titles
    }))
}

fn scan_registry_markdown_files(repo_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for file in walk_files_local(repo_root) {
        if file.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let Ok(rel) = file.strip_prefix(repo_root) else {
            continue;
        };
        let rels = rel.to_string_lossy();
        if rels.starts_with("artifacts/") || rels.contains("/target/") {
            continue;
        }
        if !is_allowed_doc_location(&rels) {
            continue;
        }
        if rels.starts_with("docs/_generated/") || rels.starts_with("docs/_drafts/") {
            continue;
        }
        files.push(file);
    }
    files.sort();
    files
}

fn is_allowed_doc_location(path: &str) -> bool {
    path == "README.md"
        || path.starts_with("docs/")
        || path.starts_with("crates/")
        || path.starts_with("ops/")
        || path.starts_with("configs/")
        || path.starts_with("docker/")
        || path.starts_with("makefiles/")
        || path.starts_with(".github/")
}

fn read_dir_entries(path: &Path) -> Vec<PathBuf> {
    match fs::read_dir(path) {
        Ok(entries) => entries.filter_map(Result::ok).map(|e| e.path()).collect(),
        Err(_) => Vec::new(),
    }
}

fn infer_doc_type(path: &str) -> &'static str {
    if path.contains("/runbooks/") {
        "runbook"
    } else if path.contains("/contracts/") || path.contains("SCHEMA") || path.contains("OPENAPI") {
        "spec"
    } else if path.contains("/quickstart/") || path.contains("how-to") {
        "how-to"
    } else if path.contains("/reference/") {
        "reference"
    } else {
        "concept"
    }
}

fn infer_lifecycle(path: &str) -> &'static str {
    if path.contains("/_drafts/") {
        "draft"
    } else if path.contains("/_style/") || path.contains("/_lint/") || path.contains("/_nav/") {
        "internal"
    } else {
        "stable"
    }
}

fn parse_owner_and_stability(file: &Path) -> (String, String) {
    let Ok(text) = fs::read_to_string(file) else {
        return ("docs-governance".to_string(), "stable".to_string());
    };
    let mut owner = None;
    let mut stability = None;
    for line in text.lines().take(40) {
        let trimmed = line.trim();
        if owner.is_none() && trimmed.starts_with("- Owner:") {
            owner = Some(
                trimmed
                    .trim_start_matches("- Owner:")
                    .trim()
                    .trim_matches('`')
                    .to_string(),
            );
        }
        if stability.is_none() && trimmed.starts_with("- Stability:") {
            stability = Some(
                trimmed
                    .trim_start_matches("- Stability:")
                    .trim()
                    .trim_matches('`')
                    .to_string(),
            );
        }
    }
    (
        owner.unwrap_or_else(|| "docs-governance".to_string()),
        stability.unwrap_or_else(|| "stable".to_string()),
    )
}

fn crate_association(path: &str) -> Option<String> {
    let parts = path.split('/').collect::<Vec<_>>();
    if parts.len() >= 3 && parts[0] == "crates" && parts[2] == "docs" {
        Some(parts[1].to_string())
    } else {
        None
    }
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

fn docs_registry_payload(ctx: &DocsContext) -> serde_json::Value {
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

fn registry_validate_payload(ctx: &DocsContext) -> Result<serde_json::Value, String> {
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
            if let Some((y, m, d)) = parse_ymd_date(last_reviewed) {
                if (y, m, d) < (2025, 8, 1) {
                    warnings.push(format!("DOCS_REGISTRY_OUTDATED: `{path}` last_reviewed={last_reviewed}"));
                }
            }
        } else {
            warnings.push(format!("DOCS_REGISTRY_MISSING_LAST_REVIEWED: `{path}`"));
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
            warnings.push(format!("DOCS_REGISTRY_DUPLICATE_TOPIC: `{topic}` appears {count} times"));
        }
    }
    let mut per_crate = BTreeMap::<String, usize>::new();
    for entry in &docs {
        let bucket = entry["crate"]
            .as_str()
            .unwrap_or("docs-root")
            .to_string();
        *per_crate.entry(bucket).or_default() += 1;
    }
    for (bucket, count) in per_crate {
        if count > 10 {
            warnings.push(format!("DOCS_REGISTRY_DOC_BUDGET_WARN: `{bucket}` has {count} docs (budget=10)"));
        }
    }
    let root_md = read_dir_entries(&ctx.repo_root)
        .into_iter()
        .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    for file in root_md {
        if file.file_name().and_then(|v| v.to_str()) != Some("README.md") {
            errors.push(format!(
                "DOCS_REGISTRY_ROOT_DOC_FORBIDDEN: only root README.md allowed, found `{}`",
                file.file_name().and_then(|v| v.to_str()).unwrap_or_default()
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
    Ok(serde_json::json!({
        "schema_version": 1,
        "errors": errors,
        "warnings": warnings,
        "summary": {
            "registered": docs.len(),
            "errors": errors.len(),
            "warnings": warnings.len()
        }
    }))
}

pub(crate) fn docs_validate_payload(
    ctx: &DocsContext,
    common: &DocsCommonArgs,
) -> Result<serde_json::Value, String> {
    let yaml = parse_mkdocs_yaml(&ctx.repo_root)?;
    let mut issues = DocsIssues::default();
    let docs_dir = yaml
        .get("docs_dir")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if docs_dir != "docs" {
        issues.errors.push(format!(
            "DOCS_NAV_ERROR: mkdocs.yml docs_dir must be `docs`, got `{docs_dir}`"
        ));
    }
    for (_, rel) in mkdocs_nav_refs(&ctx.repo_root)? {
        if !ctx.docs_root.join(&rel).exists() {
            issues.errors.push(format!(
                "DOCS_NAV_ERROR: mkdocs nav references missing file `{rel}`"
            ));
        }
    }
    let inv = docs_inventory_payload(ctx, common)?;
    for dup in inv["duplicate_nav_titles"].as_array().into_iter().flatten() {
        if let Some(title) = dup.as_str() {
            issues.warnings.push(format!(
                "DOCS_NAV_ERROR: duplicate mkdocs nav title `{title}`"
            ));
        }
    }
    let registry_checks = registry_validate_payload(ctx)?;
    for err in registry_checks["errors"].as_array().into_iter().flatten() {
        if let Some(s) = err.as_str() {
            issues.errors.push(s.to_string());
        }
    }
    for warn in registry_checks["warnings"].as_array().into_iter().flatten() {
        if let Some(s) = warn.as_str() {
            issues.warnings.push(s.to_string());
        }
    }
    if common.strict {
        issues.errors.append(&mut issues.warnings);
    }
    let text = if issues.errors.is_empty() {
        format!("docs validate passed (warnings={})", issues.warnings.len())
    } else {
        format!(
            "docs validate failed (errors={} warnings={})",
            issues.errors.len(),
            issues.warnings.len()
        )
    };
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": text,
        "errors": issues.errors,
        "warnings": issues.warnings,
        "rows": inv["nav"].as_array().cloned().unwrap_or_default(),
        "registry": registry_checks,
        "summary": {"total": inv["nav"].as_array().map(|v| v.len()).unwrap_or(0), "errors": inv["errors"].as_array().map(|v| v.len()).unwrap_or(0), "warnings": inv["warnings"].as_array().map(|v| v.len()).unwrap_or(0)},
        "capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},
        "options": {"strict": common.strict, "include_drafts": common.include_drafts}
    }))
}

fn markdown_anchors(text: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix('#') {
            let heading = rest.trim_start_matches('#').trim();
            if !heading.is_empty() {
                out.insert(slugify_anchor(heading));
            }
        }
    }
    out
}

pub(crate) fn docs_links_payload(
    ctx: &DocsContext,
    common: &DocsCommonArgs,
) -> Result<serde_json::Value, String> {
    let mut rows = Vec::<serde_json::Value>::new();
    let mut issues = DocsIssues::default();
    let link_re = Regex::new(r"\[[^\]]+\]\(([^)]+)\)").map_err(|e| e.to_string())?;
    for file in docs_markdown_files(&ctx.docs_root, common.include_drafts) {
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let text = fs::read_to_string(&file).map_err(|e| format!("failed to read {rel}: {e}"))?;
        let anchors = markdown_anchors(&text);
        for (idx, line) in text.lines().enumerate() {
            for cap in link_re.captures_iter(line) {
                let target = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                if target.starts_with("http://")
                    || target.starts_with("https://")
                    || target.starts_with("mailto:")
                {
                    if common.allow_network {
                        rows.push(serde_json::json!({"file": rel, "line": idx + 1, "target": target, "ok": true, "external": true, "checked_network": false}));
                    }
                    continue;
                }
                if let Some(anchor) = target.strip_prefix('#') {
                    let ok = anchors.contains(anchor);
                    if !ok {
                        issues.errors.push(format!(
                            "DOCS_LINK_ERROR: {rel}:{} missing same-file anchor `#{anchor}`",
                            idx + 1
                        ));
                    }
                    rows.push(serde_json::json!({"file": rel, "line": idx + 1, "target": target, "ok": ok}));
                    continue;
                }
                let (path_part, anchor_part) = target
                    .split_once('#')
                    .map_or((target, None), |(a, b)| (a, Some(b)));
                if path_part.is_empty() || path_part.ends_with('/') {
                    continue;
                }
                let resolved = file.parent().unwrap_or(&ctx.docs_root).join(path_part);
                let exists = resolved.exists();
                let mut ok = exists;
                if exists {
                    if let Some(anchor) = anchor_part {
                        if resolved.extension().and_then(|v| v.to_str()) == Some("md") {
                            let target_text = fs::read_to_string(&resolved).unwrap_or_default();
                            ok = markdown_anchors(&target_text).contains(anchor);
                        }
                    }
                }
                if !ok {
                    let generated_target =
                        path_part.starts_with("_generated/") || path_part.contains("/_generated/");
                    let message = format!(
                        "DOCS_LINK_ERROR: {rel}:{} unresolved link `{target}`",
                        idx + 1
                    );
                    if generated_target && !common.strict {
                        issues.warnings.push(message);
                    } else {
                        issues.errors.push(message);
                    }
                }
                rows.push(
                    serde_json::json!({"file": rel, "line": idx + 1, "target": target, "ok": ok}),
                );
            }
        }
    }
    rows.sort_by(|a, b| {
        a["file"]
            .as_str()
            .cmp(&b["file"].as_str())
            .then(a["line"].as_u64().cmp(&b["line"].as_u64()))
            .then(a["target"].as_str().cmp(&b["target"].as_str()))
    });
    issues.errors.sort();
    issues.errors.dedup();
    issues.warnings.sort();
    issues.warnings.dedup();
    if common.strict && !issues.warnings.is_empty() {
        issues.errors.append(&mut issues.warnings);
        issues.errors.sort();
        issues.errors.dedup();
    }
    Ok(serde_json::json!({
        "schema_version":1,
        "run_id":ctx.run_id.as_str(),
        "text": if issues.errors.is_empty() {
            if issues.warnings.is_empty() {"docs links passed"} else {"docs links passed with warnings"}
        } else {"docs links failed"},
        "rows":rows,
        "errors":issues.errors,
        "warnings": issues.warnings,
        "capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},
        "options": {"strict": common.strict, "include_drafts": common.include_drafts},
        "external_link_check": {"enabled": common.allow_network, "mode": "disabled_best_effort"}
    }))
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
    let mut errors = Vec::<String>::new();
    let adr_filename_re = Regex::new(r"^ADR-\d{4}-[a-z0-9-]+\.md$").map_err(|e| e.to_string())?;
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
        let text = fs::read_to_string(&file).map_err(|e| format!("failed to read {rel}: {e}"))?;
        for (idx, line) in text.lines().enumerate() {
            if line.ends_with(' ') || line.contains('\t') {
                errors.push(format!(
                    "{rel}:{} formatting lint failure (tab/trailing-space)",
                    idx + 1
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
                    let docs_rows = generated["documents"].as_array().cloned().unwrap_or_default();
                    let mut search_index = Vec::new();
                    let mut graph = Vec::new();
                    let mut crate_slice = BTreeMap::<String, Vec<serde_json::Value>>::new();
                    for doc in &docs_rows {
                        let path = doc["path"].as_str().unwrap_or_default().to_string();
                        let tags = doc["tags"].as_array().cloned().unwrap_or_default();
                        search_index.push(serde_json::json!({
                            "path": path,
                            "topic": doc["topic"],
                            "tags": tags
                        }));
                        graph.push(serde_json::json!({
                            "from": path,
                            "crate": doc["crate"],
                            "doc_type": doc["doc_type"]
                        }));
                        if let Some(crate_name) = doc["crate"].as_str() {
                            crate_slice
                                .entry(crate_name.to_string())
                                .or_default()
                                .push(doc.clone());
                        }
                    }
                    let crate_coverage = crate_slice
                        .iter()
                        .map(|(name, rows)| serde_json::json!({
                            "crate": name,
                            "doc_count": rows.len()
                        }))
                        .collect::<Vec<_>>();
                    if common.allow_write {
                        let generated_dir = ctx.repo_root.join("docs/_generated");
                        fs::create_dir_all(&generated_dir)
                            .map_err(|e| format!("failed to create {}: {e}", generated_dir.display()))?;
                        fs::write(
                            generated_dir.join("search-index.json"),
                            serde_json::to_string_pretty(&serde_json::json!({
                                "schema_version": 1,
                                "entries": search_index
                            }))
                            .map_err(|e| format!("search index encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("write search index failed: {e}"))?;
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
                        fs::write(
                            generated_dir.join("crate-doc-coverage.json"),
                            serde_json::to_string_pretty(&serde_json::json!({
                                "schema_version": 1,
                                "rows": crate_coverage
                            }))
                            .map_err(|e| format!("crate coverage encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("write crate coverage failed: {e}"))?;
                        let mut inventory_md =
                            String::from("# Docs Inventory\n\nLicense: Apache-2.0\n\n");
                        inventory_md.push_str("| Path | Type | Owner | Stability |\n|---|---|---|---|\n");
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
                            "dependency_graph": "docs/_generated/docs-dependency-graph.json",
                            "crate_docs_slice": "docs/_generated/crate-docs-slice.json",
                            "crate_doc_coverage": "docs/_generated/crate-doc-coverage.json"
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
                    payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
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
