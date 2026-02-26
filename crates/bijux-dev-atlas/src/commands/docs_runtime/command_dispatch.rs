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

fn docs_reference_generate_or_check(
    repo_root: &std::path::Path,
    allow_write: bool,
) -> Result<(Vec<String>, Vec<String>), String> {
    let ref_dir = repo_root.join("docs/operations/reference");
    let mut changed = Vec::<String>::new();
    let mut generated = Vec::<String>::new();
    let targets = docs_reference_target_contents(repo_root)?;
    for (rel, content) in targets {
        let path = repo_root.join(rel);
        generated.push(rel.to_string());
        let existing = std::fs::read_to_string(&path)
            .map_err(|e| format!("read {} failed: {e}", path.display()))?;
        if existing != content {
            changed.push(rel.to_string());
            if allow_write {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("create {} failed: {e}", parent.display()))?;
                }
                std::fs::write(&path, content)
                    .map_err(|e| format!("write {} failed: {e}", path.display()))?;
            }
        }
    }
    let _ = ref_dir;
    Ok((changed, generated))
}

fn docs_reference_target_contents(
    repo_root: &std::path::Path,
) -> Result<Vec<(&'static str, String)>, String> {
    Ok(vec![
        (
            "docs/operations/reference/commands.md",
            render_docs_reference_commands(repo_root)?,
        ),
        (
            "docs/operations/reference/ops-surface.md",
            render_docs_reference_ops_surface(repo_root)?,
        ),
        (
            "docs/operations/reference/tools.md",
            render_docs_reference_tools(repo_root)?,
        ),
        (
            "docs/operations/reference/toolchain.md",
            render_docs_reference_toolchain(repo_root)?,
        ),
        (
            "docs/operations/reference/pins.md",
            render_docs_reference_pins(repo_root)?,
        ),
        (
            "docs/operations/reference/gates.md",
            render_docs_reference_gates(repo_root)?,
        ),
        (
            "docs/operations/reference/drills.md",
            render_docs_reference_drills(repo_root)?,
        ),
        (
            "docs/operations/reference/schema-index.md",
            render_docs_reference_schema_index(),
        ),
        (
            "docs/operations/reference/evidence-model.md",
            render_docs_reference_evidence_model(repo_root)?,
        ),
        (
            "docs/operations/reference/what-breaks-if-removed.md",
            render_docs_reference_what_breaks(repo_root)?,
        ),
    ])
}

