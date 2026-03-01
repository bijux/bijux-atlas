fn test_configs_026_docs_markdown_removed(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-026",
                "configs.docs.no_nested_markdown",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let violations = config_files_without_exclusions(&index)
        .into_iter()
        .filter(|file| file.ends_with(".md"))
        .filter(|file| !ROOT_MARKDOWN_FILES.contains(&file.as_str()))
        .filter(|file| !is_allowed_domain_markdown(file))
        .map(|file| {
            violation(
                "CONFIGS-026",
                "configs.docs.no_nested_markdown",
                &file,
                "configs keeps markdown only at the root authority surface; move narrative markdown into docs/",
            )
        })
        .collect::<Vec<_>>();
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_027_docs_tooling_surface(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-027",
                "configs.docs.tooling_surface",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for file in config_files_without_exclusions(&index) {
        if !file.starts_with("configs/docs/") {
            continue;
        }
        if is_allowed_domain_markdown(&file) {
            continue;
        }
        if !DOCS_TOOLING_PATTERNS
            .iter()
            .any(|pattern| wildcard_match(pattern, &file))
        {
            violations.push(violation(
                "CONFIGS-027",
                "configs.docs.tooling_surface",
                &file,
                "configs/docs contains a file outside the declared tooling surface",
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_038_domain_landing_docs(ctx: &RunContext) -> TestResult {
    let domains = match configs_top_level_domain_dirs(&ctx.repo_root) {
        Ok(domains) => domains,
        Err(err) => {
            return fail(
                "CONFIGS-038",
                "configs.docs.domain_landing_files",
                "configs",
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for domain in domains {
        let candidates = [
            format!("configs/{domain}/README.md"),
            format!("configs/{domain}/INDEX.md"),
            format!("configs/{domain}/index.md"),
        ];
        let matches = candidates
            .iter()
            .filter(|path| ctx.repo_root.join(path).is_file())
            .count();
        if matches != 1 {
            violations.push(violation(
                "CONFIGS-038",
                "configs.docs.domain_landing_files",
                &format!("configs/{domain}"),
                format!(
                    "configs domain `{domain}` must contain exactly one landing markdown file, found {matches}"
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

fn test_configs_039_top_level_domain_policy(ctx: &RunContext) -> TestResult {
    let policy = match read_config_groups_policy(&ctx.repo_root) {
        Ok(policy) => policy,
        Err(err) => {
            return fail(
                "CONFIGS-039",
                "configs.layout.top_level_domain_policy",
                "configs/inventory/groups.json",
                err,
            )
        }
    };
    if policy.schema_version != 1 {
        return fail(
            "CONFIGS-039",
            "configs.layout.top_level_domain_policy",
            "configs/inventory/groups.json",
            format!(
                "unsupported configs groups policy schema_version {}",
                policy.schema_version
            ),
        );
    }
    let actual = match configs_top_level_domain_dirs(&ctx.repo_root) {
        Ok(actual) => actual,
        Err(err) => return fail("CONFIGS-039", "configs.layout.top_level_domain_policy", "configs", err),
    };
    let actual_set = actual.iter().cloned().collect::<BTreeSet<_>>();
    let allowed_set = policy.allowed_groups.iter().cloned().collect::<BTreeSet<_>>();
    let mut violations = Vec::new();
    if actual.len() > policy.max_top_level_dirs {
        violations.push(violation(
            "CONFIGS-039",
            "configs.layout.top_level_domain_policy",
            "configs/inventory/groups.json",
            format!(
                "configs exposes {} top-level domains and exceeds max_top_level_dirs {}",
                actual.len(),
                policy.max_top_level_dirs
            ),
        ));
    }
    for missing in allowed_set.difference(&actual_set) {
        violations.push(violation(
            "CONFIGS-039",
            "configs.layout.top_level_domain_policy",
            "configs/inventory/groups.json",
            format!("allowed top-level domain `{missing}` is missing from configs/"),
        ));
    }
    for extra in actual_set.difference(&allowed_set) {
        violations.push(violation(
            "CONFIGS-039",
            "configs.layout.top_level_domain_policy",
            "configs/inventory/groups.json",
            format!("configs contains undeclared top-level domain `{extra}`"),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_040_unique_domain_filenames(ctx: &RunContext) -> TestResult {
    let allowed_duplicates = ["env.schema.json", "lanes.json", "owners.json"];
    let domains = match configs_top_level_domain_dirs(&ctx.repo_root) {
        Ok(domains) => domains,
        Err(err) => {
            return fail(
                "CONFIGS-040",
                "configs.naming.unique_domain_filenames",
                "configs",
                err,
            )
        }
    };
    let mut seen = BTreeMap::<String, Vec<String>>::new();
    for domain in domains {
        let dir = ctx.repo_root.join("configs").join(&domain);
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(err) => {
                return fail(
                    "CONFIGS-040",
                    "configs.naming.unique_domain_filenames",
                    &format!("configs/{domain}"),
                    format!("read failed: {err}"),
                )
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if matches!(name, "README.md" | "INDEX.md" | "index.md") {
                continue;
            }
            if allowed_duplicates.contains(&name) {
                continue;
            }
            seen.entry(name.to_string())
                .or_default()
                .push(format!("configs/{domain}/{name}"));
        }
    }
    let mut violations = Vec::new();
    for (name, mut paths) in seen {
        if paths.len() <= 1 {
            continue;
        }
        paths.sort();
        for path in paths {
            violations.push(violation(
                "CONFIGS-040",
                "configs.naming.unique_domain_filenames",
                &path,
                format!("config filename `{name}` is duplicated across top-level domains"),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_028_owner_map_alignment(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-028",
                "configs.owners.group_alignment",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let owners = match read_owners(&ctx.repo_root) {
        Ok(owners) => owners,
        Err(err) => {
            return fail(
                "CONFIGS-028",
                "configs.owners.group_alignment",
                OWNERS_PATH,
                err,
            )
        }
    };
    if owners.schema_version != 1 {
        return fail(
            "CONFIGS-028",
            "configs.owners.group_alignment",
            OWNERS_PATH,
            format!(
                "unsupported configs owner map schema_version {}",
                owners.schema_version
            ),
        );
    }
    let mut violations = Vec::new();
    let expected_groups = index
        .registry
        .groups
        .iter()
        .map(|group| group.name.clone())
        .collect::<BTreeSet<_>>();
    let actual_groups = owners.groups.keys().cloned().collect::<BTreeSet<_>>();
    for missing in expected_groups.difference(&actual_groups) {
        violations.push(violation(
            "CONFIGS-028",
            "configs.owners.group_alignment",
            OWNERS_PATH,
            format!("owner map is missing group `{missing}`"),
        ));
    }
    for extra in actual_groups.difference(&expected_groups) {
        violations.push(violation(
            "CONFIGS-028",
            "configs.owners.group_alignment",
            OWNERS_PATH,
            format!("owner map declares unknown group `{extra}`"),
        ));
    }
    for group in &index.registry.groups {
        match owners.groups.get(&group.name) {
            Some(owner) if owner == &group.owner => {}
            Some(owner) => violations.push(violation(
                "CONFIGS-028",
                "configs.owners.group_alignment",
                OWNERS_PATH,
                format!(
                    "owner map mismatch for `{}`: expected `{}`, found `{owner}`",
                    group.name, group.owner
                ),
            )),
            None => {}
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_029_consumer_map_alignment(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-029",
                "configs.consumers.group_alignment",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let consumers = match read_consumers(&ctx.repo_root) {
        Ok(consumers) => consumers,
        Err(err) => {
            return fail(
                "CONFIGS-029",
                "configs.consumers.group_alignment",
                CONSUMERS_PATH,
                err,
            )
        }
    };
    if consumers.schema_version != 1 {
        return fail(
            "CONFIGS-029",
            "configs.consumers.group_alignment",
            CONSUMERS_PATH,
            format!(
                "unsupported configs consumer map schema_version {}",
                consumers.schema_version
            ),
        );
    }
    let mut violations = Vec::new();
    let expected_groups = index
        .registry
        .groups
        .iter()
        .map(|group| group.name.clone())
        .collect::<BTreeSet<_>>();
    let actual_groups = consumers.groups.keys().cloned().collect::<BTreeSet<_>>();
    for missing in expected_groups.difference(&actual_groups) {
        violations.push(violation(
            "CONFIGS-029",
            "configs.consumers.group_alignment",
            CONSUMERS_PATH,
            format!("consumer map is missing group `{missing}`"),
        ));
    }
    for extra in actual_groups.difference(&expected_groups) {
        violations.push(violation(
            "CONFIGS-029",
            "configs.consumers.group_alignment",
            CONSUMERS_PATH,
            format!("consumer map declares unknown group `{extra}`"),
        ));
    }
    for group in &index.registry.groups {
        if let Some(entries) = consumers.groups.get(&group.name) {
            if entries.is_empty() {
                violations.push(violation(
                    "CONFIGS-029",
                    "configs.consumers.group_alignment",
                    CONSUMERS_PATH,
                    format!(
                        "consumer map for `{}` must list at least one consumer",
                        group.name
                    ),
                ));
            }
        }
    }
    for (pattern, entries) in &consumers.files {
        let matches_public_file = index
            .root_files
            .iter()
            .any(|file| wildcard_match(pattern, file))
            || index.registry.groups.iter().any(|group| {
                let files = index
                    .group_files
                    .get(&group.name)
                    .cloned()
                    .unwrap_or_default();
                files
                    .public
                    .iter()
                    .chain(files.generated.iter())
                    .any(|file| wildcard_match(pattern, file))
            });
        if !matches_public_file {
            violations.push(violation(
                "CONFIGS-029",
                "configs.consumers.group_alignment",
                CONSUMERS_PATH,
                format!(
                    "consumer file pattern `{pattern}` does not match any public or generated config file"
                ),
            ));
        }
        if entries.is_empty() {
            violations.push(violation(
                "CONFIGS-029",
                "configs.consumers.group_alignment",
                CONSUMERS_PATH,
                format!("consumer file pattern `{pattern}` must list at least one consumer"),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_030_file_consumer_coverage(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-030",
                "configs.consumers.file_coverage",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let consumers = match read_consumers(&ctx.repo_root) {
        Ok(consumers) => consumers,
        Err(err) => {
            return fail(
                "CONFIGS-030",
                "configs.consumers.file_coverage",
                CONSUMERS_PATH,
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
        for file in files.public.iter().chain(files.generated.iter()) {
            let matched = matching_file_consumers(&consumers, file);
            if matched.is_empty() {
                violations.push(violation(
                    "CONFIGS-030",
                    "configs.consumers.file_coverage",
                    file,
                    "public or generated config file is missing a per-file consumer declaration",
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

fn test_configs_031_schema_map_coverage(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-031",
                "configs.schemas.file_coverage",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let schemas = match read_schemas(&ctx.repo_root) {
        Ok(schemas) => schemas,
        Err(err) => {
            return fail(
                "CONFIGS-031",
                "configs.schemas.file_coverage",
                SCHEMAS_PATH,
                err,
            )
        }
    };
    if schemas.schema_version != 1 {
        return fail(
            "CONFIGS-031",
            "configs.schemas.file_coverage",
            SCHEMAS_PATH,
            format!(
                "unsupported configs schema map schema_version {}",
                schemas.schema_version
            ),
        );
    }
    let mut violations = Vec::new();
    for (pattern, schema) in &schemas.files {
        let matches_file = index.registry.groups.iter().any(|group| {
            let files = index
                .group_files
                .get(&group.name)
                .cloned()
                .unwrap_or_default();
            files
                .public
                .iter()
                .chain(files.generated.iter())
                .filter(|file| json_like(file) && !schema_like(file))
                .any(|file| wildcard_match(pattern, file))
        }) || index
            .root_files
            .iter()
            .filter(|file| json_like(file) && !schema_like(file))
            .any(|file| wildcard_match(pattern, file));
        if !matches_file {
            violations.push(violation(
                "CONFIGS-031",
                "configs.schemas.file_coverage",
                SCHEMAS_PATH,
                format!(
                    "schema file pattern `{pattern}` does not match any governed json or jsonc config file"
                ),
            ));
        }
        if !ctx.repo_root.join(schema).is_file() {
            violations.push(violation(
                "CONFIGS-031",
                "configs.schemas.file_coverage",
                schema,
                format!("declared schema map target for `{pattern}` does not exist"),
            ));
        }
    }
    for file in index
        .root_files
        .iter()
        .filter(|file| json_like(file) && !schema_like(file))
    {
        if matched_schema_path(&schemas, file).is_none() {
            violations.push(violation(
                "CONFIGS-031",
                "configs.schemas.file_coverage",
                file,
                "root json or jsonc config file is missing a per-file schema declaration",
            ));
        }
    }
    for group in &index.registry.groups {
        let files = index
            .group_files
            .get(&group.name)
            .cloned()
            .unwrap_or_default();
        for file in files.public.iter().chain(files.generated.iter()) {
            if !json_like(file) || schema_like(file) {
                continue;
            }
            if matched_schema_path(&schemas, file).is_none() {
                violations.push(violation(
                    "CONFIGS-031",
                    "configs.schemas.file_coverage",
                    file,
                    "public or generated json or jsonc config file is missing a per-file schema declaration",
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
