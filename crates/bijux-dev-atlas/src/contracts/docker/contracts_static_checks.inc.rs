fn test_dir_allowed_markdown(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) if e.starts_with("SKIP_MISSING_TOOL:") => {
            return TestResult::Skip(e.replacen("SKIP_MISSING_TOOL: ", "", 1));
        }
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for file in match walk_files(&dctx.docker_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    } {
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        if rel.ends_with(".md") && rel != "docker/README.md" && rel != "docker/CONTRACT.md" {
            violations.push(violation(
                "DOCKER-000",
                "docker.dir.allowed_markdown",
                Some(rel),
                Some(1),
                "only docker/README.md and docker/CONTRACT.md are allowed",
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

fn test_dir_no_contracts_subdir(ctx: &RunContext) -> TestResult {
    let forbidden = ctx.repo_root.join("docker/contracts");
    if forbidden.exists() {
        TestResult::Fail(vec![violation(
            "DOCKER-000",
            "docker.dir.no_contracts_subdir",
            Some("docker/contracts".to_string()),
            Some(1),
            "docker/contracts directory is forbidden",
            None,
        )])
    } else {
        TestResult::Pass
    }
}

fn test_dir_dockerfiles_location(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for df in dctx.dockerfiles {
        let rel = df
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&df)
            .display()
            .to_string();
        if !rel.starts_with("docker/images/") {
            violations.push(violation(
                "DOCKER-000",
                "docker.dir.dockerfiles_location",
                Some(rel),
                Some(1),
                "Dockerfiles must live under docker/images/**",
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

fn test_root_dockerfile_symlink_or_absent(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join("Dockerfile");
    if !path.exists() {
        return TestResult::Pass;
    }
    let meta = match std::fs::symlink_metadata(&path) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(format!("stat {} failed: {e}", path.display())),
    };
    if meta.file_type().is_symlink() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "DOCKER-003",
            "docker.root_dockerfile.symlink_or_absent",
            Some("Dockerfile".to_string()),
            Some(1),
            "root Dockerfile must be a symlink or absent",
            None,
        )])
    }
}

fn test_root_dockerfile_target_runtime(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join("Dockerfile");
    if !path.exists() {
        return TestResult::Pass;
    }
    let target = match std::fs::read_link(&path) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(format!("readlink {} failed: {e}", path.display())),
    };
    let expected = Path::new("docker/images/runtime/Dockerfile");
    if target == expected {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "DOCKER-003",
            "docker.root_dockerfile.target_runtime",
            Some("Dockerfile".to_string()),
            Some(1),
            "root Dockerfile symlink must target docker/images/runtime/Dockerfile",
            Some(target.display().to_string()),
        )])
    }
}

fn test_dockerfiles_under_images_only(ctx: &RunContext) -> TestResult {
    test_dir_dockerfiles_location(ctx)
}

fn test_dockerfiles_filename_convention(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for df in dctx.dockerfiles {
        let rel = df
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&df)
            .display()
            .to_string();
        if !rel.ends_with("/Dockerfile") {
            violations.push(violation(
                "DOCKER-004",
                "docker.dockerfiles.filename_convention",
                Some(rel),
                Some(1),
                "Dockerfile names must be `Dockerfile`",
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

fn test_contract_doc_generated_match(ctx: &RunContext) -> TestResult {
    let expected = match render_contract_markdown(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let path = ctx.repo_root.join("docker/CONTRACT.md");
    let actual = match std::fs::read_to_string(&path) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(format!("read {} failed: {e}", path.display())),
    };
    if actual == expected {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "DOCKER-000",
            "docker.contract_doc.generated_match",
            Some("docker/CONTRACT.md".to_string()),
            Some(1),
            "docker/CONTRACT.md drifted from generated contract registry",
            None,
        )])
    }
}

fn dockerfiles_with_instructions(ctx: &RunContext) -> Result<Vec<(String, Vec<DockerInstruction>)>, String> {
    let dctx = load_ctx(&ctx.repo_root)?;
    let mut rows = Vec::new();
    for file in dctx.dockerfiles {
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let text = std::fs::read_to_string(&file)
            .map_err(|e| format!("read {} failed: {e}", file.display()))?;
        rows.push((rel, parse_dockerfile(&text)));
    }
    Ok(rows)
}

fn allowed_tag_exceptions(policy: &Value) -> BTreeSet<String> {
    policy["allow_tagged_images_exceptions"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect()
}

fn load_json(path: &Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("read {} failed: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {} failed: {e}", path.display()))
}

fn load_bases_lock(repo_root: &Path) -> Result<BTreeMap<String, String>, String> {
    let path = repo_root.join("docker/bases.lock");
    let payload = load_json(&path)?;
    let mut rows = BTreeMap::new();
    for entry in payload["images"].as_array().cloned().unwrap_or_default() {
        let image = entry
            .get("image")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("{} is missing image field", path.display()))?;
        let digest = entry
            .get("digest")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("{} is missing digest field", path.display()))?;
        rows.insert(image.to_string(), digest.to_string());
    }
    if rows.is_empty() {
        return Err(format!("{} has no image entries", path.display()));
    }
    Ok(rows)
}

