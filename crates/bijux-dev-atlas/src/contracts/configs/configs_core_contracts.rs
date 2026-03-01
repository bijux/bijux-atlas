fn test_configs_001_root_surface(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-001",
                "configs.root.only_root_docs",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let actual = root_config_files(&index);
    let expected = index.root_files;
    if actual == expected {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "CONFIGS-001",
            "configs.root.only_root_docs",
            "configs",
            format!("expected root files {expected:?}, found {actual:?}"),
        )])
    }
}

fn test_configs_002_no_undocumented_files(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-002",
                "configs.registry.no_undocumented_files",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut covered = index.root_files.clone();
    for files in index.group_files.values() {
        covered.extend(files.all());
    }
    let missing = config_files_without_exclusions(&index)
        .into_iter()
        .filter(|file| !covered.contains(file))
        .filter(|file| !is_allowed_domain_markdown(file))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(
            missing
                .into_iter()
                .map(|file| {
                    violation(
                        "CONFIGS-002",
                        "configs.registry.no_undocumented_files",
                        &file,
                        "config file is not covered by configs/inventory/configs.json",
                    )
                })
                .collect(),
        )
    }
}

fn test_configs_003_depth_budget(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-003",
                "configs.layout.depth_budget",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let violations = config_files_without_exclusions(&index)
        .into_iter()
        .filter(|file| path_depth(file) > index.registry.max_depth)
        .map(|file| {
            violation(
                "CONFIGS-003",
                "configs.layout.depth_budget",
                &file,
                format!(
                    "path depth {} exceeds configs max_depth {}",
                    path_depth(&file),
                    index.registry.max_depth
                ),
            )
        })
        .collect::<Vec<_>>();
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_004_internal_naming(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-004",
                "configs.naming.internal_surface",
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
        for file in files.public.intersection(&files.internal) {
            violations.push(violation(
                "CONFIGS-004",
                "configs.naming.internal_surface",
                file,
                "a config file cannot be both public and internal",
            ));
        }
        for file in &files.public {
            let name = Path::new(file)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("");
            if name.starts_with('_') {
                violations.push(violation(
                    "CONFIGS-004",
                    "configs.naming.internal_surface",
                    file,
                    "internal-looking config file cannot be classified as public",
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

fn test_configs_005_owner_complete(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-005",
                "configs.registry.owner_complete",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let violations = index
        .registry
        .groups
        .iter()
        .filter(|group| group.owner.trim().is_empty())
        .map(|group| {
            violation(
                "CONFIGS-005",
                "configs.registry.owner_complete",
                REGISTRY_PATH,
                format!("group `{}` is missing an owner", group.name),
            )
        })
        .collect::<Vec<_>>();
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_006_schema_coverage(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => return fail("CONFIGS-006", "configs.schema.coverage", REGISTRY_PATH, err),
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        let files = index
            .group_files
            .get(&group.name)
            .cloned()
            .unwrap_or_default();
        let has_json = files
            .all()
            .into_iter()
            .any(|file| json_like(&file) && !schema_like(&file));
        if has_json && group.schemas.is_empty() {
            violations.push(violation(
                "CONFIGS-006",
                "configs.schema.coverage",
                REGISTRY_PATH,
                format!(
                    "group `{}` contains json configs but declares no schemas",
                    group.name
                ),
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
                    "CONFIGS-006",
                    "configs.schema.coverage",
                    schema,
                    format!("declared schema for group `{}` does not exist", group.name),
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

fn test_configs_007_lockfiles(ctx: &RunContext) -> TestResult {
    let required_pairs = [
        (
            "configs/docs/package.json",
            "configs/docs/package-lock.json",
        ),
        (
            "configs/docs/requirements.txt",
            "configs/docs/requirements.lock.txt",
        ),
    ];
    let mut violations = Vec::new();
    for (source, lockfile) in required_pairs {
        if ctx.repo_root.join(source).is_file() && !ctx.repo_root.join(lockfile).is_file() {
            violations.push(violation(
                "CONFIGS-007",
                "configs.lockfiles.required_pairs",
                lockfile,
                format!("lockfile is required when `{source}` exists"),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_008_no_overlap(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-008",
                "configs.registry.no_overlap",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut owners = BTreeMap::<String, Vec<String>>::new();
    for (group, files) in &index.group_files {
        for file in files.all() {
            owners.entry(file).or_default().push(group.clone());
        }
    }
    let violations = owners
        .into_iter()
        .filter(|(_, groups)| groups.len() > 1)
        .map(|(file, groups)| {
            violation(
                "CONFIGS-008",
                "configs.registry.no_overlap",
                &file,
                format!("file is claimed by multiple groups: {groups:?}"),
            )
        })
        .collect::<Vec<_>>();
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_configs_009_generated_boundary(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-009",
                "configs.generated.authored_boundary",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let mut violations = Vec::new();
    for group in &index.registry.groups {
        for pattern in &group.generated_files {
            if !pattern.contains("/_generated/")
                && !pattern.contains("/_generated.")
                && !pattern.contains("/schema/generated/")
            {
                violations.push(violation(
                    "CONFIGS-009",
                    "configs.generated.authored_boundary",
                    REGISTRY_PATH,
                    format!(
                        "generated pattern `{pattern}` for group `{}` must live under an _generated surface",
                        group.name
                    ),
                ));
            }
        }
        let files = index
            .group_files
            .get(&group.name)
            .cloned()
            .unwrap_or_default();
        for file in files.generated {
            if !file.contains("/_generated/")
                && !file.contains("/_generated.")
                && !file.contains("/schema/generated/")
            {
                violations.push(violation(
                    "CONFIGS-009",
                    "configs.generated.authored_boundary",
                    &file,
                    "generated configs must live under an _generated surface",
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

fn test_configs_010_no_policy_theater(ctx: &RunContext) -> TestResult {
    let index = match registry_index(&ctx.repo_root) {
        Ok(index) => index,
        Err(err) => {
            return fail(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                REGISTRY_PATH,
                err,
            )
        }
    };
    let surface = match read_contract_surface(&ctx.repo_root) {
        Ok(surface) => surface,
        Err(err) => {
            return fail(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                CONTRACT_SURFACE_PATH,
                err,
            )
        }
    };
    if surface.schema_version != 1 {
        return fail(
            "CONFIGS-010",
            "configs.contracts.no_policy_theater",
            CONTRACT_SURFACE_PATH,
            format!(
                "unsupported configs contract registry schema_version {}",
                surface.schema_version
            ),
        );
    }
    if surface.domain != "configs" {
        return fail(
            "CONFIGS-010",
            "configs.contracts.no_policy_theater",
            CONTRACT_SURFACE_PATH,
            format!(
                "configs contract registry must declare domain `configs`, found `{}`",
                surface.domain
            ),
        );
    }
    let expected = (1..=49)
        .map(|n| format!("CFG-{n:03}"))
        .collect::<BTreeSet<_>>();
    let actual = surface
        .contracts
        .iter()
        .map(|row| row.id.clone())
        .collect::<BTreeSet<_>>();
    let executable_tests = contracts(&ctx.repo_root)
        .unwrap_or_default()
        .into_iter()
        .flat_map(|contract| contract.tests.into_iter().map(|test| test.id.0))
        .collect::<BTreeSet<_>>();
    let allowed_severities = ["blocker", "must", "should"];
    let allowed_types = ["static", "filelayout", "schema", "drift", "supplychain"];
    let mut violations = Vec::new();
    if actual != expected {
        violations.push(violation(
            "CONFIGS-010",
            "configs.contracts.no_policy_theater",
            CONTRACT_SURFACE_PATH,
            format!(
                "expected configs contract ids {:?}, found {:?}",
                expected, actual
            ),
        ));
    }
    let _ = index.contract_surface_ids;
    for row in &surface.contracts {
        if row.title.trim().is_empty() {
            violations.push(violation(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                CONTRACT_SURFACE_PATH,
                format!("contract `{}` is missing a title", row.id),
            ));
        }
        if !allowed_severities.contains(&row.severity.as_str()) {
            violations.push(violation(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                CONTRACT_SURFACE_PATH,
                format!(
                    "contract `{}` uses unsupported severity `{}`",
                    row.id, row.severity
                ),
            ));
        }
        if !allowed_types.contains(&row.contract_type.as_str()) {
            violations.push(violation(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                CONTRACT_SURFACE_PATH,
                format!(
                    "contract `{}` uses unsupported contract_type `{}`",
                    row.id, row.contract_type
                ),
            ));
        }
        if row.rationale.trim().is_empty() {
            violations.push(violation(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                CONTRACT_SURFACE_PATH,
                format!("contract `{}` is missing a rationale", row.id),
            ));
        }
        if row.enforced_by.command != "bijux dev atlas contracts configs" {
            violations.push(violation(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                CONTRACT_SURFACE_PATH,
                format!(
                    "contract `{}` must use `bijux dev atlas contracts configs` as its enforcement command",
                    row.id
                ),
            ));
        }
        if !executable_tests.contains(&row.enforced_by.test_id) {
            violations.push(violation(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                CONTRACT_SURFACE_PATH,
                format!(
                    "contract `{}` references unknown enforcement test `{}`",
                    row.id, row.enforced_by.test_id
                ),
            ));
        }
        if row.touched_paths.is_empty() {
            violations.push(violation(
                "CONFIGS-010",
                "configs.contracts.no_policy_theater",
                CONTRACT_SURFACE_PATH,
                format!("contract `{}` must declare touched_paths", row.id),
            ));
        }
        for path in &row.touched_paths {
            if !path.starts_with("configs/") && !path.starts_with("artifacts/") {
                violations.push(violation(
                    "CONFIGS-010",
                    "configs.contracts.no_policy_theater",
                    CONTRACT_SURFACE_PATH,
                    format!(
                        "contract `{}` has touched path `{path}` outside configs or artifacts",
                        row.id
                    ),
                ));
            }
        }
        if let Some(artifact) = &row.evidence_artifact {
            if !artifact.starts_with("artifacts/") {
                violations.push(violation(
                    "CONFIGS-010",
                    "configs.contracts.no_policy_theater",
                    CONTRACT_SURFACE_PATH,
                    format!(
                        "contract `{}` has evidence_artifact `{artifact}` outside artifacts/",
                        row.id
                    ),
                ));
            }
        }
    }
    let mapped_enforcement_checks = surface
        .contracts
        .iter()
        .map(|row| row.enforced_by.test_id.clone())
        .collect::<BTreeSet<_>>();
    let unmapped_executable_checks = executable_tests
        .difference(&mapped_enforcement_checks)
        .cloned()
        .collect::<Vec<_>>();
    if !unmapped_executable_checks.is_empty() {
        violations.push(violation(
            "CONFIGS-010",
            "configs.contracts.no_policy_theater",
            CONTRACT_SURFACE_PATH,
            format!(
                "configs contract registry leaves executable checks unmapped: {:?}",
                unmapped_executable_checks
            ),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}