fn run_bijux_dev_atlas_help(repo_root: &std::path::Path, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("cargo")
        .current_dir(repo_root)
        .args(["run", "-q", "-p", "bijux-dev-atlas", "--"])
        .args(args)
        .output()
        .map_err(|e| format!("spawn cargo for docs reference help failed: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo help command failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8(output.stdout)
        .map_err(|e| format!("invalid utf8 in help output: {e}"))?
        .trim_end()
        .to_string())
}

fn trim_help_usage_and_commands(help: &str) -> String {
    let mut out = Vec::new();
    for line in help.lines() {
        if line.starts_with("Options:") {
            break;
        }
        out.push(line.trim_end());
    }
    while out.last().is_some_and(|line| line.is_empty()) {
        out.pop();
    }
    out.join("\n")
}

fn render_docs_reference_commands(repo_root: &std::path::Path) -> Result<String, String> {
    let root_help = trim_help_usage_and_commands(&run_bijux_dev_atlas_help(repo_root, &["--help"])?);
    let ops_help = trim_help_usage_and_commands(&run_bijux_dev_atlas_help(repo_root, &["ops", "--help"])?);
    Ok(format!(
        "# Command Surface Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `bijux dev atlas --help`, `bijux dev atlas ops --help`, `makefiles/GENERATED_TARGETS.md`\n\n## Purpose\n\nGenerated reference for the supported command surface. Narrative docs should link here instead of restating command lists.\n\n## bijux-dev-atlas\n\n```text\n{root_help}\n```\n\n## bijux-dev-atlas ops\n\n```text\n{ops_help}\n```\n\n## Make Wrapper Surface\n\nSee `makefiles/GENERATED_TARGETS.md` and generated ops surface references. Narrative docs must not duplicate long `make ops-*` command lists.\n\n## Regenerate\n\n- `bijux dev atlas docs reference generate --allow-subprocess --allow-write`\n"
    ))
}

fn render_docs_reference_ops_surface(repo_root: &std::path::Path) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/inventory/surfaces.json"))
            .map_err(|e| format!("read surfaces.json failed: {e}"))?,
    )
    .map_err(|e| format!("parse surfaces.json failed: {e}"))?;
    let mut entrypoints = value["entrypoints"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect::<Vec<_>>();
    entrypoints.sort();
    let mut commands = value["bijux-dev-atlas_commands"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect::<Vec<_>>();
    commands.sort();
    let mut actions = value["actions"].as_array().cloned().unwrap_or_default();
    actions.sort_by_key(|row| row["id"].as_str().unwrap_or_default().to_string());
    let mut out = String::new();
    out.push_str("# Ops Surface Reference\n\n");
    out.push_str("- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/inventory/surfaces.json`, `ops/_generated.example/control-plane.snapshot.md`\n\n");
    out.push_str("## Purpose\n\nGenerated ops surface reference derived from inventory surfaces.\n\n");
    out.push_str("## Entry Points\n\n");
    for item in entrypoints {
        out.push_str(&format!("- `{item}`\n"));
    }
    out.push_str("\n## bijux-dev-atlas Commands\n\n");
    for item in commands {
        out.push_str(&format!("- `{item}`\n"));
    }
    out.push_str("\n## Actions\n\n");
    for item in actions {
        let encoded = serde_json::to_string(&item)
            .map_err(|e| format!("encode action row for ops surface reference failed: {e}"))?;
        out.push_str(&format!("- `{encoded}`\n"));
    }
    out.push_str("\n## See Also\n\n- `ops/_generated.example/control-plane.snapshot.md` (example generated snapshot)\n- `ops/inventory/surfaces.json` (machine truth)\n");
    Ok(out)
}

fn render_docs_reference_tools(repo_root: &std::path::Path) -> Result<String, String> {
    let text = std::fs::read_to_string(repo_root.join("ops/inventory/tools.toml"))
        .map_err(|e| format!("read tools.toml failed: {e}"))?;
    let value: toml::Value = toml::from_str(&text).map_err(|e| format!("parse tools.toml failed: {e}"))?;
    let mut rows = value["tools"].as_array().cloned().unwrap_or_default();
    rows.sort_by_key(|row| row["name"].as_str().unwrap_or_default().to_string());
    let mut out = String::new();
    out.push_str("# Tools Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/inventory/tools.toml`\n\n## Tools\n\n| Tool | Required | Probe Args | Version Regex |\n| --- | --- | --- | --- |\n");
    for row in rows {
        let probe = row["probe_argv"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` |\n",
            row["name"].as_str().unwrap_or_default(),
            row["required"].as_bool().unwrap_or(false),
            probe,
            row["version_regex"].as_str().unwrap_or_default()
        ));
    }
    out.push_str("\n## Regenerate\n\n- `bijux dev atlas docs reference generate --allow-subprocess --allow-write`\n");
    Ok(out)
}

fn render_docs_reference_toolchain(repo_root: &std::path::Path) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/inventory/toolchain.json"))
            .map_err(|e| format!("read toolchain.json failed: {e}"))?,
    )
    .map_err(|e| format!("parse toolchain.json failed: {e}"))?;
    let mut out = String::new();
    out.push_str("# Toolchain Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/inventory/toolchain.json`\n\n## Tools\n\n| Tool | Required | Probe Args |\n| --- | --- | --- |\n");
    let mut tools = value["tools"]
        .as_object()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>();
    tools.sort_by_key(|(k, _)| k.clone());
    for (name, row) in tools {
        let probe = row["probe_argv"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` |\n",
            name,
            row["required"].as_bool().unwrap_or(false),
            probe
        ));
    }
    out.push_str("\n## Images\n\n| Image Key | Reference |\n| --- | --- |\n");
    let mut images = value["images"]
        .as_object()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>();
    images.sort_by_key(|(k, _)| k.clone());
    for (k, v) in images {
        out.push_str(&format!("| `{}` | `{}` |\n", k, v.as_str().unwrap_or_default()));
    }
    out.push_str("\n## GitHub Actions Pins\n\n| Action | Ref | SHA |\n| --- | --- | --- |\n");
    let mut actions = value["github_actions"]
        .as_object()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>();
    actions.sort_by_key(|(k, _)| k.clone());
    for (action, row) in actions {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` |\n",
            action,
            row["ref"].as_str().unwrap_or_default(),
            row["sha"].as_str().unwrap_or_default()
        ));
    }
    Ok(out)
}

fn render_docs_reference_pins(repo_root: &std::path::Path) -> Result<String, String> {
    let value: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(repo_root.join("ops/inventory/pins.yaml"))
            .map_err(|e| format!("read pins.yaml failed: {e}"))?,
    )
    .map_err(|e| format!("parse pins.yaml failed: {e}"))?;
    let mut rows = Vec::<(String, String, String)>::new();
    collect_yaml_rows("root", &value, &mut rows);
    rows.sort();
    let mut out = String::new();
    out.push_str("# Pins Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/inventory/pins.yaml`\n\n## Pins\n\n| Section | Key | Value |\n| --- | --- | --- |\n");
    for (section, key, val) in rows {
        out.push_str(&format!("| `{}` | `{}` | `{}` |\n", section, key, val));
    }
    Ok(out)
}

fn collect_yaml_rows(prefix: &str, value: &serde_yaml::Value, out: &mut Vec<(String, String, String)>) {
    if let serde_yaml::Value::Mapping(map) = value {
        for (k, v) in map {
            let key = k.as_str().unwrap_or_default();
            if prefix == "root" && !matches!(v, serde_yaml::Value::Mapping(_) | serde_yaml::Value::Sequence(_)) {
                out.push((prefix.to_string(), key.to_string(), yaml_scalar_string(v)));
            } else {
                match v {
                    serde_yaml::Value::Mapping(inner) => {
                        for (ik, iv) in inner {
                            out.push((
                                key.to_string(),
                                ik.as_str().unwrap_or_default().to_string(),
                                yaml_scalar_string(iv),
                            ));
                        }
                    }
                    serde_yaml::Value::Sequence(seq) => {
                        for (idx, item) in seq.iter().enumerate() {
                            out.push((key.to_string(), idx.to_string(), yaml_scalar_string(item)));
                        }
                    }
                    _ => out.push((prefix.to_string(), key.to_string(), yaml_scalar_string(v))),
                }
            }
        }
    }
}

fn yaml_scalar_string(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::Null => "null".to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::String(s) => s.clone(),
        _ => serde_yaml::to_string(v).unwrap_or_default().trim().to_string(),
    }
}

