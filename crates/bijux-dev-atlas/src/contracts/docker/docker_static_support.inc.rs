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

