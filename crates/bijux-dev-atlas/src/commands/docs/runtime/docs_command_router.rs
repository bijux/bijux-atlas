use std::io::{self, Write};

fn current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn docs_gate_artifact_dir(ctx: &DocsContext) -> std::path::PathBuf {
    ctx.artifacts_root
        .join("run")
        .join(ctx.run_id.as_str())
        .join("gates")
        .join("docs")
}

fn write_docs_gate_artifact(
    path: &std::path::Path,
    payload: &serde_json::Value,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(payload).map_err(|e| format!("encode failed: {e}"))?,
    )
    .map_err(|e| format!("write {} failed: {e}", path.display()))
}

#[derive(serde::Deserialize)]
struct DocsConceptRegistry {
    concepts: Vec<DocsConceptRow>,
}

#[derive(serde::Deserialize)]
struct DocsConceptRow {
    id: String,
    canonical: String,
    #[serde(default)]
    pointers: Vec<String>,
}

fn load_docs_concepts(repo_root: &std::path::Path) -> Result<Vec<DocsConceptRow>, String> {
    let path = repo_root.join("docs/_internal/style/concepts.yml");
    let text =
        fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let payload: DocsConceptRegistry = serde_yaml::from_str(&text)
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;
    Ok(payload.concepts)
}

fn render_concept_registry_markdown(rows: &[DocsConceptRow]) -> String {
    let mut out = String::from("# Concept Registry\n\n");
    out.push_str("Generated from `docs/_internal/style/concepts.yml`.\n\n");
    out.push_str("| Concept ID | Canonical Page | Pointer Pages |\n|---|---|---|\n");
    for row in rows {
        let pointers = if row.pointers.is_empty() {
            "`none`".to_string()
        } else {
            row.pointers
                .iter()
                .map(|pointer| format!("`{pointer}`"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        out.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            row.id, row.canonical, pointers
        ));
    }
    out
}

include!("docs_command_router_registry.inc.rs");

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
                let mut payload = serde_json::json!({
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
                if common.allow_write {
                    let out_dir = docs_gate_artifact_dir(&ctx);
                    let check_path = out_dir.join("check.json");
                    let validate_path = out_dir.join("validate.json");
                    let links_path = out_dir.join("links.json");
                    let lint_path = out_dir.join("lint.json");
                    let build_path = out_dir.join("build.json");
                    let meta_path = out_dir.join("meta.json");
                    let summary_path = out_dir.join("summary.json");
                    let meta = serde_json::json!({
                        "schema_version": 1,
                        "gate": "docs",
                        "command": "bijux dev atlas docs check",
                        "run_id": ctx.run_id.as_str(),
                        "status": if errors == 0 { "pass" } else { "fail" },
                        "timestamp_unix": current_unix_timestamp(),
                    });
                    let summary = serde_json::json!({
                        "schema_version": 1,
                        "gate": "docs",
                        "run_id": ctx.run_id.as_str(),
                        "status": if errors == 0 { "pass" } else { "fail" },
                        "counts": {
                            "errors": errors,
                            "validate_errors": validate["errors"].as_array().map(|v| v.len()).unwrap_or(0),
                            "links_errors": links["errors"].as_array().map(|v| v.len()).unwrap_or(0),
                            "lint_errors": lint["errors"].as_array().map(|v| v.len()).unwrap_or(0),
                            "build_exit_code": build_code
                        }
                    });
                    write_docs_gate_artifact(&validate_path, &validate)?;
                    write_docs_gate_artifact(&links_path, &links)?;
                    write_docs_gate_artifact(&lint_path, &lint)?;
                    write_docs_gate_artifact(&build_path, &build_payload)?;
                    write_docs_gate_artifact(&meta_path, &meta)?;
                    write_docs_gate_artifact(&summary_path, &summary)?;
                    write_docs_gate_artifact(&check_path, &payload)?;
                    payload["artifacts"] = serde_json::json!({
                        "check": check_path.display().to_string(),
                        "validate": validate_path.display().to_string(),
                        "links": links_path.display().to_string(),
                        "lint": lint_path.display().to_string(),
                        "build": build_path.display().to_string(),
                        "meta": meta_path.display().to_string(),
                        "summary": summary_path.display().to_string()
                    });
                }
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
            DocsCommand::ShrinkReport(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_shrink_report_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                if common.allow_write {
                    let report_path = ctx
                        .repo_root
                        .join("ops/_generated.example/docs-shrink-report.json");
                    if let Some(parent) = report_path.parent() {
                        fs::create_dir_all(parent).map_err(|e| {
                            format!("failed to create {}: {e}", parent.display())
                        })?;
                    }
                    fs::write(
                        &report_path,
                        serde_json::to_string_pretty(&payload)
                            .map_err(|e| format!("encode docs shrink report failed: {e}"))?,
                    )
                    .map_err(|e| format!("write {} failed: {e}", report_path.display()))?;
                }
                let code = if payload["status"] == "pass" { 0 } else { 1 };
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::Reference { command } => match command {
                crate::cli::DocsReferenceCommand::Generate(common) => {
                    let ctx = docs_context(&common)?;
                    let (changed, generated) = docs_reference_generate_or_check(&ctx.repo_root, true)?;
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "run_id": ctx.run_id.as_str(),
                        "status": "ok",
                        "text": "docs reference pages generated",
                        "changed": changed,
                        "generated": generated,
                        "duration_ms": started.elapsed().as_millis() as u64,
                    });
                    Ok((emit_payload(common.format, common.out, &payload)?, 0))
                }
                crate::cli::DocsReferenceCommand::Check(common) => {
                    let ctx = docs_context(&common)?;
                    let (changed, generated) = docs_reference_generate_or_check(&ctx.repo_root, false)?;
                    let code = if changed.is_empty() { 0 } else { 1 };
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "run_id": ctx.run_id.as_str(),
                        "status": if code == 0 { "ok" } else { "drift" },
                        "text": if code == 0 { "docs reference pages up to date" } else { "docs reference pages drift detected" },
                        "changed": changed,
                        "generated": generated,
                        "duration_ms": started.elapsed().as_millis() as u64,
                    });
                    Ok((emit_payload(common.format, common.out, &payload)?, code))
                }
            },
            DocsCommand::Registry { command } => run_docs_registry_command(&started, command),
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
            DocsCommand::ExternalLinks(args) => {
                let ctx = docs_context(&args.common)?;
                let mut payload =
                    docs_external_links_payload(&ctx, &args.common, &args.allowlist)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                if code != 0 {
                    payload["error_code"] = serde_json::json!("DOCS_EXTERNAL_LINK_ERROR");
                }
                Ok((emit_payload(args.common.format, args.common.out, &payload)?, code))
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
                    let _ = writeln!(io::stdout(), "{rendered}");
                } else {
                    let _ = writeln!(io::stderr(), "{rendered}");
                }
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas docs failed: {err}");
            1
        }
    }
}