fn render_docs_reference_gates(repo_root: &std::path::Path) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/inventory/gates.json"))
            .map_err(|e| format!("read gates.json failed: {e}"))?,
    )
    .map_err(|e| format!("parse gates.json failed: {e}"))?;
    let mut gates = value["gates"].as_array().cloned().unwrap_or_default();
    gates.sort_by_key(|g| g["id"].as_str().unwrap_or_default().to_string());
    let mut out = String::new();
    out.push_str("# Gates Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/inventory/gates.json`\n\n## Gates\n\n| Gate ID | Category | Action ID | Description |\n| --- | --- | --- | --- |\n");
    for gate in gates {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} |\n",
            gate["id"].as_str().unwrap_or_default(),
            gate["category"].as_str().unwrap_or_default(),
            gate["action_id"].as_str().unwrap_or_default(),
            gate["description"].as_str().unwrap_or_default()
        ));
    }
    Ok(out)
}

fn render_docs_reference_drills(repo_root: &std::path::Path) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/inventory/drills.json"))
            .map_err(|e| format!("read drills.json failed: {e}"))?,
    )
    .map_err(|e| format!("parse drills.json failed: {e}"))?;
    let mut drills = value["drills"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|v| v.as_str().map(ToString::to_string).unwrap_or_else(|| v["id"].as_str().unwrap_or_default().to_string()))
        .collect::<Vec<_>>();
    drills.sort();
    let mut out = String::new();
    out.push_str("# Drills Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/inventory/drills.json`\n\n## Drills\n\n");
    for drill in drills {
        out.push_str(&format!("- `{drill}`\n"));
    }
    Ok(out)
}

fn render_docs_reference_schema_index() -> String {
    "# Schema Index Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/schema/generated/schema-index.md`\n\n## Canonical Source\n\n- `ops/schema/generated/schema-index.md` is the authoritative generated schema index.\n- This page is a docs-site reference pointer to avoid duplicating the schema table.\n".to_string()
}

fn render_docs_reference_evidence_model(repo_root: &std::path::Path) -> Result<String, String> {
    let levels: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/schema/report/evidence-levels.schema.json"))
            .map_err(|e| format!("read evidence-levels schema failed: {e}"))?,
    )
    .map_err(|e| format!("parse evidence-levels schema failed: {e}"))?;
    let _bundle: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/schema/report/release-evidence-bundle.schema.json"))
            .map_err(|e| format!("read release-evidence-bundle schema failed: {e}"))?,
    )
    .map_err(|e| format!("parse release-evidence-bundle schema failed: {e}"))?;
    Ok(format!(
        "# Evidence Model Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/schema/report/evidence-levels.schema.json`, `ops/schema/report/release-evidence-bundle.schema.json`\n\n## Canonical Schemas\n\n- `ops/schema/report/evidence-levels.schema.json`\n- `ops/schema/report/release-evidence-bundle.schema.json`\n\n## Notes\n\n- evidence-levels schema title: `{}`\n",
        levels["title"].as_str().unwrap_or_default()
    ))
}

fn render_docs_reference_what_breaks(repo_root: &std::path::Path) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/_generated.example/what-breaks-if-removed-report.json"))
            .map_err(|e| format!("read what-breaks-if-removed report failed: {e}"))?,
    )
    .map_err(|e| format!("parse what-breaks-if-removed report failed: {e}"))?;
    let mut out = String::new();
    out.push_str("# What Breaks If Removed Reference\n\n- Owner: `bijux-atlas-operations`\n- Tier: `generated`\n- Audience: `operators`\n- Source-of-truth: `ops/_generated.example/what-breaks-if-removed-report.json`\n\n## Removal Impact Targets\n\n| Path | Impact | Consumers |\n| --- | --- | --- |\n");
    for row in value["targets"].as_array().into_iter().flatten() {
        let consumers = row["consumers"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` |\n",
            row["path"].as_str().unwrap_or_default(),
            row["impact"].as_str().unwrap_or_default(),
            consumers
        ));
    }
    Ok(out)
}
