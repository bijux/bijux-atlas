fn test_shell_policy(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let policy = dctx
        .policy
        .get("shell_policy")
        .and_then(|v| v.as_str())
        .unwrap_or("forbid");
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        let shell_count = instructions.iter().filter(|ins| ins.keyword == "SHELL").count();
        match policy {
            "required" if shell_count == 0 => violations.push(violation(
                "DOCKER-019",
                "docker.shell.explicit_policy",
                Some(rel.clone()),
                Some(1),
                "Dockerfile must declare SHELL explicitly",
                None,
            )),
            "forbid" if shell_count > 0 => violations.push(violation(
                "DOCKER-019",
                "docker.shell.explicit_policy",
                Some(rel.clone()),
                Some(1),
                "Dockerfile must not declare SHELL when shell_policy=forbid",
                None,
            )),
            _ => {}
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_package_manager_cleanup(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword != "RUN" {
                continue;
            }
            let args = ins.args.to_ascii_lowercase();
            if args.contains("apk add") && !args.contains("--no-cache") {
                violations.push(violation(
                    "DOCKER-020",
                    "docker.run.package_manager_cleanup",
                    Some(rel.clone()),
                    Some(ins.line),
                    "apk add requires --no-cache",
                    Some(ins.args.clone()),
                ));
            }
            if args.contains("apt-get install") && !args.contains("rm -rf /var/lib/apt/lists/*") {
                violations.push(violation(
                    "DOCKER-020",
                    "docker.run.package_manager_cleanup",
                    Some(rel.clone()),
                    Some(ins.line),
                    "apt-get install requires apt lists cleanup in the same RUN instruction",
                    Some(ins.args),
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

fn test_runtime_non_root(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let exceptions = dctx
        .policy
        .get("allow_root_runtime_images")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect::<BTreeSet<_>>();
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        let Some((start, end)) = final_stage_bounds(&instructions) else {
            continue;
        };
        let final_from = parse_from_ref(&instructions[start].args).unwrap_or_default();
        if exceptions.contains(&final_from) {
            continue;
        }
        let has_nonroot_user = instructions[start..end].iter().any(|ins| {
            ins.keyword == "USER"
                && !matches!(
                    ins.args.trim().to_ascii_lowercase().as_str(),
                    "" | "root" | "0" | "0:0" | "root:root"
                )
        });
        if !has_nonroot_user {
            violations.push(violation(
                "DOCKER-021",
                "docker.runtime.non_root",
                Some(rel.clone()),
                Some(instructions[start].line),
                "final runtime stage must run as a non-root user",
                Some(final_from),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_final_stage_has_user(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        let Some((start, end)) = final_stage_bounds(&instructions) else {
            continue;
        };
        if !instructions[start..end].iter().any(|ins| ins.keyword == "USER") {
            violations.push(violation(
                "DOCKER-022",
                "docker.final_stage.user_required",
                Some(rel.clone()),
                Some(instructions[start].line),
                "final stage must declare USER",
                None,
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_final_stage_has_workdir(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        let Some((start, end)) = final_stage_bounds(&instructions) else {
            continue;
        };
        if !instructions[start..end].iter().any(|ins| ins.keyword == "WORKDIR") {
            violations.push(violation(
                "DOCKER-023",
                "docker.final_stage.workdir_required",
                Some(rel.clone()),
                Some(instructions[start].line),
                "final stage must declare WORKDIR",
                None,
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_final_stage_has_entrypoint_or_cmd(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        let Some((start, end)) = final_stage_bounds(&instructions) else {
            continue;
        };
        if !instructions[start..end]
            .iter()
            .any(|ins| ins.keyword == "ENTRYPOINT" || ins.keyword == "CMD")
        {
            violations.push(violation(
                "DOCKER-024",
                "docker.final_stage.entrypoint_or_cmd_required",
                Some(rel.clone()),
                Some(instructions[start].line),
                "final stage must declare ENTRYPOINT or CMD",
                None,
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_label_contract_fields(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        let labels = instructions
            .iter()
            .filter(|ins| ins.keyword == "LABEL")
            .map(|ins| ins.args.to_ascii_lowercase())
            .collect::<Vec<_>>();
        let joined = labels.join(" ");
        for required in [
            "org.opencontainers.image.source",
            "org.opencontainers.image.revision",
            "org.opencontainers.image.created",
            "org.opencontainers.image.licenses",
        ] {
            if !joined.contains(required) {
                violations.push(violation(
                    "DOCKER-025",
                    "docker.labels.contract_fields",
                    Some(rel.clone()),
                    Some(1),
                    "required release label is missing",
                    Some(required.to_string()),
                ));
            }
        }
        let build_date_valid = instructions.iter().any(|ins| {
            ins.keyword == "ARG"
                && ins.args.starts_with("BUILD_DATE=")
                && ins.args.contains('T')
                && ins.args.ends_with('Z')
        });
        if !build_date_valid {
            violations.push(violation(
                "DOCKER-025",
                "docker.labels.contract_fields",
                Some(rel.clone()),
                Some(1),
                "BUILD_DATE default must use an RFC3339 UTC timestamp format",
                None,
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_copy_no_secrets(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    let forbidden = ["id_rsa", ".env", ".pem"];
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword != "COPY" {
                continue;
            }
            for src in extract_copy_sources(&ins.args) {
                if forbidden.iter().any(|pattern| src.contains(pattern)) {
                    violations.push(violation(
                        "DOCKER-026",
                        "docker.copy.no_secrets",
                        Some(rel.clone()),
                        Some(ins.line),
                        "COPY must not include secret-like files",
                        Some(src),
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

fn test_add_forbidden(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let exceptions = dctx
        .policy
        .get("allow_add_exceptions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect::<BTreeSet<_>>();
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword == "ADD" && !exceptions.contains(&rel) {
                violations.push(violation(
                    "DOCKER-027",
                    "docker.add.forbidden",
                    Some(rel.clone()),
                    Some(ins.line),
                    "ADD is forbidden; use COPY unless explicitly allowlisted",
                    Some(ins.args),
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

fn test_compiling_images_are_multistage(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        let compiles = instructions.iter().any(|ins| {
            ins.keyword == "RUN"
                && (ins.args.contains("cargo build")
                    || ins.args.contains("go build")
                    || ins.args.contains("npm run build")
                    || ins.args.contains("pip wheel"))
        });
        let from_count = instructions.iter().filter(|ins| ins.keyword == "FROM").count();
        let has_builder_alias = instructions.iter().any(|ins| {
            ins.keyword == "FROM" && ins.args.to_ascii_lowercase().contains(" as builder")
        });
        if compiles && (from_count < 2 || !has_builder_alias) {
            violations.push(violation(
                "DOCKER-028",
                "docker.build.multistage_required",
                Some(rel.clone()),
                Some(1),
                "images that compile artifacts must use a multi-stage build with a builder stage",
                None,
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_dockerignore_required_entries(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join(".dockerignore");
    let text = match std::fs::read_to_string(&path) {
        Ok(v) => v,
        Err(e) => {
            return TestResult::Fail(vec![violation(
                "DOCKER-029",
                "docker.ignore.required_entries",
                Some(".dockerignore".to_string()),
                Some(1),
                &format!(".dockerignore is required: {e}"),
                None,
            )]);
        }
    };
    let mut violations = Vec::new();
    for required in [".git", "artifacts", "target"] {
        if !text.contains(required) {
            violations.push(violation(
                "DOCKER-029",
                "docker.ignore.required_entries",
                Some(".dockerignore".to_string()),
                Some(1),
                "required .dockerignore entry is missing",
                Some(required.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_repro_build_args_present(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        let args = arg_defaults(&instructions);
        for required in ["SOURCE_DATE_EPOCH", "BUILD_DATE"] {
            if !args.contains_key(required) {
                violations.push(violation(
                    "DOCKER-030",
                    "docker.args.repro_build_args",
                    Some(rel.clone()),
                    Some(1),
                    "required reproducible build ARG is missing",
                    Some(required.to_string()),
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

fn test_no_network_in_final_stage(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        let Some((start, end)) = final_stage_bounds(&instructions) else {
            continue;
        };
        for ins in &instructions[start..end] {
            if ins.keyword == "RUN" {
                let args = ins.args.to_ascii_lowercase();
                if args.contains("curl ") || args.contains("wget ") {
                    violations.push(violation(
                        "DOCKER-031",
                        "docker.final_stage.no_network",
                        Some(rel.clone()),
                        Some(ins.line),
                        "final stage must not fetch over the network",
                        Some(ins.args.clone()),
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

fn test_no_package_manager_in_final_stage(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        let Some((start, end)) = final_stage_bounds(&instructions) else {
            continue;
        };
        for ins in &instructions[start..end] {
            if ins.keyword == "RUN" {
                let args = ins.args.to_ascii_lowercase();
                if ["apt-get", "apt ", "apk add", "yum ", "dnf "]
                    .iter()
                    .any(|token| args.contains(token))
                {
                    violations.push(violation(
                        "DOCKER-032",
                        "docker.final_stage.no_package_manager",
                        Some(rel.clone()),
                        Some(ins.line),
                        "final stage must not run package managers",
                        Some(ins.args.clone()),
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

fn test_images_have_smoke_manifest(ctx: &RunContext) -> TestResult {
    let manifest = match load_images_manifest(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut declared = BTreeSet::new();
    let mut violations = Vec::new();
    for image in manifest["images"].as_array().cloned().unwrap_or_default() {
        let name = image
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let dockerfile = image
            .get("dockerfile")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let smoke = image
            .get("smoke")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if name.is_empty() || dockerfile.is_empty() || smoke.is_empty() {
            violations.push(violation(
                "DOCKER-033",
                "docker.images.smoke_manifest",
                Some("docker/images.manifest.json".to_string()),
                Some(1),
                "each image manifest entry must include name, dockerfile, and non-empty smoke command",
                Some(name),
            ));
            continue;
        }
        declared.insert(dockerfile);
    }
    let discovered = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    for (rel, _) in discovered {
        if !declared.contains(&rel) {
            violations.push(violation(
                "DOCKER-033",
                "docker.images.smoke_manifest",
                Some("docker/images.manifest.json".to_string()),
                Some(1),
                "each Dockerfile must have a smoke manifest entry",
                Some(rel),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_images_manifest_schema_valid(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join("docker/images.manifest.json");
    let manifest = match load_images_manifest(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Fail(vec![violation("DOCKER-034", "docker.images.manifest_schema_valid", Some("docker/images.manifest.json".to_string()), Some(1), &e, None)]),
    };
    let mut violations = Vec::new();
    if manifest.get("schema_version").and_then(|v| v.as_u64()) != Some(1) {
        violations.push(violation("DOCKER-034", "docker.images.manifest_schema_valid", Some("docker/images.manifest.json".to_string()), Some(1), "images manifest must set schema_version=1", None));
    }
    let images = manifest.get("images").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    if images.is_empty() {
        violations.push(violation("DOCKER-034", "docker.images.manifest_schema_valid", Some("docker/images.manifest.json".to_string()), Some(1), "images manifest must declare at least one image", None));
    }
    for image in images {
        for field in ["name", "dockerfile", "context"] {
            if image.get(field).and_then(|v| v.as_str()).is_none() {
                violations.push(violation("DOCKER-034", "docker.images.manifest_schema_valid", Some(path.display().to_string().replace('\\', "/").split("/docker/").last().map(|v| format!("docker/{v}")).unwrap_or_else(|| "docker/images.manifest.json".to_string())), Some(1), "manifest image entry is missing a required field", Some(field.to_string())));
            }
        }
        if image.get("smoke").and_then(|v| v.as_array()).is_none() {
            violations.push(violation("DOCKER-034", "docker.images.manifest_schema_valid", Some("docker/images.manifest.json".to_string()), Some(1), "manifest image entry must declare smoke as an array", None));
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_images_manifest_matches_dockerfiles(ctx: &RunContext) -> TestResult {
    let manifest = match load_images_manifest(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let listed = manifest["images"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|image| image.get("dockerfile").and_then(|v| v.as_str()).map(ToString::to_string))
        .collect::<BTreeSet<_>>();
    let discovered = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v.into_iter().map(|(rel, _)| rel).collect::<BTreeSet<_>>(),
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for rel in &listed {
        if !ctx.repo_root.join(rel).exists() {
            violations.push(violation("DOCKER-035", "docker.images.manifest_matches_dockerfiles", Some("docker/images.manifest.json".to_string()), Some(1), "manifest references a missing Dockerfile", Some(rel.clone())));
        }
    }
    for rel in &discovered {
        if !listed.contains(rel) {
            violations.push(violation("DOCKER-035", "docker.images.manifest_matches_dockerfiles", Some("docker/images.manifest.json".to_string()), Some(1), "manifest is missing a Dockerfile entry", Some(rel.clone())));
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_build_matrix_defined(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join("docker/build-matrix.json");
    let payload = match load_json(&path) {
        Ok(v) => v,
        Err(e) => return TestResult::Fail(vec![violation("DOCKER-036", "docker.build_matrix.defined", Some("docker/build-matrix.json".to_string()), Some(1), &e, None)]),
    };
    let manifest = match load_images_manifest(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let manifest_names = manifest["images"].as_array().cloned().unwrap_or_default().into_iter().filter_map(|image| image.get("name").and_then(|v| v.as_str()).map(ToString::to_string)).collect::<BTreeSet<_>>();
    let rows = payload["images"].as_array().cloned().unwrap_or_default();
    let mut matrix_names = BTreeSet::new();
    let mut violations = Vec::new();
    if payload.get("schema_version").and_then(|v| v.as_u64()) != Some(1) {
        violations.push(violation("DOCKER-036", "docker.build_matrix.defined", Some("docker/build-matrix.json".to_string()), Some(1), "build matrix must set schema_version=1", None));
    }
    for row in rows {
        let Some(name) = row.get("name").and_then(|v| v.as_str()) else {
            violations.push(violation("DOCKER-036", "docker.build_matrix.defined", Some("docker/build-matrix.json".to_string()), Some(1), "build matrix entry missing name", None));
            continue;
        };
        matrix_names.insert(name.to_string());
        for field in ["platforms", "tags", "outputs"] {
            if row.get(field).and_then(|v| v.as_array()).is_none() {
                violations.push(violation("DOCKER-036", "docker.build_matrix.defined", Some("docker/build-matrix.json".to_string()), Some(1), "build matrix entry missing required array field", Some(format!("{name}:{field}"))));
            }
        }
    }
    for name in manifest_names {
        if !matrix_names.contains(&name) {
            violations.push(violation("DOCKER-036", "docker.build_matrix.defined", Some("docker/build-matrix.json".to_string()), Some(1), "build matrix must cover every manifest image", Some(name)));
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

