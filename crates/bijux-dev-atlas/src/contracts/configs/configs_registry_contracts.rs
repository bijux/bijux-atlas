fn test_configs_011_registry_complete(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-011",
                "configs.registry.complete_surface",
                REGISTRY_PATH,
                err,
            )
        }
    };
    if index.registry.schema_version != 1 {
        return fail(
            "CONFIGS-011",
            "configs.registry.complete_surface",
            REGISTRY_PATH,
            format!(
                "unsupported configs registry schema_version {}",
                index.registry.schema_version
            ),
        );
    }
    let has_root_readme = index.root_files.contains("configs/README.md");
    let has_root_contract = index.root_files.contains("configs/CONTRACT.md");
    if has_root_readme && has_root_contract {
        TestResult::Pass
    } else {
        fail(
            "CONFIGS-011",
            "configs.registry.complete_surface",
            REGISTRY_PATH,
            "configs registry root_files must include configs/README.md and configs/CONTRACT.md",
        )
    }
}

fn test_configs_012_no_orphans(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-012",
                "configs.registry.no_orphans",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut covered = index.root_files.clone();
    for files in index.group_files.values() {
        covered.extend(files.all());
    }
    let orphans = config_files_without_exclusions(&index)
        .into_iter()
        .filter(|file| !covered.contains(file))
        .filter(|file| !is_allowed_domain_markdown(file))
        .collect::<Vec<_>>();
    if orphans.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(
            orphans
                .into_iter()
                .map(|file| {
                    violation(
                        "CONFIGS-012",
                        "configs.registry.no_orphans",
                        &file,
                        "config file is orphaned from the configs registry",
                    )
                })
                .collect(),
        )
    }
}

