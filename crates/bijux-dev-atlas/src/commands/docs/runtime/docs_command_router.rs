use std::io::{self, Write};

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
                        let front_matter_index = serde_json::json!({
                            "schema_version": "v1",
                            "description": "Canonical ownership, stability, title, and audience metadata registry for documentation pages",
                            "source": "docs/registry.json",
                            "documents": docs_rows.iter().map(|row| serde_json::json!({
                                "path": row["path"],
                                "title": row["title"],
                                "owner": row["owner"],
                                "area": row["area"],
                                "stability": row["stability"],
                                "audience": row["audience"],
                            })).collect::<Vec<_>>()
                        });
                        fs::write(
                            ctx.repo_root.join("docs/metadata/front-matter.index.json"),
                            serde_json::to_string_pretty(&front_matter_index)
                                .map_err(|e| format!("front matter index encode failed: {e}"))?,
                        )
                        .map_err(|e| format!("write front matter index failed: {e}"))?;
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
                                ctx.repo_root.join("docs/_generated/make-targets.md"),
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
                            "front_matter_index": "docs/metadata/front-matter.index.json",
                            "command_index": "docs/_generated/command-index.json",
                            "schema_index": "docs/_generated/schema-index.json",
                            "docs_quality_dashboard": "docs/_generated/docs-quality-dashboard.json",
                            "generated_make_targets": "docs/_generated/make-targets.md"
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
