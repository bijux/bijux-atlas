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