fn test_configs_013_no_dead_entries(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-013",
                "configs.registry.no_dead_entries",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        let files = index
            .group_files
            .get(&group.name)
            .cloned()
            .unwrap_or_default();
        if !group.public_files.is_empty() && files.public.is_empty() {
            violations.push(violation(
                "CONFIGS-013",
                "configs.registry.no_dead_entries",
                REGISTRY_PATH,
                format!(
                    "group `{}` has public patterns with no matching files",
                    group.name
                ),
            ));
        }
        if !group.internal_files.is_empty() && files.internal.is_empty() {
            violations.push(violation(
                "CONFIGS-013",
                "configs.registry.no_dead_entries",
                REGISTRY_PATH,
                format!(
                    "group `{}` has internal patterns with no matching files",
                    group.name
                ),
            ));
        }
    }
    for item in &index.registry.exclusions {
        let matched = index
            .files
            .iter()
            .any(|file| wildcard_match(&item.pattern, file));
        if !matched {
            violations.push(violation(
                "CONFIGS-013",
                "configs.registry.no_dead_entries",
                REGISTRY_PATH,
                format!(
                    "exclusion `{}` has no matching files ({})",
                    item.pattern, item.reason
                ),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_014_group_budget(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-014",
                "configs.registry.group_budget",
                REGISTRY_PATH,
                err,
            )
        }
    };
    if index.registry.groups.len() <= index.registry.max_groups {
        TestResult::Pass
    } else {
        fail(
            "CONFIGS-014",
            "configs.registry.group_budget",
            REGISTRY_PATH,
            format!(
                "configs registry declares {} groups, which exceeds max_groups {}",
                index.registry.groups.len(),
                index.registry.max_groups
            ),
        )
    }
}

fn test_configs_015_group_depth_budget(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-015",
                "configs.registry.group_depth_budget",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        let files = index
            .group_files
            .get(&group.name)
            .cloned()
            .unwrap_or_default();
        for file in files.all() {
            if let Some(depth) = group_depth(&file, &group.name) {
                if depth > index.registry.max_group_depth {
                    violations.push(violation(
                        "CONFIGS-015",
                        "configs.registry.group_depth_budget",
                        &file,
                        format!(
                            "group depth {} exceeds max_group_depth {} for `{}`",
                            depth, index.registry.max_group_depth, group.name
                        ),
                    ));
                }
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_016_visibility_classification(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-016",
                "configs.registry.visibility_classification",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        if group.stability != "stable" && group.stability != "experimental" {
            violations.push(violation(
                "CONFIGS-016",
                "configs.registry.visibility_classification",
                REGISTRY_PATH,
                format!(
                    "group `{}` has invalid stability `{}`",
                    group.name, group.stability
                ),
            ));
        }
        let files = index
            .group_files
            .get(&group.name)
            .cloned()
            .unwrap_or_default();
        for file in files.all() {
            let classifications = usize::from(files.public.contains(&file))
                + usize::from(files.internal.contains(&file))
                + usize::from(files.generated.contains(&file));
            if classifications != 1 {
                violations.push(violation(
                    "CONFIGS-016",
                    "configs.registry.visibility_classification",
                    &file,
                    "each config file must map to exactly one visibility bucket",
                ));
            }
        }
        if group.public_files.is_empty()
            && group.internal_files.is_empty()
            && group.generated_files.is_empty()
        {
            violations.push(violation(
                "CONFIGS-016",
                "configs.registry.visibility_classification",
                REGISTRY_PATH,
                format!("group `{}` declares no file buckets", group.name),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_017_tool_entrypoints(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-017",
                "configs.registry.tool_entrypoints",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        if group.tool_entrypoints.is_empty() {
            violations.push(violation(
                "CONFIGS-017",
                "configs.registry.tool_entrypoints",
                REGISTRY_PATH,
                format!(
                    "group `{}` must declare at least one tool entrypoint",
                    group.name
                ),
            ));
        }
        for entrypoint in &group.tool_entrypoints {
            if !entrypoint.starts_with("bijux ")
                && !entrypoint.starts_with("make ")
                && !entrypoint.starts_with("cargo ")
            {
                violations.push(violation(
                    "CONFIGS-017",
                    "configs.registry.tool_entrypoints",
                    REGISTRY_PATH,
                    format!(
                        "group `{}` has unsupported tool entrypoint `{entrypoint}`",
                        group.name
                    ),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_018_schema_owner(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-018",
                "configs.registry.schema_owner",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        if group.schema_owner.trim().is_empty() {
            violations.push(violation(
                "CONFIGS-018",
                "configs.registry.schema_owner",
                REGISTRY_PATH,
                format!("group `{}` is missing schema_owner", group.name),
            ));
        }
        for schema in &group.schemas {
            let exists = if schema.contains('*') {
                index.files.iter().any(|file| wildcard_match(schema, file))
            } else {
                ctx.repo_root.join(schema).is_file()
            };
            if !exists {
                violations.push(violation(
                    "CONFIGS-018",
                    "configs.registry.schema_owner",
                    schema,
                    format!(
                        "schema owner `{}` declares missing schema for group `{}`",
                        group.schema_owner, group.name
                    ),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_019_lifecycle(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-019",
                "configs.registry.lifecycle",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        if group.stability != "stable" && group.stability != "experimental" {
            violations.push(violation(
                "CONFIGS-019",
                "configs.registry.lifecycle",
                REGISTRY_PATH,
                format!(
                    "group `{}` has invalid stability `{}`",
                    group.name, group.stability
                ),
            ));
        }
        if group.owner.trim().is_empty() || group.schema_owner.trim().is_empty() {
            violations.push(violation(
                "CONFIGS-019",
                "configs.registry.lifecycle",
                REGISTRY_PATH,
                format!("group `{}` lifecycle metadata is incomplete", group.name),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_020_generated_index_deterministic(ctx: &RunContext) -> TestResult {
    let first = match generated_index_json(&ctx.repo_root) {
        Ok(value) => value,
        Err(err) => {
            return fail(
                "CONFIGS-020",
                "configs.generated_index.deterministic",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let second = match generated_index_json(&ctx.repo_root) {
        Ok(value) => value,
        Err(err) => {
            return fail(
                "CONFIGS-020",
                "configs.generated_index.deterministic",
                REGISTRY_PATH,
                err,
            )
        }
    };
    if first == second {
        TestResult::Pass
    } else {
        fail(
            "CONFIGS-020",
            "configs.generated_index.deterministic",
            "configs/_generated/configs-index.json",
            "generated configs index is not deterministic across consecutive renders",
        )
    }
}

fn test_configs_021_generated_index_matches_committed(ctx: &RunContext) -> TestResult {
    let expected = match generated_index_json(&ctx.repo_root) {
        Ok(value) => value,
        Err(err) => {
            return fail(
                "CONFIGS-021",
                "configs.generated_index.committed_match",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let path = ctx.repo_root.join("configs/_generated/configs-index.json");
    let text = match read_text(&path) {
        Ok(text) => text,
        Err(err) => {
            return fail(
                "CONFIGS-021",
                "configs.generated_index.committed_match",
                "configs/_generated/configs-index.json",
                err,
            )
        }
    };
    let actual = match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(value) => value,
        Err(err) => {
            return fail(
                "CONFIGS-021",
                "configs.generated_index.committed_match",
                "configs/_generated/configs-index.json",
                format!("parse configs/_generated/configs-index.json failed: {err}"),
            )
        }
    };
    if actual == expected {
        TestResult::Pass
    } else {
        fail(
            "CONFIGS-021",
            "configs.generated_index.committed_match",
            "configs/_generated/configs-index.json",
            "committed generated configs index does not match registry render",
        )
    }
}

fn test_configs_022_json_configs_parse(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => return fail("CONFIGS-022", "configs.parse.json", REGISTRY_PATH, err),
    };
    let violations = parse_checked_files(&index, &["json", "jsonc"])
        .into_iter()
        .filter_map(|file| {
            parse_supported_config_file(&ctx.repo_root.join(&file))
                .err()
                .map(|err| violation("CONFIGS-022", "configs.parse.json", &file, err))
        })
        .collect::<Vec<_>>();
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_023_yaml_configs_parse(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => return fail("CONFIGS-023", "configs.parse.yaml", REGISTRY_PATH, err),
    };
    let violations = parse_checked_files(&index, &["yaml", "yml"])
        .into_iter()
        .filter_map(|file| {
            parse_supported_config_file(&ctx.repo_root.join(&file))
                .err()
                .map(|err| violation("CONFIGS-023", "configs.parse.yaml", &file, err))
        })
        .collect::<Vec<_>>();
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_024_toml_configs_parse(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => return fail("CONFIGS-024", "configs.parse.toml", REGISTRY_PATH, err),
    };
    let violations = parse_checked_files(&index, &["toml"])
        .into_iter()
        .filter_map(|file| {
            parse_supported_config_file(&ctx.repo_root.join(&file))
                .err()
                .map(|err| violation("CONFIGS-024", "configs.parse.toml", &file, err))
        })
        .collect::<Vec<_>>();
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_025_text_hygiene(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => return fail("CONFIGS-025", "configs.text.hygiene", REGISTRY_PATH, err),
    };
    let text_exts = ["md", "txt", "toml", "json", "jsonc", "yml", "yaml", "ini"];
    let mut violations = Vec::new();
    let root = ctx.repo_root.join("configs");
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(err) => {
                violations.push(violation(
                    "CONFIGS-025",
                    "configs.text.hygiene",
                    &dir
                        .strip_prefix(&ctx.repo_root)
                        .unwrap_or(&dir)
                        .display()
                        .to_string()
                        .replace('\\', "/"),
                    format!("read {} failed: {err}", dir.display()),
                ));
                continue;
            }
        };
        let mut file_count = 0usize;
        for entry in entries.flatten() {
            let path = entry.path();
            let rel = path
                .strip_prefix(&ctx.repo_root)
                .unwrap_or(&path)
                .display()
                .to_string()
                .replace('\\', "/");
            let meta = match std::fs::symlink_metadata(&path) {
                Ok(meta) => meta,
                Err(err) => {
                    violations.push(violation(
                        "CONFIGS-025",
                        "configs.text.hygiene",
                        &rel,
                        format!("read metadata failed: {err}"),
                    ));
                    continue;
                }
            };
            if meta.file_type().is_symlink() {
                violations.push(violation(
                    "CONFIGS-025",
                    "configs.text.hygiene",
                    &rel,
                    "symlinks are forbidden under configs/",
                ));
                continue;
            }
            if meta.is_dir() {
                stack.push(path);
                continue;
            }
            if !meta.is_file() {
                continue;
            }
            file_count += 1;
            if is_binary_file(&path) {
                violations.push(violation(
                    "CONFIGS-025",
                    "configs.text.hygiene",
                    &rel,
                    "binary files are forbidden under configs/",
                ));
            }
        }
        let rel_dir = dir
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&dir)
            .display()
            .to_string()
            .replace('\\', "/");
        let max_files = directory_file_budget(&rel_dir);
        if file_count > max_files {
            violations.push(violation(
                "CONFIGS-025",
                "configs.text.hygiene",
                &rel_dir,
                format!(
                    "directory has {} direct files and exceeds budget {}",
                    file_count, max_files
                ),
            ));
        }
    }
    for file in config_files_without_exclusions(&index) {
        let ext = Path::new(&file)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if !text_exts.contains(&ext) {
            continue;
        }
        let text = match read_text(&ctx.repo_root.join(&file)) {
            Ok(text) => text,
            Err(err) => {
                violations.push(violation("CONFIGS-025", "configs.text.hygiene", &file, err));
                continue;
            }
        };
        for (line_no, line) in text.lines().enumerate() {
            if line.ends_with(' ') || line.ends_with('\t') {
                violations.push(Violation {
                    contract_id: "CONFIGS-025".to_string(),
                    test_id: "configs.text.hygiene".to_string(),
                    file: Some(file.clone()),
                    line: Some(line_no + 1),
                    message: "trailing whitespace is forbidden in config text files".to_string(),
                    evidence: None,
                });
                break;
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn is_binary_file(path: &Path) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    bytes.iter().take(4096).any(|byte| *byte == 0)
}

fn directory_file_budget(rel_dir: &str) -> usize {
    match rel_dir {
        "configs" => 16,
        "configs/contracts" => 30,
        "configs/docs" => 16,
        "configs/ops" => 40,
        "configs/policy" => 40,
        "configs/schema" => 30,
        _ => 10,
    }
}