fn load_images_manifest(repo_root: &Path) -> Result<Value, String> {
    load_json(&repo_root.join("docker/images.manifest.json"))
}

fn split_from_image(from_ref: &str) -> (String, Option<String>, Option<String>) {
    let (base, digest) = match from_ref.split_once('@') {
        Some((image, digest)) => (image.to_string(), Some(digest.to_string())),
        None => (from_ref.to_string(), None),
    };
    let image = base.clone();
    let tag = image
        .rfind(':')
        .filter(|idx| image.rfind('/').map(|slash| idx > &slash).unwrap_or(true))
        .map(|idx| image[idx + 1..].to_string());
    (base, tag, digest)
}

fn final_stage_bounds(instructions: &[DockerInstruction]) -> Option<(usize, usize)> {
    let from_positions = instructions
        .iter()
        .enumerate()
        .filter_map(|(idx, ins)| (ins.keyword == "FROM").then_some(idx))
        .collect::<Vec<_>>();
    let start = *from_positions.last()?;
    Some((start, instructions.len()))
}

fn arg_defaults(instructions: &[DockerInstruction]) -> BTreeMap<String, (bool, usize)> {
    let mut out = BTreeMap::new();
    for ins in instructions {
        if ins.keyword != "ARG" {
            continue;
        }
        let raw = ins.args.trim();
        let name = raw.split('=').next().unwrap_or("").trim();
        if name.is_empty() {
            continue;
        }
        out.insert(name.to_string(), (raw.contains('='), ins.line));
    }
    out
}