fn docs_shrink_report_payload(
    ctx: &DocsContext,
    common: &DocsCommonArgs,
) -> Result<serde_json::Value, String> {
    let inventory = docs_inventory_payload(ctx, common)?;
    let mut counts = BTreeMap::<String, usize>::new();
    for row in inventory["rows"].as_array().into_iter().flatten() {
        if let Some(path) = row["path"].as_str() {
            let p = std::path::Path::new(path);
            if p.components().next().and_then(|c| c.as_os_str().to_str()) == Some("docs") {
                let mut it = p.components();
                let _ = it.next();
                if let Some(dir) = it.next().and_then(|c| c.as_os_str().to_str()) {
                    if dir != "_generated" && dir != "_drafts" {
                        *counts.entry(dir.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    let mut top_directories = counts
        .iter()
        .map(|(dir, count)| serde_json::json!({"directory": dir, "markdown_count": count}))
        .collect::<Vec<_>>();
    top_directories.sort_by(|a, b| {
        b["markdown_count"]
            .as_u64()
            .cmp(&a["markdown_count"].as_u64())
            .then(a["directory"].as_str().cmp(&b["directory"].as_str()))
    });
    let max_md_per_dir = counts.values().copied().max().unwrap_or(0);
    let status = if max_md_per_dir > 40 { "fail" } else { "pass" };
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "status": status,
        "max_md_per_dir": max_md_per_dir,
        "top_directories": top_directories,
        "budget": { "max_md_per_dir": 40 },
        "text": if status == "pass" { "docs shrink report passed" } else { "docs shrink report failed" }
    }))
}
