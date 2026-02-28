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