fn test_from_no_latest(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword != "FROM" {
                continue;
            }
            let Some(from_ref) = parse_from_ref(&ins.args) else {
                continue;
            };
            if is_latest(&from_ref) {
                violations.push(violation(
                    "DOCKER-006",
                    "docker.from.no_latest",
                    Some(rel.clone()),
                    Some(ins.line),
                    "latest tag in FROM is forbidden",
                    Some(from_ref),
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

fn test_from_no_floating_tags(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let exceptions = allowed_tag_exceptions(&dctx.policy);
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword != "FROM" {
                continue;
            }
            let Some(from_ref) = parse_from_ref(&ins.args) else {
                continue;
            };
            if has_floating_tag(&from_ref) && !exceptions.contains(&from_ref) {
                violations.push(violation(
                    "DOCKER-006",
                    "docker.from.no_floating_tags",
                    Some(rel.clone()),
                    Some(ins.line),
                    "floating tags are forbidden unless allowlisted",
                    Some(from_ref),
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

fn test_from_no_branch_like_tags(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let forbidden = ["main", "master", "edge", "nightly"];
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword != "FROM" {
                continue;
            }
            let Some(from_ref) = parse_from_ref(&ins.args) else {
                continue;
            };
            let (_, tag, _) = split_from_image(&from_ref);
            if tag
                .as_deref()
                .is_some_and(|value| forbidden.iter().any(|candidate| candidate == &value))
            {
                violations.push(violation(
                    "DOCKER-014",
                    "docker.from.no_branch_like_tags",
                    Some(rel.clone()),
                    Some(ins.line),
                    "branch-like tags in FROM are forbidden",
                    Some(from_ref),
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

fn test_from_images_allowlisted(ctx: &RunContext) -> TestResult {
    let allowlist = match load_bases_lock(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword != "FROM" {
                continue;
            }
            let Some(from_ref) = parse_from_ref(&ins.args) else {
                continue;
            };
            let (base, _, _) = split_from_image(&from_ref);
            let image = base.split('@').next().unwrap_or(&base).to_string();
            let normalized = image.split('@').next().unwrap_or(&image).to_string();
            let without_digest = normalized.split('@').next().unwrap_or(&normalized).to_string();
            let without_tag = without_digest.clone();
            let lookup = without_tag
                .split_once('@')
                .map(|(value, _)| value.to_string())
                .unwrap_or(without_tag);
            if !allowlist.contains_key(&lookup) {
                violations.push(violation(
                    "DOCKER-015",
                    "docker.from.allowlisted_base_images",
                    Some(rel.clone()),
                    Some(ins.line),
                    "FROM image must be declared in docker/bases.lock",
                    Some(lookup),
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

fn test_from_digest_matches_bases_lock(ctx: &RunContext) -> TestResult {
    let allowlist = match load_bases_lock(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword != "FROM" {
                continue;
            }
            let Some(from_ref) = parse_from_ref(&ins.args) else {
                continue;
            };
            let (base, _, digest) = split_from_image(&from_ref);
            let image = base.split('@').next().unwrap_or(&base).to_string();
            let Some(expected) = allowlist.get(&image) else {
                continue;
            };
            let actual = digest
                .as_deref()
                .map(|v| v.to_string())
                .unwrap_or_default();
            if actual != *expected {
                violations.push(violation(
                    "DOCKER-016",
                    "docker.from.digest_matches_lock",
                    Some(rel.clone()),
                    Some(ins.line),
                    "FROM digest must match docker/bases.lock",
                    Some(format!("expected={expected} actual={actual}")),
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

fn test_from_args_have_defaults(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        let args = arg_defaults(&instructions);
        for ins in instructions {
            if ins.keyword != "FROM" {
                continue;
            }
            let Some(from_ref) = parse_from_ref(&ins.args) else {
                continue;
            };
            let bytes = from_ref.as_bytes();
            let mut idx = 0usize;
            while idx + 3 < bytes.len() {
                if bytes[idx] == b'$' && bytes[idx + 1] == b'{' {
                    let end = from_ref[idx + 2..].find('}');
                    if let Some(end) = end {
                        let name = &from_ref[idx + 2..idx + 2 + end];
                        let has_default = args.get(name).map(|entry| entry.0).unwrap_or(false);
                        if !has_default {
                            violations.push(violation(
                                "DOCKER-017",
                                "docker.from.args_have_defaults",
                                Some(rel.clone()),
                                Some(ins.line),
                                "ARG referenced by FROM must have a default value",
                                Some(name.to_string()),
                            ));
                        }
                        idx += end + 3;
                        continue;
                    }
                }
                idx += 1;
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_from_no_platform_override(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    if dctx
        .policy
        .get("allow_platform_in_from")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return TestResult::Pass;
    }
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword == "FROM" && ins.args.contains("--platform") {
                violations.push(violation(
                    "DOCKER-018",
                    "docker.from.no_platform_override",
                    Some(rel.clone()),
                    Some(ins.line),
                    "--platform in FROM is forbidden unless explicitly allowed by policy",
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

fn test_no_pip_install_without_hashes(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) { Ok(v) => v, Err(e) => return TestResult::Error(e) };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword == "RUN" {
                let args = ins.args.to_ascii_lowercase();
                if args.contains("pip install") && !args.contains("--require-hashes") {
                    violations.push(violation("DOCKER-044", "docker.run.no_pip_install_without_hashes", Some(rel.clone()), Some(ins.line), "pip install requires --require-hashes or a locked strategy", Some(ins.args)));
                }
            }
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_no_cargo_install_without_version(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) { Ok(v) => v, Err(e) => return TestResult::Error(e) };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword == "RUN" {
                let args = ins.args.to_ascii_lowercase();
                if args.contains("cargo install") && !args.contains("--version") {
                    violations.push(violation("DOCKER-045", "docker.run.no_cargo_install_without_version", Some(rel.clone()), Some(ins.line), "cargo install must pin a version", Some(ins.args)));
                }
            }
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_no_go_install_latest(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) { Ok(v) => v, Err(e) => return TestResult::Error(e) };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword == "RUN" {
                let args = ins.args.to_ascii_lowercase();
                if args.contains("go install") && args.contains("@latest") {
                    violations.push(violation("DOCKER-046", "docker.run.no_go_install_latest", Some(rel.clone()), Some(ins.line), "go install must not use @latest", Some(ins.args)));
                }
            }
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_markdown_surface_only_root_docs(ctx: &RunContext) -> TestResult {
    test_dir_allowed_markdown(ctx)
}

fn test_contract_registry_export_matches(ctx: &RunContext) -> TestResult {
    let expected = match render_contract_registry_json(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let path = ctx.repo_root.join("docker/docker.contracts.json");
    let actual = match std::fs::read_to_string(&path) {
        Ok(v) => v.trim().to_string(),
        Err(e) => return TestResult::Error(format!("read {} failed: {e}", path.display())),
    };
    if actual == expected {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation("DOCKER-049", "docker.registry.export_matches_generated", Some("docker/docker.contracts.json".to_string()), Some(1), "docker contract registry export drifted from generated output", None)])
    }
}

fn test_contract_gate_map_matches(ctx: &RunContext) -> TestResult {
    let expected = match render_contract_gate_map_json(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let path = ctx.repo_root.join("ops/inventory/docker-contract-gate-map.json");
    let actual = match std::fs::read_to_string(&path) {
        Ok(v) => v.trim().to_string(),
        Err(e) => return TestResult::Error(format!("read {} failed: {e}", path.display())),
    };
    if actual == expected {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation("DOCKER-050", "docker.gate_map.matches_generated", Some("ops/inventory/docker-contract-gate-map.json".to_string()), Some(1), "docker contract gate map drifted from generated output", None)])
    }
}

fn test_exceptions_registry_schema(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join("docker/exceptions.json");
    let payload = match load_json(&path) {
        Ok(v) => v,
        Err(e) => return TestResult::Fail(vec![violation("DOCKER-051", "docker.exceptions.schema_valid", Some("docker/exceptions.json".to_string()), Some(1), &e, None)]),
    };
    let mut violations = Vec::new();
    if payload.get("schema_version").and_then(|v| v.as_u64()) != Some(1) {
        violations.push(violation("DOCKER-051", "docker.exceptions.schema_valid", Some("docker/exceptions.json".to_string()), Some(1), "exceptions registry must set schema_version=1", None));
    }
    if payload.get("exceptions").and_then(|v| v.as_array()).is_none() {
        violations.push(violation("DOCKER-051", "docker.exceptions.schema_valid", Some("docker/exceptions.json".to_string()), Some(1), "exceptions registry must define an exceptions array", None));
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_exceptions_minimal(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join("docker/exceptions.json");
    let payload = match load_json(&path) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let known = match contracts(&ctx.repo_root) {
        Ok(v) => v.into_iter().map(|contract| contract.id.0).collect::<BTreeSet<_>>(),
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for entry in payload["exceptions"].as_array().cloned().unwrap_or_default() {
        let contract_id = entry.get("contract_id").and_then(|v| v.as_str()).unwrap_or("");
        let expires_on = entry.get("expires_on").and_then(|v| v.as_str()).unwrap_or("");
        let justification = entry.get("justification").and_then(|v| v.as_str()).unwrap_or("");
        if !known.contains(contract_id) {
            violations.push(violation("DOCKER-052", "docker.exceptions.minimal_entries", Some("docker/exceptions.json".to_string()), Some(1), "exception must cite a valid contract id", Some(contract_id.to_string())));
        }
        let valid_date = expires_on.len() == 10
            && expires_on.chars().enumerate().all(|(idx, ch)| match idx {
                4 | 7 => ch == '-',
                _ => ch.is_ascii_digit(),
            });
        if !valid_date {
            violations.push(violation("DOCKER-052", "docker.exceptions.minimal_entries", Some("docker/exceptions.json".to_string()), Some(1), "exception must set expires_on in YYYY-MM-DD format", Some(expires_on.to_string())));
        }
        if justification.trim().is_empty() {
            violations.push(violation("DOCKER-052", "docker.exceptions.minimal_entries", Some("docker/exceptions.json".to_string()), Some(1), "exception must include a justification", Some(contract_id.to_string())));
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_from_digest_required(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let exceptions = allowed_tag_exceptions(&dctx.policy);
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword != "FROM" {
                continue;
            }
            let Some(from_ref) = parse_from_ref(&ins.args) else {
                continue;
            };
            if !has_digest(&from_ref) && !exceptions.contains(&from_ref) {
                violations.push(violation(
                    "DOCKER-007",
                    "docker.from.digest_required",
                    Some(rel.clone()),
                    Some(ins.line),
                    "FROM image must include digest pin unless allowlisted",
                    Some(from_ref),
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

fn test_from_repo_digest_format(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword != "FROM" {
                continue;
            }
            let Some(from_ref) = parse_from_ref(&ins.args) else {
                continue;
            };
            let parts = from_ref.split('@').collect::<Vec<_>>();
            if parts.len() > 2 {
                violations.push(violation(
                    "DOCKER-007",
                    "docker.from.repo_digest_format",
                    Some(rel.clone()),
                    Some(ins.line),
                    "FROM image has invalid digest format",
                    Some(from_ref),
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

fn test_labels_required_present(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let required = dctx.policy["required_oci_labels"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect::<Vec<_>>();
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        let mut found = BTreeSet::new();
        for ins in &instructions {
            if ins.keyword == "LABEL" {
                let args_lc = ins.args.to_ascii_lowercase();
                for key in &required {
                    if args_lc.contains(&key.to_ascii_lowercase()) {
                        found.insert(key.to_ascii_lowercase());
                    }
                }
            }
        }
        for key in &required {
            if !found.contains(&key.to_ascii_lowercase()) {
                violations.push(violation(
                    "DOCKER-008",
                    "docker.labels.required_present",
                    Some(rel.clone()),
                    Some(1),
                    "required OCI label missing",
                    Some(key.clone()),
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

fn test_labels_required_nonempty(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let required = dctx.policy["required_oci_labels"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect::<Vec<_>>();
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in &instructions {
            if ins.keyword != "LABEL" {
                continue;
            }
            let args_lc = ins.args.to_ascii_lowercase();
            for key in &required {
                let key_lc = key.to_ascii_lowercase();
                if args_lc.contains(&key_lc) {
                    let token = ins
                        .args
                        .split_whitespace()
                        .find(|t| t.to_ascii_lowercase().contains(&key_lc))
                        .unwrap_or_default();
                    if token.ends_with("=\"\"") || token.ends_with('=') {
                        violations.push(violation(
                            "DOCKER-008",
                            "docker.labels.required_nonempty",
                            Some(rel.clone()),
                            Some(ins.line),
                            "required OCI label value must not be empty",
                            Some(token.to_string()),
                        ));
                    }
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

fn test_args_defaults_present(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let required = ["RUST_VERSION", "IMAGE_VERSION", "VCS_REF", "BUILD_DATE"];
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        let mut map = BTreeMap::<String, bool>::new();
        for ins in instructions {
            if ins.keyword != "ARG" {
                continue;
            }
            let arg = ins.args.trim().to_string();
            let has_default = arg.contains('=');
            let name = arg.split('=').next().unwrap_or("").trim().to_string();
            map.insert(name.clone(), has_default);
            if required.iter().any(|k| k == &name) && !has_default {
                violations.push(violation(
                    "DOCKER-009",
                    "docker.args.defaults_present",
                    Some(rel.clone()),
                    Some(ins.line),
                    "required ARG must have default value",
                    Some(name),
                ));
            }
        }
        for req in required {
            if !map.contains_key(req) {
                violations.push(violation(
                    "DOCKER-009",
                    "docker.args.defaults_present",
                    Some(rel.clone()),
                    Some(1),
                    "required ARG declaration missing",
                    Some(req.to_string()),
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

fn test_args_required_declared(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let required = ["RUST_VERSION", "IMAGE_VERSION", "VCS_REF", "BUILD_DATE"];
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        let mut declared = BTreeSet::new();
        for ins in instructions {
            if ins.keyword != "ARG" {
                continue;
            }
            let name = ins.args.split('=').next().unwrap_or("").trim().to_string();
            declared.insert(name);
        }
        for req in required {
            if !declared.contains(req) {
                violations.push(violation(
                    "DOCKER-009",
                    "docker.args.required_declared",
                    Some(rel.clone()),
                    Some(1),
                    "required ARG declaration missing",
                    Some(req.to_string()),
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

fn test_pattern_no_curl_pipe_sh(ctx: &RunContext) -> TestResult {
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
            if (args.contains("curl") || args.contains("wget"))
                && args.contains('|')
                && args.contains("sh")
            {
                violations.push(violation(
                    "DOCKER-010",
                    "docker.pattern.no_curl_pipe_sh",
                    Some(rel.clone()),
                    Some(ins.line),
                    "curl|sh and wget|sh patterns are forbidden",
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

fn test_pattern_no_add_remote_url(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword != "ADD" {
                continue;
            }
            let args_lower = ins.args.to_ascii_lowercase();
            if args_lower.contains("http://") || args_lower.contains("https://") {
                violations.push(violation(
                    "DOCKER-010",
                    "docker.pattern.no_add_remote_url",
                    Some(rel.clone()),
                    Some(ins.line),
                    "ADD remote URL is forbidden",
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

fn test_copy_sources_exist(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword != "COPY" {
                continue;
            }
            for src in extract_copy_sources(&ins.args) {
                if src == "." || src.starts_with('/') {
                    continue;
                }
                if !ctx.repo_root.join(&src).exists() {
                    violations.push(violation(
                        "DOCKER-011",
                        "docker.copy.sources_exist",
                        Some(rel.clone()),
                        Some(ins.line),
                        "COPY source path does not exist",
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

fn test_copy_no_absolute_sources(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword != "COPY" {
                continue;
            }
            for src in extract_copy_sources(&ins.args) {
                if src.starts_with('/') {
                    violations.push(violation(
                        "DOCKER-011",
                        "docker.copy.no_absolute_sources",
                        Some(rel.clone()),
                        Some(ins.line),
                        "absolute COPY source path is forbidden",
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

fn test_copy_no_parent_traversal(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword != "COPY" {
                continue;
            }
            for src in extract_copy_sources(&ins.args) {
                if src.contains("..") {
                    violations.push(violation(
                        "DOCKER-011",
                        "docker.copy.no_parent_traversal",
                        Some(rel.clone()),
                        Some(ins.line),
                        "COPY sources must not traverse parent directories",
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

fn image_directories_with_dockerfile(repo_root: &Path) -> Result<BTreeSet<String>, String> {
    let images_root = repo_root.join("docker/images");
    let mut out = BTreeSet::new();
    if !images_root.exists() {
        return Ok(out);
    }
    let entries = std::fs::read_dir(&images_root)
        .map_err(|e| format!("read_dir {} failed: {e}", images_root.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read docker/images entry failed: {e}"))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dockerfile = path.join("Dockerfile");
        if dockerfile.exists() {
            out.insert(
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .to_string(),
            );
        }
    }
    Ok(out)
}

fn required_image_directories(policy: &Value) -> BTreeSet<String> {
    policy["required_image_directories"]
        .as_array()
        .cloned()
        .unwrap_or_else(|| vec![Value::String("runtime".to_string())])
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect()
}

fn allowed_image_directories(policy: &Value) -> BTreeSet<String> {
    policy["allowed_image_directories"]
        .as_array()
        .cloned()
        .unwrap_or_else(|| vec![Value::String("runtime".to_string())])
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect()
}

fn test_required_images_exist(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let required = required_image_directories(&dctx.policy);
    let discovered = match image_directories_with_dockerfile(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for image in required {
        if !discovered.contains(&image) {
            violations.push(violation(
                "DOCKER-012",
                "docker.images.required_exist",
                Some("docker/images".to_string()),
                Some(1),
                "required docker image directory is missing a Dockerfile",
                Some(image),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}
