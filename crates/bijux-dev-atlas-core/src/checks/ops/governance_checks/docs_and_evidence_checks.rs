pub(super) fn check_ops_docs_governance(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let retired_docs_subtree = Path::new("docs/ops");
    if ctx.adapters.fs.exists(ctx.repo_root, retired_docs_subtree) {
        violations.push(violation(
            "OPS_DOCS_RETIRED_SUBTREE_REINTRODUCED",
            format!(
                "retired docs subtree `{}` must not exist",
                retired_docs_subtree.display()
            ),
            "keep ops handbook docs under docs/operations and remove docs/ops",
            Some(retired_docs_subtree),
        ));
    }
    let forbidden_transitional_tokens = ["phase", "task"];
    for root in ["ops", "docs"] {
        for file in walk_files(&ctx.repo_root.join(root)) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            let rel_str = rel.display().to_string();
            let has_forbidden_segment = rel
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .any(|segment| {
                    let lowercase = segment.to_ascii_lowercase();
                    forbidden_transitional_tokens.iter().any(|token| {
                        lowercase == *token
                            || lowercase.starts_with(&format!("{token}-"))
                            || lowercase.ends_with(&format!("-{token}"))
                            || lowercase.contains(&format!("-{token}-"))
                            || lowercase.starts_with(&format!("{token}_"))
                            || lowercase.ends_with(&format!("_{token}"))
                            || lowercase.contains(&format!("_{token}_"))
                    })
                });
            if has_forbidden_segment {
                violations.push(violation(
                    "OPS_NAMING_TRANSITIONAL_TOKEN_FORBIDDEN",
                    format!(
                        "path uses transitional naming token (`phase`/`task`): `{rel_str}`"
                    ),
                    "rename files/directories to durable intent-based names",
                    Some(rel),
                ));
            }
        }
    }

    let domain_dirs = [
        "ops/datasets",
        "ops/e2e",
        "ops/k8s",
        "ops/load",
        "ops/observe",
        "ops/report",
        "ops/stack",
        "ops/env",
        "ops/inventory",
        "ops/schema",
    ];
    for domain in domain_dirs {
        let index_rel = Path::new(domain).join("INDEX.md");
        if ctx.adapters.fs.exists(ctx.repo_root, &index_rel) {
            let index_text = fs::read_to_string(ctx.repo_root.join(&index_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            for line in index_text.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if !trimmed.starts_with("- ") {
                    violations.push(violation(
                        "OPS_DOC_INDEX_NON_LINK_CONTENT",
                        format!(
                            "domain index must be links-only; found non-link content in `{}`: `{trimmed}`",
                            index_rel.display()
                        ),
                        "keep domain INDEX.md files links-only with headings and bullet links",
                        Some(&index_rel),
                    ));
                }
            }

            for required_doc in ["README.md", "CONTRACT.md", "REQUIRED_FILES.md", "OWNER.md"] {
                let doc_rel = Path::new(domain).join(required_doc);
                if ctx.adapters.fs.exists(ctx.repo_root, &doc_rel)
                    && !index_text.contains(required_doc)
                {
                    violations.push(violation(
                        "OPS_DOC_INDEX_REQUIRED_LINK_MISSING",
                        format!(
                            "domain index `{}` must link `{}`",
                            index_rel.display(),
                            doc_rel.display()
                        ),
                        "add README.md and CONTRACT.md links to domain INDEX.md when files exist",
                        Some(&index_rel),
                    ));
                }
            }
        }

        let readme_rel = Path::new(domain).join("README.md");
        if ctx.adapters.fs.exists(ctx.repo_root, &readme_rel) {
            let readme_text = fs::read_to_string(ctx.repo_root.join(&readme_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let line_count = readme_text.lines().count();
            if line_count > 30 {
                violations.push(violation(
                    "OPS_DOC_README_SIZE_BUDGET_EXCEEDED",
                    format!(
                        "domain README exceeds 30 line budget: `{}` has {} lines",
                        readme_rel.display(),
                        line_count
                    ),
                    "keep domain README focused on what it is and where to start within 30 lines",
                    Some(&readme_rel),
                ));
            }
        }
    }

    let reference_index_rel = Path::new("docs/operations/ops-system/INDEX.md");
    let reference_index_text = fs::read_to_string(ctx.repo_root.join(reference_index_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let docs_root = ctx.repo_root.join("docs/operations/ops-system");
    for doc in walk_files(&docs_root) {
        let rel = doc.strip_prefix(ctx.repo_root).unwrap_or(doc.as_path());
        if rel.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let Some(name) = rel.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if name == "INDEX.md" {
            continue;
        }
        if !reference_index_text.contains(&format!("({name})")) {
            violations.push(violation(
                "OPS_REPORT_DOC_ORPHAN",
                format!(
                    "ops doc `{}` is not linked from docs/operations/ops-system/INDEX.md",
                    rel.display()
                ),
                "add doc link to docs/operations/ops-system/INDEX.md or remove orphan ops-system doc",
                Some(reference_index_rel),
            ));
        }
    }
    for target in markdown_link_targets(&reference_index_text) {
        let rel = Path::new("docs/operations/ops-system").join(&target);
        if !ctx.adapters.fs.exists(ctx.repo_root, &rel) {
            violations.push(violation(
                "OPS_REPORT_DOC_REFERENCE_BROKEN_LINK",
                format!(
                    "docs/operations/ops-system/INDEX.md links missing ops doc `{}`",
                    rel.display()
                ),
                "fix broken docs links in docs/operations/ops-system/INDEX.md",
                Some(reference_index_rel),
            ));
        }
    }

    let control_plane_rel = Path::new("ops/CONTROL_PLANE.md");
    let control_plane_snapshot_rel = Path::new("ops/_generated.example/control-plane.snapshot.md");
    let control_plane_drift_rel = Path::new("ops/_generated.example/control-plane-drift-report.json");
    let control_plane_surface_list_rel =
        Path::new("ops/_generated.example/control-plane-surface-list.json");
    if !ctx
        .adapters
        .fs
        .exists(ctx.repo_root, control_plane_snapshot_rel)
    {
        violations.push(violation(
            "OPS_CONTROL_PLANE_SNAPSHOT_MISSING",
            format!(
                "missing control-plane snapshot `{}`",
                control_plane_snapshot_rel.display()
            ),
            "generate and commit control-plane snapshot for drift checks",
            Some(control_plane_snapshot_rel),
        ));
    } else {
        let current = fs::read_to_string(ctx.repo_root.join(control_plane_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let snapshot = fs::read_to_string(ctx.repo_root.join(control_plane_snapshot_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if current != snapshot {
            violations.push(violation(
                "OPS_CONTROL_PLANE_SNAPSHOT_DRIFT",
                "ops/CONTROL_PLANE.md does not match ops/_generated.example/control-plane.snapshot.md"
                    .to_string(),
                "refresh control-plane snapshot to match current control-plane contract",
                Some(control_plane_snapshot_rel),
            ));
        }
        for line in current.lines() {
            let lower = line.to_ascii_lowercase();
            if (lower.contains("example") || lower.contains("examples")) || !line.contains("bijux-")
            {
                continue;
            }
            violations.push(violation(
                "OPS_CONTROL_PLANE_CRATE_LIST_FORBIDDEN",
                format!(
                    "ops/CONTROL_PLANE.md contains crate reference outside example context: `{}`",
                    line.trim()
                ),
                "keep ops/CONTROL_PLANE.md policy-only; move current crate inventory to ops/_generated.example/control-plane.snapshot.md",
                Some(control_plane_rel),
            ));
            break;
        }
    }

    if !ctx
        .adapters
        .fs
        .exists(ctx.repo_root, control_plane_drift_rel)
    {
        violations.push(violation(
            "OPS_CONTROL_PLANE_DRIFT_REPORT_MISSING",
            format!(
                "missing control-plane drift report `{}`",
                control_plane_drift_rel.display()
            ),
            "generate and commit control-plane drift report artifact",
            Some(control_plane_drift_rel),
        ));
    } else {
        let drift_text = fs::read_to_string(ctx.repo_root.join(control_plane_drift_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let drift_json: serde_json::Value =
            serde_json::from_str(&drift_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        if drift_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_CONTROL_PLANE_DRIFT_REPORT_BLOCKING",
                "control-plane-drift-report.json status is not `pass`".to_string(),
                "resolve control-plane drift and regenerate control-plane-drift-report.json",
                Some(control_plane_drift_rel),
            ));
        }
        let has_surface_check = drift_json
            .get("checks")
            .and_then(|v| v.as_array())
            .map(|checks| {
                checks.iter().any(|item| {
                    item.get("id").and_then(|v| v.as_str())
                        == Some("control-plane-surface-list-present")
                        && item.get("status").and_then(|v| v.as_str()) == Some("pass")
                })
            })
            .unwrap_or(false);
        if !has_surface_check {
            violations.push(violation(
                "OPS_CONTROL_PLANE_SURFACE_LIST_CHECK_MISSING",
                "control-plane-drift-report.json must include passing `control-plane-surface-list-present` check"
                    .to_string(),
                "regenerate control-plane drift report with control-plane surface-list status",
                Some(control_plane_drift_rel),
            ));
        }
    }

    if !ctx
        .adapters
        .fs
        .exists(ctx.repo_root, control_plane_surface_list_rel)
    {
        violations.push(violation(
            "OPS_CONTROL_PLANE_SURFACE_LIST_MISSING",
            format!(
                "missing control-plane surface list report `{}`",
                control_plane_surface_list_rel.display()
            ),
            "generate and commit control-plane-surface-list report",
            Some(control_plane_surface_list_rel),
        ));
    } else {
        let surface_text = fs::read_to_string(ctx.repo_root.join(control_plane_surface_list_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let surface_json: serde_json::Value = serde_json::from_str(&surface_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if surface_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_CONTROL_PLANE_SURFACE_LIST_BLOCKING",
                "control-plane-surface-list.json status is not `pass`".to_string(),
                "resolve control-plane surface-list drift and regenerate the report",
                Some(control_plane_surface_list_rel),
            ));
        }
        let surfaces = surface_json
            .get("surfaces")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let expected = ["check", "docs", "configs", "ops"];
        for required in expected {
            if !surfaces
                .iter()
                .any(|value| value.as_str() == Some(required))
            {
                violations.push(violation(
                    "OPS_CONTROL_PLANE_SURFACE_LIST_INCOMPLETE",
                    format!(
                        "control-plane-surface-list.json missing required surface `{required}`"
                    ),
                    "regenerate control-plane surface list report from command ownership source",
                    Some(control_plane_surface_list_rel),
                ));
            }
        }
    }

    let docs_drift_rel = Path::new("ops/_generated.example/docs-drift-report.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, docs_drift_rel) {
        violations.push(violation(
            "OPS_DOCS_DRIFT_ARTIFACT_MISSING",
            format!("missing docs drift artifact `{}`", docs_drift_rel.display()),
            "generate and commit docs drift report artifact",
            Some(docs_drift_rel),
        ));
    } else {
        let docs_drift_text = fs::read_to_string(ctx.repo_root.join(docs_drift_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let docs_drift_json: serde_json::Value = serde_json::from_str(&docs_drift_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if docs_drift_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_DOCS_DRIFT_REPORT_BLOCKING",
                "docs-drift-report.json status is not `pass`".to_string(),
                "resolve docs drift and regenerate docs-drift-report.json",
                Some(docs_drift_rel),
            ));
        }
        if let Some(checks) = docs_drift_json.get("checks").and_then(|v| v.as_array()) {
            for check in checks {
                if check.get("status").and_then(|v| v.as_str()) != Some("pass") {
                    let id = check
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    violations.push(violation(
                        "OPS_DOCS_DRIFT_CHECK_BLOCKING",
                        format!("docs-drift-report check `{id}` is not pass"),
                        "fix the failing docs drift check and regenerate docs-drift-report.json",
                        Some(docs_drift_rel),
                    ));
                }
            }
        }
    }

    let docs_links_rel = Path::new("ops/_generated.example/docs-links-report.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, docs_links_rel) {
        violations.push(violation(
            "OPS_DOCS_LINKS_REPORT_MISSING",
            format!("missing docs links artifact `{}`", docs_links_rel.display()),
            "generate and commit docs links report artifact",
            Some(docs_links_rel),
        ));
    } else {
        let docs_links_text = fs::read_to_string(ctx.repo_root.join(docs_links_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let docs_links_json: serde_json::Value = serde_json::from_str(&docs_links_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if !ctx.adapters.fs.exists(ctx.repo_root, Path::new("docs/ops")) {
            let stale_docs_ops_ref = docs_links_json
                .get("links")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|entry| entry.get("doc").and_then(|v| v.as_str()))
                .find(|doc| doc.starts_with("docs/ops/"));
            if let Some(stale_doc) = stale_docs_ops_ref {
                violations.push(violation(
                    "OPS_DOCS_LINKS_REPORT_STALE_DOCS_OPS_REFERENCE",
                    format!(
                        "docs links report references retired docs path `{stale_doc}` while `docs/ops` does not exist"
                    ),
                    "regenerate docs-links-report.json after removing stale docs/ops references",
                    Some(docs_links_rel),
                ));
            }
        }
    }

    let docs_registry_rel = Path::new("docs/registry.json");
    if ctx.adapters.fs.exists(ctx.repo_root, docs_registry_rel) {
        let docs_registry_text = fs::read_to_string(ctx.repo_root.join(docs_registry_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let docs_registry_json: serde_json::Value = serde_json::from_str(&docs_registry_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let stale_ops_report_doc = docs_registry_json
            .get("entries")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.get("path").and_then(|v| v.as_str()))
            .find(|path| {
                path.starts_with("ops/report/docs/")
                    && *path != "ops/report/docs/README.md"
                    && *path != "ops/report/docs/REFERENCE_INDEX.md"
            });
        if let Some(stale_path) = stale_ops_report_doc {
            violations.push(violation(
                "OPS_DOCS_REGISTRY_STALE_OPS_REPORT_DOC_REFERENCE",
                format!(
                    "docs registry references removed ops report doc `{stale_path}`; only redirect stubs are allowed"
                ),
                "regenerate docs registry and generated docs indexes after removing stale ops/report/docs references",
                Some(docs_registry_rel),
            ));
        }
    }

    let forbidden_doc_refs = [
        "ops/schema/obs/",
        "ops/obs/",
        "ops/k8s/Makefile",
        "ops/load/k6/manifests/suites.json",
        "ops/load/k6/thresholds/",
    ];
    for file in walk_files(&ctx.repo_root.join("ops")) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        if rel.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let text = fs::read_to_string(&file).map_err(|err| CheckError::Failed(err.to_string()))?;
        for forbidden in forbidden_doc_refs {
            if text.contains(forbidden) {
                violations.push(violation(
                    "OPS_DOC_FORBIDDEN_PATH_REFERENCE",
                    format!(
                        "doc `{}` references retired or forbidden path `{forbidden}`",
                        rel.display()
                    ),
                    "replace with current canonical path and remove retired references",
                    Some(rel),
                ));
            }
        }
        if text.contains("TODO") || text.contains("TBD") {
            violations.push(violation(
                "OPS_DOC_TODO_MARKER_FORBIDDEN",
                format!("doc `{}` contains TODO/TBD marker", rel.display()),
                "remove TODO/TBD markers from ops docs for release-ready contracts",
                Some(rel),
            ));
        }
        if !rel.starts_with("ops/_generated")
            && !rel.starts_with("ops/_generated.example")
            && !rel.starts_with("ops/schema/generated")
        {
            let lower = text.to_ascii_lowercase();
            if lower.contains("final crate set")
                || lower.contains("crate set (locked)")
                || lower.contains("final crate list")
            {
                violations.push(violation(
                    "OPS_STALE_LOCKED_LANGUAGE",
                    format!(
                        "authored ops markdown `{}` contains stale locked/final wording",
                        rel.display()
                    ),
                    "remove stale locked/final claims from authored ops docs and keep current-state lists in generated artifacts",
                    Some(rel),
                ));
            }
        }
    }

    let surfaces_text = fs::read_to_string(ctx.repo_root.join("ops/inventory/surfaces.json"))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let surfaces_json: serde_json::Value =
        serde_json::from_str(&surfaces_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let allowed_commands = surfaces_json
        .get("bijux-dev-atlas_commands")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    for doc in walk_files(&ctx.repo_root.join("docs")) {
        let rel = doc.strip_prefix(ctx.repo_root).unwrap_or(doc.as_path());
        if rel.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let text = fs::read_to_string(&doc).map_err(|err| CheckError::Failed(err.to_string()))?;
        for command in extract_ops_command_refs(&text) {
            if !allowed_commands.contains(&command) {
                violations.push(violation(
                    "OPS_DOC_COMMAND_SURFACE_UNKNOWN",
                    format!(
                        "doc `{}` references command not in surfaces.json: `{command}`",
                        rel.display()
                    ),
                    "replace stale command references with canonical surfaces.json commands",
                    Some(rel),
                ));
            }
        }
    }
    let ops_markdown_files = walk_files(&ctx.repo_root.join("ops"))
        .into_iter()
        .filter(|path| path.extension().and_then(|v| v.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    let ops_markdown_file_budget = 103usize;
    if ops_markdown_files.len() > ops_markdown_file_budget {
        violations.push(violation(
            "OPS_MARKDOWN_FILE_BUDGET_EXCEEDED",
            format!(
                "ops markdown file budget exceeded: {} > {}",
                ops_markdown_files.len(),
                ops_markdown_file_budget
            ),
            "reduce ops markdown sprawl or move handbook content into docs/",
            Some(Path::new("ops")),
        ));
    }
    let ops_markdown_line_budget = 2800usize;
    let mut ops_markdown_lines = 0usize;
    let allowed_standard_names = BTreeSet::from([
        "README.md".to_string(),
        "INDEX.md".to_string(),
        "CONTRACT.md".to_string(),
        "REQUIRED_FILES.md".to_string(),
        "OWNER.md".to_string(),
    ]);
    let allowed_nonstandard_paths = BTreeSet::from([
        "ops/CONTROL_PLANE.md".to_string(),
        "ops/DIRECTORY_BUDGET_POLICY.md".to_string(),
        "ops/DRIFT.md".to_string(),
        "ops/ERRORS.md".to_string(),
        "ops/GENERATED_LIFECYCLE.md".to_string(),
        "ops/NAMING.md".to_string(),
        "ops/SSOT.md".to_string(),
        "ops/_generated.example/INDEX.example.md".to_string(),
        "ops/_generated.example/MIRROR_POLICY.md".to_string(),
        "ops/_generated.example/control-plane.snapshot.md".to_string(),
        "ops/datasets/FIXTURE_LIFECYCLE.md".to_string(),
        "ops/load/evaluations/POLICY.md".to_string(),
        "ops/observe/drills/templates/incident-template.md".to_string(),
        "ops/schema/BUDGET_POLICY.md".to_string(),
        "ops/schema/SCHEMA_BUDGET_EXCEPTIONS.md".to_string(),
        "ops/schema/SCHEMA_REFERENCE_ALLOWLIST.md".to_string(),
        "ops/schema/VERSIONING_POLICY.md".to_string(),
        "ops/schema/generated/schema-index.md".to_string(),
        "ops/stack/dependencies.md".to_string(),
    ]);
    let ops_markdown_max_depth = 3usize;
    let depth_exception_paths = BTreeSet::from([
        "ops/k8s/charts/bijux-atlas/README.md".to_string(),
        "ops/load/k6/queries/INDEX.md".to_string(),
        "ops/observe/drills/templates/incident-template.md".to_string(),
    ]);
    let ops_markdown_domain_budget = BTreeMap::from([
        ("_generated".to_string(), 3usize),
        ("_generated.example".to_string(), 7usize),
        ("datasets".to_string(), 11usize),
        ("e2e".to_string(), 10usize),
        ("env".to_string(), 4usize),
        ("inventory".to_string(), 4usize),
        ("k8s".to_string(), 8usize),
        ("load".to_string(), 11usize),
        ("observe".to_string(), 9usize),
        ("report".to_string(), 8usize),
        ("schema".to_string(), 12usize),
        ("stack".to_string(), 8usize),
    ]);
    let mut ops_markdown_domain_counts = BTreeMap::<String, usize>::new();
    for doc in &ops_markdown_files {
        let rel = doc.strip_prefix(ctx.repo_root).unwrap_or(doc.as_path());
        let rel_str = rel.display().to_string();
        let name = rel
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_string();
        let text = fs::read_to_string(doc).map_err(|err| CheckError::Failed(err.to_string()))?;
        ops_markdown_lines += text.lines().count();
        if let Ok(stripped) = rel.strip_prefix("ops") {
            let domain = stripped
                .components()
                .next()
                .and_then(|component| component.as_os_str().to_str())
                .map(ToString::to_string);
            if let Some(domain) = domain {
                *ops_markdown_domain_counts.entry(domain).or_insert(0) += 1;
            }
        }
        let depth = rel.components().count().saturating_sub(1);
        if depth > ops_markdown_max_depth && !depth_exception_paths.contains(&rel_str) {
            violations.push(violation(
                "OPS_MARKDOWN_DEPTH_BUDGET_EXCEEDED",
                format!(
                    "ops markdown depth exceeded: `{rel_str}` depth={} max={}",
                    depth, ops_markdown_max_depth
                ),
                "move deep ops markdown into docs/operations or add an explicit governance allowlist exception",
                Some(rel),
            ));
        }
        if rel.starts_with(Path::new("ops/report/docs/")) {
            if name != "README.md" && name != "REFERENCE_INDEX.md" {
                violations.push(violation(
                    "OPS_MARKDOWN_FILENAME_FORBIDDEN",
                    format!("non-canonical markdown file under redirect-only area: `{rel_str}`"),
                    "keep only redirect stubs under ops/report/docs or migrate docs to docs/operations",
                    Some(rel),
                ));
            }
        } else if !allowed_standard_names.contains(&name)
            && !allowed_nonstandard_paths.contains(&rel_str)
        {
            violations.push(violation(
                "OPS_MARKDOWN_FILENAME_FORBIDDEN",
                format!("non-canonical markdown file under ops: `{rel_str}`"),
                "rename to canonical doc filenames or add explicit governance allowlist entry",
                Some(rel),
            ));
        }
        for line in text.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') && trimmed.to_ascii_lowercase().contains("how to") {
                violations.push(violation(
                    "OPS_MARKDOWN_HOW_TO_HEADING_FORBIDDEN",
                    format!("ops markdown contains workflow-style heading in `{rel_str}`"),
                    "move tutorial/workflow prose to docs/operations and keep ops markdown normative",
                    Some(rel),
                ));
                break;
            }
            let line_lower = trimmed.to_ascii_lowercase();
            let is_markdown_link_line = trimmed.starts_with("- [") && trimmed.contains("](");
            if line_lower.contains("how to") && !is_markdown_link_line {
                violations.push(violation(
                    "OPS_MARKDOWN_HOW_TO_PHRASE_FORBIDDEN",
                    format!("ops markdown contains \"How to\" prose in `{rel_str}`"),
                    "move workflow prose to docs/operations and keep ops markdown normative",
                    Some(rel),
                ));
                break;
            }
        }
        for command in extract_ops_command_refs(&text) {
            if !allowed_commands.contains(&command) {
                violations.push(violation(
                    "OPS_MARKDOWN_COMMAND_SURFACE_UNKNOWN",
                    format!(
                        "ops markdown `{}` references command not in surfaces.json: `{command}`",
                        rel.display()
                    ),
                    "replace stale command references with canonical surfaces.json commands",
                    Some(rel),
                ));
            }
        }
    }
    for (domain, max) in ops_markdown_domain_budget {
        let count = ops_markdown_domain_counts.get(&domain).copied().unwrap_or(0);
        if count > max {
            violations.push(violation(
                "OPS_MARKDOWN_DOMAIN_BUDGET_EXCEEDED",
                format!(
                    "ops markdown domain budget exceeded for `{domain}`: {count} > {max}"
                ),
                "move handbook-style docs out of ops domain surfaces and keep only canonical headers",
                Some(Path::new("ops")),
            ));
        }
    }
    if ops_markdown_lines > ops_markdown_line_budget {
        violations.push(violation(
            "OPS_MARKDOWN_LINE_BUDGET_EXCEEDED",
            format!(
                "ops markdown line budget exceeded: {} > {}",
                ops_markdown_lines, ops_markdown_line_budget
            ),
            "move handbook-style content into docs/ and keep ops markdown concise",
            Some(Path::new("ops")),
        ));
    }
    let mut seen_docs_dirs = BTreeSet::new();
    for file in walk_files(&ctx.repo_root.join("ops")) {
        let mut parent = file.parent();
        while let Some(dir) = parent {
            let rel = dir.strip_prefix(ctx.repo_root).unwrap_or(dir);
            if rel == Path::new("ops/report/docs") {
                break;
            }
            if rel
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == "docs")
                && seen_docs_dirs.insert(rel.to_path_buf())
            {
                violations.push(violation(
                    "OPS_DOCS_DIRECTORY_FORBIDDEN",
                    format!(
                        "forbidden ops docs subtree `{}`; ops docs must live under docs/operations",
                        rel.display()
                    ),
                    "remove ops/**/docs/** subtree or migrate docs into docs/operations",
                    Some(rel),
                ));
            }
            parent = dir.parent();
        }
    }

    let ops_index_rel = Path::new("ops/INDEX.md");
    let ops_index_text = fs::read_to_string(ctx.repo_root.join(ops_index_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for root_doc in [
        "CONTRACT.md",
        "CONTROL_PLANE.md",
        "DRIFT.md",
        "ERRORS.md",
        "NAMING.md",
        "README.md",
        "SSOT.md",
    ] {
        let rel = Path::new("ops").join(root_doc);
        if ctx.adapters.fs.exists(ctx.repo_root, &rel) && !ops_index_text.contains(root_doc) {
            violations.push(violation(
                "OPS_ROOT_DOC_INDEX_LINK_MISSING",
                format!(
                    "ops root document `{}` must be linked from `ops/INDEX.md`",
                    rel.display()
                ),
                "link all root ops docs from ops/INDEX.md",
                Some(ops_index_rel),
            ));
        }
    }
    let index_line_count = ops_index_text.lines().count();
    if index_line_count > 80 {
        violations.push(violation(
            "OPS_ROOT_INDEX_SIZE_BUDGET_EXCEEDED",
            format!(
                "ops/INDEX.md exceeds max line budget (80): {} lines",
                index_line_count
            ),
            "keep ops/INDEX.md compact and move details to linked docs",
            Some(ops_index_rel),
        ));
    }
    let root_doc_line_budgets = [
        ("ops/README.md", 80usize),
        ("ops/CONTRACT.md", 140usize),
        ("ops/CONTROL_PLANE.md", 80usize),
        ("ops/DRIFT.md", 80usize),
        ("ops/ERRORS.md", 80usize),
        ("ops/NAMING.md", 80usize),
        ("ops/SSOT.md", 80usize),
    ];
    for (rel_str, max_lines) in root_doc_line_budgets {
        let rel = Path::new(rel_str);
        if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
            continue;
        }
        let text = fs::read_to_string(ctx.repo_root.join(rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let line_count = text.lines().count();
        if line_count > max_lines {
            violations.push(violation(
                "OPS_ROOT_DOC_SIZE_BUDGET_EXCEEDED",
                format!(
                    "ops root doc exceeds line budget: `{}` has {} lines (max {})",
                    rel.display(),
                    line_count,
                    max_lines
                ),
                "keep root governance docs compact and move extended narrative into docs/",
                Some(rel),
            ));
        }
    }

    Ok(violations)
}

fn markdown_link_targets(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let mut cursor = line;
        while let Some(start) = cursor.find('(') {
            let after_start = &cursor[start + 1..];
            let Some(end) = after_start.find(')') else {
                break;
            };
            let target = &after_start[..end];
            if target.ends_with(".md") && !target.contains("://") {
                out.push(target.to_string());
            }
            cursor = &after_start[end + 1..];
        }
    }
    out
}

fn extract_ops_command_refs(content: &str) -> BTreeSet<String> {
    let mut commands = BTreeSet::new();
    for line in content.lines() {
        let mut cursor = line;
        while let Some(pos) = cursor.find("bijux dev atlas ops ") {
            let after = &cursor[pos + "bijux dev atlas ops ".len()..];
            let mut tokens = Vec::new();
            for token in after.split_whitespace() {
                if token.starts_with("--")
                    || token.starts_with('`')
                    || token.starts_with('|')
                    || token.starts_with('(')
                {
                    break;
                }
                let clean = token
                    .trim_matches(|ch: char| ",.;:()[]`".contains(ch))
                    .to_string();
                if clean.is_empty() {
                    break;
                }
                if !clean
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
                {
                    break;
                }
                tokens.push(clean);
                if tokens.len() >= 3 {
                    break;
                }
            }
            if !tokens.is_empty() {
                commands.insert(format!("bijux dev atlas ops {}", tokens.join(" ")));
            }
            cursor = after;
        }
    }
    commands
}

pub(super) fn check_ops_evidence_bundle_discipline(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let generated_lifecycle_rel = Path::new("ops/GENERATED_LIFECYCLE.md");
    let mirror_policy_rel = Path::new("ops/_generated.example/MIRROR_POLICY.md");
    let allowlist_rel = Path::new("ops/_generated.example/ALLOWLIST.json");
    let ops_index_rel = Path::new("ops/_generated.example/ops-index.json");
    let scorecard_rel = Path::new("ops/_generated.example/scorecard.json");
    let bundle_rel = Path::new("ops/_generated.example/ops-evidence-bundle.json");
    let contract_audit_rel = Path::new("ops/_generated.example/contract-audit-report.json");
    let contract_graph_rel = Path::new("ops/_generated.example/contract-dependency-graph.json");
    let control_graph_diff_rel =
        Path::new("ops/_generated.example/control-graph-diff-report.json");
    let schema_drift_rel = Path::new("ops/_generated.example/schema-drift-report.json");
    let gates_rel = Path::new("ops/inventory/gates.json");

    for rel in [
        generated_lifecycle_rel,
        mirror_policy_rel,
        allowlist_rel,
        ops_index_rel,
        scorecard_rel,
        bundle_rel,
        contract_audit_rel,
        contract_graph_rel,
        control_graph_diff_rel,
        schema_drift_rel,
    ] {
        if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
            violations.push(violation(
                "OPS_EVIDENCE_REQUIRED_ARTIFACT_MISSING",
                format!("missing required evidence artifact `{}`", rel.display()),
                "generate and commit required evidence artifact",
                Some(rel),
            ));
        }
    }

    let mirror_policy_text = fs::read_to_string(ctx.repo_root.join(mirror_policy_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let generated_lifecycle_text = fs::read_to_string(ctx.repo_root.join(generated_lifecycle_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for required in [
        "## Lifecycle Classes",
        "transient_runtime",
        "domain_derived",
        "curated_evidence",
        "## Retention Policy",
        "## Regeneration Triggers",
        "## Deterministic Ordering",
        "## Generator Contract Versioning",
    ] {
        if !generated_lifecycle_text.contains(required) {
            violations.push(violation(
                "OPS_GENERATED_LIFECYCLE_CONTRACT_INCOMPLETE",
                format!(
                    "generated lifecycle contract must include required section or marker `{required}`"
                ),
                "update ops/GENERATED_LIFECYCLE.md with complete lifecycle, retention, trigger, and versioning policy",
                Some(generated_lifecycle_rel),
            ));
        }
    }
    for required in [
        "ops-index.json",
        "ops-evidence-bundle.json",
        "scorecard.json",
        "ALLOWLIST.json",
        "contract-audit-report.json",
        "contract-dependency-graph.json",
        "inventory-index.json",
        "control-plane.snapshot.md",
        "control-graph-diff-report.json",
        "docs-drift-report.json",
        "schema-drift-report.json",
        "stack-drift-report.json",
        "ops/GENERATED_LIFECYCLE.md",
    ] {
        if !mirror_policy_text.contains(required) {
            violations.push(violation(
                "OPS_EVIDENCE_MIRROR_POLICY_INCOMPLETE",
                format!("mirror policy must declare mirrored artifact `{required}`"),
                "update MIRROR_POLICY.md mirrored artifact list",
                Some(mirror_policy_rel),
            ));
        }
    }

    let allowlist_text = fs::read_to_string(ctx.repo_root.join(allowlist_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let allowlist_json: serde_json::Value =
        serde_json::from_str(&allowlist_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let allowlisted_files = allowlist_json
        .get("allowed_files")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if allowlisted_files.is_empty() {
        violations.push(violation(
            "OPS_EVIDENCE_ALLOWLIST_EMPTY",
            "ops/_generated.example/ALLOWLIST.json must declare non-empty `allowed_files`"
                .to_string(),
            "populate ALLOWLIST.json with exact committed files allowed under ops/_generated.example",
            Some(allowlist_rel),
        ));
    }
    let generated_example_root = ctx.repo_root.join("ops/_generated.example");
    if generated_example_root.exists() {
        let mut seen_files = BTreeSet::new();
        for file in walk_files(&generated_example_root) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            let rel_str = rel.display().to_string();
            seen_files.insert(rel_str.clone());
            if !allowlisted_files.contains(&rel_str) {
                violations.push(violation(
                    "OPS_EVIDENCE_ALLOWLIST_MISSING_FILE",
                    format!(
                        "committed file `{}` is not declared in ops/_generated.example/ALLOWLIST.json",
                        rel.display()
                    ),
                    "update ALLOWLIST.json when adding or removing curated evidence artifacts",
                    Some(allowlist_rel),
                ));
            }
            if is_binary_like_file(&file)? {
                violations.push(violation(
                    "OPS_EVIDENCE_BINARY_FORBIDDEN",
                    format!(
                        "binary file is forbidden under ops/_generated.example: `{}`",
                        rel.display()
                    ),
                    "keep _generated.example text-only curated evidence artifacts",
                    Some(rel),
                ));
            }
            if rel.extension().and_then(|ext| ext.to_str()) == Some("json") {
                let file_name = rel
                    .file_name()
                    .and_then(|v| v.to_str())
                    .unwrap_or_default()
                    .to_string();
                let suffix_allowed = file_name.ends_with("-report.json")
                    || file_name.ends_with("-index.json")
                    || file_name.ends_with(".example.json")
                    || matches!(
                        file_name.as_str(),
                        "ALLOWLIST.json"
                            | "ops-ledger.json"
                            | "ops-index.json"
                            | "ops-evidence-bundle.json"
                            | "scorecard.json"
                            | "control-plane-surface-list.json"
                    );
                if !suffix_allowed {
                    violations.push(violation(
                        "OPS_EVIDENCE_FILENAME_SUFFIX_POLICY_VIOLATION",
                        format!(
                            "curated evidence json filename does not match suffix policy: `{}`",
                            rel.display()
                        ),
                        "use -report.json, -index.json, .example.json, or an approved canonical evidence filename",
                        Some(rel),
                    ));
                }
                let text =
                    fs::read_to_string(&file).map_err(|err| CheckError::Failed(err.to_string()))?;
                let json: serde_json::Value = serde_json::from_str(&text)
                    .map_err(|err| CheckError::Failed(err.to_string()))?;
                if json.get("schema_version").is_none() {
                    violations.push(violation(
                        "OPS_EVIDENCE_SCHEMA_VERSION_MISSING",
                        format!(
                            "curated evidence json `{}` must include schema_version",
                            rel.display()
                        ),
                        "add schema_version to curated evidence json artifact",
                        Some(rel),
                    ));
                }
                if json.get("generated_by").is_none() {
                    violations.push(violation(
                        "OPS_EVIDENCE_GENERATED_BY_MISSING",
                        format!(
                            "curated evidence json `{}` must include generated_by",
                            rel.display()
                        ),
                        "add generated_by to curated evidence json artifact",
                        Some(rel),
                    ));
                }
            }
        }
        for allowlisted in &allowlisted_files {
            let rel = Path::new(allowlisted);
            if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
                violations.push(violation(
                    "OPS_EVIDENCE_ALLOWLIST_STALE_ENTRY",
                    format!(
                        "allowlist entry points to missing curated artifact `{}`",
                        rel.display()
                    ),
                    "remove stale entry from ALLOWLIST.json or restore the artifact",
                    Some(allowlist_rel),
                ));
            }
            if !seen_files.contains(allowlisted) {
                violations.push(violation(
                    "OPS_EVIDENCE_ALLOWLIST_UNUSED_ENTRY",
                    format!(
                        "allowlist entry does not match a committed curated artifact `{}`",
                        rel.display()
                    ),
                    "keep ALLOWLIST.json aligned with committed files in ops/_generated.example",
                    Some(allowlist_rel),
                ));
            }
        }
    }

    let bundle_text = fs::read_to_string(ctx.repo_root.join(bundle_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let bundle_json: serde_json::Value =
        serde_json::from_str(&bundle_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    for key in [
        "schema_version",
        "release",
        "status",
        "hashes",
        "gates",
        "pin_freeze_status",
    ] {
        if bundle_json.get(key).is_none() {
            violations.push(violation(
                "OPS_EVIDENCE_BUNDLE_REQUIRED_KEY_MISSING",
                format!("evidence bundle missing required key `{key}`"),
                "populate required evidence bundle key",
                Some(bundle_rel),
            ));
        }
    }

    if let Some(schema_index) = bundle_json
        .get("hashes")
        .and_then(|v| v.get("schema_index"))
        .and_then(|v| v.as_object())
    {
        let Some(path) = schema_index.get("path").and_then(|v| v.as_str()) else {
            return Ok(violations);
        };
        let Some(sha) = schema_index.get("sha256").and_then(|v| v.as_str()) else {
            return Ok(violations);
        };
        let path_rel = Path::new(path);
        if !ctx.adapters.fs.exists(ctx.repo_root, path_rel) {
            violations.push(violation(
                "OPS_EVIDENCE_BUNDLE_SCHEMA_INDEX_PATH_MISSING",
                format!("schema index path in evidence bundle does not exist: `{path}`"),
                "fix hashes.schema_index.path in evidence bundle",
                Some(bundle_rel),
            ));
        } else {
            let actual_sha = sha256_hex(&ctx.repo_root.join(path_rel))?;
            if actual_sha != sha {
                violations.push(violation(
                    "OPS_EVIDENCE_BUNDLE_SCHEMA_INDEX_HASH_DRIFT",
                    "schema index hash in evidence bundle is stale".to_string(),
                    "refresh hashes.schema_index.sha256 in ops-evidence-bundle.json",
                    Some(bundle_rel),
                ));
            }
        }
    }
    if let Some(inventory_hashes) = bundle_json
        .get("hashes")
        .and_then(|v| v.get("inventory"))
        .and_then(|v| v.as_array())
    {
        let mut paths = Vec::new();
        for entry in inventory_hashes {
            let path = entry
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let sha = entry
                .get("sha256")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            if !path.starts_with("ops/") {
                violations.push(violation(
                    "OPS_EVIDENCE_BUNDLE_INVENTORY_HASH_PATH_INVALID",
                    format!("inventory hash entry path must live under ops/: `{path}`"),
                    "set hashes.inventory[].path to canonical ops paths only",
                    Some(bundle_rel),
                ));
            }
            if !sha.chars().all(|c| c.is_ascii_hexdigit()) || sha.len() != 64 {
                violations.push(violation(
                    "OPS_EVIDENCE_BUNDLE_INVENTORY_HASH_INVALID",
                    format!("inventory hash entry must be 64 lowercase hex chars for `{path}`"),
                    "refresh hashes.inventory sha256 values from deterministic artifact hashing",
                    Some(bundle_rel),
                ));
            }
            paths.push(path);
        }
        let mut sorted = paths.clone();
        sorted.sort();
        if paths != sorted {
            violations.push(violation(
                "OPS_EVIDENCE_BUNDLE_INVENTORY_HASH_ORDER_NONDETERMINISTIC",
                "hashes.inventory must be sorted by path for deterministic output".to_string(),
                "sort hashes.inventory entries lexicographically by path",
                Some(bundle_rel),
            ));
        }
    }

    let schema_drift_text = fs::read_to_string(ctx.repo_root.join(schema_drift_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let schema_drift_json: serde_json::Value = serde_json::from_str(&schema_drift_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for key in [
        "schema_version",
        "generated_by",
        "status",
        "summary",
        "drift",
    ] {
        if schema_drift_json.get(key).is_none() {
            violations.push(violation(
                "OPS_SCHEMA_DRIFT_REPORT_INVALID",
                format!("schema drift report is missing required key `{key}`"),
                "populate schema drift report with required governance keys",
                Some(schema_drift_rel),
            ));
        }
    }

    let contract_audit_text = fs::read_to_string(ctx.repo_root.join(contract_audit_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let contract_audit_json: serde_json::Value = serde_json::from_str(&contract_audit_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for key in [
        "schema_version",
        "generated_by",
        "status",
        "summary",
        "contracts",
    ] {
        if contract_audit_json.get(key).is_none() {
            violations.push(violation(
                "OPS_CONTRACT_AUDIT_REPORT_INVALID",
                format!("contract audit report is missing required key `{key}`"),
                "populate contract-audit-report.json with required governance keys",
                Some(contract_audit_rel),
            ));
        }
    }

    let contract_graph_text = fs::read_to_string(ctx.repo_root.join(contract_graph_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let contract_graph_json: serde_json::Value = serde_json::from_str(&contract_graph_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for key in ["schema_version", "generated_by", "nodes", "edges"] {
        if contract_graph_json.get(key).is_none() {
            violations.push(violation(
                "OPS_CONTRACT_DEPENDENCY_GRAPH_INVALID",
                format!("contract dependency graph is missing required key `{key}`"),
                "populate contract-dependency-graph.json with nodes and dependency edges",
                Some(contract_graph_rel),
            ));
        }
    }

    let gates_text = fs::read_to_string(ctx.repo_root.join(gates_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let gates_json: serde_json::Value =
        serde_json::from_str(&gates_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let expected_gates = gates_json
        .get("gates")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.get("id").and_then(|v| v.as_str()))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let bundle_gates = bundle_json
        .get("gates")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.get("id").and_then(|v| v.as_str()))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if expected_gates != bundle_gates {
        violations.push(violation(
            "OPS_EVIDENCE_BUNDLE_GATE_LIST_DRIFT",
            format!(
                "evidence bundle gates mismatch: expected={expected_gates:?} actual={bundle_gates:?}"
            ),
            "synchronize evidence bundle gates list with ops/inventory/gates.json",
            Some(bundle_rel),
        ));
    }

    let generated_root = ctx.repo_root.join("ops/_generated");
    if generated_root.exists() {
        let allowed = BTreeSet::from([
            "ops/_generated/.gitkeep".to_string(),
            "ops/_generated/OWNER.md".to_string(),
            "ops/_generated/README.md".to_string(),
            "ops/_generated/REQUIRED_FILES.md".to_string(),
        ]);
        for file in walk_files(&generated_root) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            let rel_str = rel.display().to_string();
            if !allowed.contains(&rel_str) {
                let is_json = rel.extension().and_then(|v| v.to_str()) == Some("json");
                if is_json {
                    violations.push(violation(
                        "OPS_GENERATED_RUNTIME_JSON_COMMITTED_FORBIDDEN",
                        format!(
                            "runtime json evidence must not be committed under ops/_generated: `{}`",
                            rel.display()
                        ),
                        "delete committed runtime json and regenerate into runtime-only ignored outputs",
                        Some(rel),
                    ));
                }
                violations.push(violation(
                    "OPS_GENERATED_DIRECTORY_COMMITTED_EVIDENCE_FORBIDDEN",
                    format!("ops/_generated contains unexpected committed file `{}`", rel.display()),
                    "keep ops/_generated to marker docs only; store curated evidence under ops/_generated.example",
                    Some(rel),
                ));
            }
        }
    }

    Ok(violations)
}

