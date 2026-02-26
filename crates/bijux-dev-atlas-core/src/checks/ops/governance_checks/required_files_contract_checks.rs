pub(super) fn check_ops_required_files_contracts(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let ops_root = ctx.repo_root.join("ops");
    if !ops_root.exists() {
        return Ok(Vec::new());
    }
    let mut violations = Vec::new();
    for required_doc in walk_files(&ops_root) {
        let rel = required_doc
            .strip_prefix(ctx.repo_root)
            .unwrap_or(required_doc.as_path());
        if rel.file_name().and_then(|n| n.to_str()) != Some("REQUIRED_FILES.md") {
            continue;
        }
        let content =
            fs::read_to_string(&required_doc).map_err(|err| CheckError::Failed(err.to_string()))?;
        let required_contract = parse_required_files_markdown_yaml(&content, rel)?;
        let required_files = required_contract.required_files.clone();
        let required_directories = required_contract.required_dirs.clone();
        let domain_root = rel.parent().unwrap_or(Path::new("ops"));
        let domain_readme_rel = domain_root.join("README.md");
        let domain_readme_text = if ctx.adapters.fs.exists(ctx.repo_root, &domain_readme_rel) {
            Some(
                fs::read_to_string(ctx.repo_root.join(&domain_readme_rel))
                    .map_err(|err| CheckError::Failed(err.to_string()))?,
            )
        } else {
            None
        };

        for forbidden in [
            "ops/obs/",
            "ops/schema/obs/",
            "ops/load/k6/manifests/suites.json",
        ] {
            if content.contains(forbidden) {
                violations.push(violation(
                    "OPS_REQUIRED_FILES_FORBIDDEN_REFERENCE",
                    format!(
                        "`{}` contains forbidden reference `{forbidden}`",
                        rel.display()
                    ),
                    "remove retired path references from REQUIRED_FILES.md",
                    Some(rel),
                ));
            }
        }
        for forbidden in &required_contract.forbidden_patterns {
            if forbidden.is_empty() {
                continue;
            }
            for domain_file in walk_files(&ctx.repo_root.join(domain_root)) {
                let domain_rel = domain_file
                    .strip_prefix(ctx.repo_root)
                    .unwrap_or(domain_file.as_path());
                if domain_rel == rel {
                    continue;
                }
                let Ok(domain_text) = fs::read_to_string(&domain_file) else {
                    continue;
                };
                if domain_text.contains(forbidden) {
                    violations.push(violation(
                        "OPS_REQUIRED_FILES_FORBIDDEN_PATTERN_MATCHED",
                        format!(
                            "forbidden pattern `{}` found in `{}`",
                            forbidden,
                            domain_rel.display()
                        ),
                        "remove forbidden path/pattern references from domain files",
                        Some(domain_rel),
                    ));
                }
            }
        }
        if content.contains("TODO") || content.contains("TBD") {
            violations.push(violation(
                "OPS_REQUIRED_FILES_PLACEHOLDER_FORBIDDEN",
                format!("`{}` contains TODO/TBD placeholder markers", rel.display()),
                "replace TODO/TBD placeholders with concrete required file contracts",
                Some(rel),
            ));
        }
        for header_name in ["OWNER.md", "README.md", "INDEX.md", "CONTRACT.md"] {
            let header_rel = domain_root.join(header_name);
            if ctx.adapters.fs.exists(ctx.repo_root, &header_rel)
                && !required_files.iter().any(|file| file == &header_rel)
            {
                violations.push(violation(
                    "OPS_REQUIRED_FILES_DOMAIN_HEADER_MISSING",
                    format!(
                        "`{}` must include domain header `{}` in required_files",
                        rel.display(),
                        header_rel.display()
                    ),
                    "list domain header docs explicitly in required_files",
                    Some(rel),
                ));
            }
        }
        if !required_contract
            .notes
            .iter()
            .any(|note| note.starts_with("authored_root:"))
        {
            violations.push(violation(
                "OPS_REQUIRED_FILES_AUTHORED_ROOT_MISSING",
                format!(
                    "`{}` must include at least one `authored_root:` note",
                    rel.display()
                ),
                "add authored_root notes that point at canonical authored SSOT artifacts",
                Some(rel),
            ));
        }
        let generated_dir = domain_root.join("generated");
        if ctx.adapters.fs.exists(ctx.repo_root, &generated_dir)
            && !required_contract
                .notes
                .iter()
                .any(|note| note.starts_with("generated_output:"))
        {
            violations.push(violation(
                "OPS_REQUIRED_FILES_GENERATED_OUTPUT_MISSING",
                format!(
                    "`{}` must include at least one `generated_output:` note",
                    rel.display()
                ),
                "add generated_output notes for generated artifacts produced in the domain",
                Some(rel),
            ));
        }
        for file_rel in required_files {
            let file_path = ctx.repo_root.join(&file_rel);
            if !file_path.exists() {
                violations.push(violation(
                    "OPS_REQUIRED_FILE_MISSING",
                    format!("required file missing: `{}`", file_rel.display()),
                    "create missing required file or remove stale declaration from REQUIRED_FILES.md",
                    Some(rel),
                ));
                continue;
            }
            let metadata =
                fs::metadata(&file_path).map_err(|err| CheckError::Failed(err.to_string()))?;
            if metadata.len() == 0 {
                violations.push(violation(
                    "OPS_REQUIRED_FILE_EMPTY",
                    format!("required file is empty: `{}`", file_rel.display()),
                    "populate required file with non-empty contract content",
                    Some(&file_rel),
                ));
            }
            if file_rel.extension().and_then(|v| v.to_str()) == Some("md") {
                let domain_index = domain_root.join("INDEX.md");
                if ctx.adapters.fs.exists(ctx.repo_root, &domain_index)
                    && file_rel.starts_with(domain_root)
                    && file_rel.file_name().and_then(|v| v.to_str()) != Some("INDEX.md")
                {
                    let index_text = fs::read_to_string(ctx.repo_root.join(&domain_index))
                        .map_err(|err| CheckError::Failed(err.to_string()))?;
                    let file_name = file_rel.file_name().and_then(|v| v.to_str()).unwrap_or("");
                    if !index_text.contains(file_name) {
                        violations.push(violation(
                            "OPS_REQUIRED_DOC_NOT_INDEXED",
                            format!(
                                "required document `{}` is not linked from `{}`",
                                file_rel.display(),
                                domain_index.display()
                            ),
                            "add required doc link to the domain INDEX.md",
                            Some(&domain_index),
                        ));
                    }
                }
            }
        }
        for dir_rel in &required_directories {
            let dir_path = ctx.repo_root.join(dir_rel);
            if !dir_path.exists() || !dir_path.is_dir() {
                violations.push(violation(
                    "OPS_REQUIRED_DIRECTORY_MISSING",
                    format!("required directory missing: `{}`", dir_rel.display()),
                    "create required directory or remove stale declaration from REQUIRED_FILES.md",
                    Some(rel),
                ));
            } else {
                let mut entries = fs::read_dir(&dir_path)
                    .map_err(|err| CheckError::Failed(err.to_string()))?;
                if entries.next().is_none() {
                    violations.push(violation(
                        "OPS_EMPTY_DIRECTORY_WITHOUT_GITKEEP",
                        format!(
                            "required directory `{}` is empty and missing `.gitkeep`",
                            dir_rel.display()
                        ),
                        "add `.gitkeep` to empty required directories or remove the stale directory",
                        Some(rel),
                    ));
                }
            }
        }
        for file in walk_files(&ctx.repo_root.join(domain_root)) {
            if file.file_name().and_then(|n| n.to_str()) != Some(".gitkeep") {
                continue;
            }
            let Some(keep_dir) = file
                .parent()
                .and_then(|p| p.strip_prefix(ctx.repo_root).ok())
                .map(PathBuf::from)
            else {
                continue;
            };
            if !required_directories.iter().any(|dir| dir == &keep_dir) {
                violations.push(violation(
                    "OPS_REQUIRED_FILES_GITKEEP_DIR_UNDECLARED",
                    format!(
                        "directory with .gitkeep must be declared in required_dirs: `{}`",
                        keep_dir.display()
                    ),
                    "declare placeholder directories in required_dirs",
                    Some(rel),
                ));
            }
            if let Some(readme_text) = &domain_readme_text {
                let keep_dir_str = keep_dir.display().to_string();
                if !readme_text.contains(&keep_dir_str) {
                    violations.push(violation(
                        "OPS_PLACEHOLDER_DIR_README_NOTE_MISSING",
                        format!(
                            "placeholder directory `{}` is not documented in `{}`",
                            keep_dir.display(),
                            domain_readme_rel.display()
                        ),
                        "document each placeholder extension directory in the domain README",
                        Some(&domain_readme_rel),
                    ));
                }
            }
        }
    }

    let actual_gitkeep_dirs = walk_files(&ops_root)
        .into_iter()
        .filter(|p| p.file_name().and_then(|n| n.to_str()) == Some(".gitkeep"))
        .filter_map(|p| {
            p.parent()
                .and_then(|parent| parent.strip_prefix(ctx.repo_root).ok())
                .map(PathBuf::from)
        })
        .collect::<BTreeSet<_>>();
    let placeholder_allowlist_rel = Path::new("ops/inventory/placeholder-dirs.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, placeholder_allowlist_rel) {
        violations.push(violation(
            "OPS_PLACEHOLDER_DIR_ALLOWLIST_MISSING",
            "missing inventory placeholder-dir allowlist `ops/inventory/placeholder-dirs.json`"
                .to_string(),
            "add and maintain ops/inventory/placeholder-dirs.json as the single placeholder-dir allowlist",
            Some(placeholder_allowlist_rel),
        ));
    } else {
        let allowlist_text = fs::read_to_string(ctx.repo_root.join(placeholder_allowlist_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let allowlist_json: serde_json::Value = serde_json::from_str(&allowlist_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let mut allowlisted_dirs = BTreeSet::new();
        if let Some(entries) = allowlist_json
            .get("placeholder_entries")
            .and_then(|v| v.as_array())
        {
            for entry in entries {
                let path = entry
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let owner = entry
                    .get("owner")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let purpose = entry
                    .get("purpose")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let expected_contents = entry
                    .get("expected_contents")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let lifecycle_policy = entry
                    .get("lifecycle_policy")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_ascii_lowercase();

                if path.is_empty() {
                    violations.push(violation(
                        "OPS_PLACEHOLDER_DIR_ENTRY_PATH_MISSING",
                        "placeholder entry is missing `path`".to_string(),
                        "set placeholder_entries[].path in ops/inventory/placeholder-dirs.json",
                        Some(placeholder_allowlist_rel),
                    ));
                    continue;
                }
                allowlisted_dirs.insert(PathBuf::from(path));
                if owner.is_empty() {
                    violations.push(violation(
                        "OPS_PLACEHOLDER_DIR_ENTRY_OWNER_MISSING",
                        format!("placeholder entry `{path}` is missing owner"),
                        "set placeholder_entries[].owner for every placeholder directory",
                        Some(placeholder_allowlist_rel),
                    ));
                }
                if purpose.is_empty() {
                    violations.push(violation(
                        "OPS_PLACEHOLDER_DIR_ENTRY_PURPOSE_MISSING",
                        format!("placeholder entry `{path}` is missing purpose"),
                        "set placeholder_entries[].purpose for every placeholder directory",
                        Some(placeholder_allowlist_rel),
                    ));
                }
                if expected_contents.is_empty() {
                    violations.push(violation(
                        "OPS_PLACEHOLDER_DIR_ENTRY_EXPECTED_CONTENTS_MISSING",
                        format!("placeholder entry `{path}` is missing expected_contents"),
                        "set placeholder_entries[].expected_contents for every placeholder directory",
                        Some(placeholder_allowlist_rel),
                    ));
                }
                let has_permanent = lifecycle_policy.contains("permanent extension point");
                let has_sunset = lifecycle_policy.contains("sunset");
                if !has_permanent && !has_sunset {
                    violations.push(violation(
                        "OPS_PLACEHOLDER_DIR_ENTRY_LIFECYCLE_INVALID",
                        format!(
                            "placeholder entry `{path}` lifecycle_policy must declare sunset or permanent extension point"
                        ),
                        "set placeholder_entries[].lifecycle_policy to include `sunset` or `permanent extension point`",
                        Some(placeholder_allowlist_rel),
                    ));
                }
            }
        }
        if allowlisted_dirs.is_empty() {
            allowlisted_dirs = allowlist_json
                .get("placeholder_dirs")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_str())
                        .map(PathBuf::from)
                        .collect::<BTreeSet<_>>()
                })
                .unwrap_or_default();
        }
        for dir in &actual_gitkeep_dirs {
            if !allowlisted_dirs.contains(dir) {
                violations.push(violation(
                    "OPS_PLACEHOLDER_DIR_NOT_ALLOWLISTED",
                    format!(
                        "placeholder directory `{}` is not declared in `{}`",
                        dir.display(),
                        placeholder_allowlist_rel.display()
                    ),
                    "add the directory to ops/inventory/placeholder-dirs.json or remove `.gitkeep`",
                    Some(placeholder_allowlist_rel),
                ));
            }
        }
        for dir in &allowlisted_dirs {
            if !actual_gitkeep_dirs.contains(dir) {
                violations.push(violation(
                    "OPS_PLACEHOLDER_DIR_STALE_ALLOWLIST_ENTRY",
                    format!(
                        "allowlisted placeholder directory `{}` has no `.gitkeep` directory",
                        dir.display()
                    ),
                    "remove stale placeholder allowlist entries or recreate the directory with `.gitkeep`",
                    Some(placeholder_allowlist_rel),
                ));
            }
        }
    }

    let placeholder_report_rel = Path::new("ops/_generated.example/placeholder-dirs-report.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, placeholder_report_rel) {
        violations.push(violation(
            "OPS_PLACEHOLDER_DIR_REPORT_MISSING",
            format!(
                "missing placeholder directory report `{}`",
                placeholder_report_rel.display()
            ),
            "generate and commit ops/_generated.example/placeholder-dirs-report.json",
            Some(placeholder_report_rel),
        ));
    } else {
        let report_text = fs::read_to_string(ctx.repo_root.join(placeholder_report_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let report_json: serde_json::Value =
            serde_json::from_str(&report_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        if report_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_PLACEHOLDER_DIR_REPORT_BLOCKING",
                "placeholder-dirs-report.json status is not `pass`".to_string(),
                "resolve placeholder directory drift and regenerate placeholder-dirs-report.json",
                Some(placeholder_report_rel),
            ));
        }
        let report_dirs = report_json
            .get("placeholder_dirs")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str())
                    .map(PathBuf::from)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        if report_dirs != actual_gitkeep_dirs {
            violations.push(violation(
                "OPS_PLACEHOLDER_DIR_REPORT_DRIFT",
                "placeholder-dirs-report.json does not match current ops .gitkeep directory set"
                    .to_string(),
                "regenerate placeholder-dirs-report.json with deterministic sorted placeholder directories",
                Some(placeholder_report_rel),
            ));
        }
        if report_json.get("placeholder_debt_score").is_none() {
            violations.push(violation(
                "OPS_PLACEHOLDER_DIR_REPORT_DEBT_SCORE_MISSING",
                "placeholder-dirs-report.json must include placeholder_debt_score".to_string(),
                "add placeholder_debt_score metrics to the placeholder directory report",
                Some(placeholder_report_rel),
            ));
        }
    }

    let inventory_meta_allowed = [
        Path::new("ops/inventory/meta/contracts.json"),
        Path::new("ops/inventory/meta/error-registry.json"),
        Path::new("ops/inventory/meta/layer-contract.json"),
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    for file in walk_files(&ctx.repo_root.join("ops/inventory/meta")) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        if !inventory_meta_allowed.contains(rel) {
            violations.push(violation(
                "OPS_INVENTORY_META_UNKNOWN_FILE",
                format!(
                    "unexpected file in tight inventory meta surface: `{}`",
                    rel.display()
                ),
                "remove unknown file or update tight inventory meta contract",
                Some(rel),
            ));
        }
    }

    for file in walk_files(&ctx.repo_root.join("ops/schema")) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        let ext = rel.extension().and_then(|v| v.to_str()).unwrap_or("");
        if rel.starts_with(Path::new("ops/schema/generated")) {
            continue;
        }
        let is_allowed_doc = matches!(
            rel.file_name().and_then(|n| n.to_str()),
            Some("README.md" | "OWNER.md" | "REQUIRED_FILES.md" | "INDEX.md" | ".gitkeep")
        );
        let is_schema = ext == "json"
            && rel
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".schema.json"));
        if !is_allowed_doc && !is_schema {
            violations.push(violation(
                "OPS_SCHEMA_UNKNOWN_FILE",
                format!(
                    "unexpected file in tight schema surface: `{}`",
                    rel.display()
                ),
                "keep ops/schema constrained to .schema.json and canonical docs only",
                Some(rel),
            ));
        }
    }

    let ops_root = ctx.repo_root.join("ops");
    for generated_dir in walk_files(&ops_root)
        .into_iter()
        .filter_map(|p| p.parent().map(PathBuf::from))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|p| p.ends_with("generated"))
    {
        let rel_dir = generated_dir
            .strip_prefix(ctx.repo_root)
            .unwrap_or(generated_dir.as_path());
        let readme_rel = rel_dir.join("README.md");
        if !ctx.adapters.fs.exists(ctx.repo_root, &readme_rel) {
            violations.push(violation(
                "OPS_GENERATED_DIRECTORY_README_MISSING",
                format!(
                    "generated directory `{}` is missing README.md",
                    rel_dir.display()
                ),
                "add README.md with generated-only contract in each ops/**/generated directory",
                Some(rel_dir),
            ));
        }
        let domain_root = rel_dir.parent().unwrap_or(Path::new("ops"));
        let domain_required_rel = domain_root.join("REQUIRED_FILES.md");
        if ctx.adapters.fs.exists(ctx.repo_root, &domain_required_rel) {
            let required_text = fs::read_to_string(ctx.repo_root.join(&domain_required_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let rel_dir_str = rel_dir.display().to_string();
            if !required_text.contains(&rel_dir_str) {
                violations.push(violation(
                    "OPS_GENERATED_DIRECTORY_NOT_DECLARED",
                    format!(
                        "generated directory `{}` is not declared in `{}`",
                        rel_dir.display(),
                        domain_required_rel.display()
                    ),
                    "declare generated directory in domain REQUIRED_FILES.md",
                    Some(&domain_required_rel),
                ));
            }
        }
        for file in walk_files(&generated_dir) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            if rel.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let text =
                fs::read_to_string(&file).map_err(|err| CheckError::Failed(err.to_string()))?;
            let value: serde_json::Value =
                serde_json::from_str(&text).map_err(|err| CheckError::Failed(err.to_string()))?;
            if value.get("generated_by").is_none() {
                violations.push(violation(
                    "OPS_GENERATED_METADATA_MISSING",
                    format!(
                        "generated artifact `{}` must include `generated_by` metadata",
                        rel.display()
                    ),
                    "add generated_by field to generated JSON artifacts",
                    Some(rel),
                ));
            }
            if value.get("schema_version").is_none() {
                violations.push(violation(
                    "OPS_GENERATED_SCHEMA_VERSION_MISSING",
                    format!(
                        "generated artifact `{}` must include `schema_version` metadata",
                        rel.display()
                    ),
                    "add schema_version field to generated JSON artifacts",
                    Some(rel),
                ));
            }
        }
    }

    Ok(violations)
}

