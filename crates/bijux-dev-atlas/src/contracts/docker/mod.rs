// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::{
    Contract, ContractId, ContractRegistry, RunContext, TestCase, TestId, TestKind, TestResult,
    Violation,
};

#[derive(Clone, Debug)]
pub struct DockerCtx {
    pub repo_root: PathBuf,
    pub docker_root: PathBuf,
    pub dockerfiles: Vec<PathBuf>,
    pub policy: Value,
}

#[derive(Clone, Debug)]
struct DockerInstruction {
    keyword: String,
    args: String,
    line: usize,
}

fn walk_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    let mut q = VecDeque::from([root.to_path_buf()]);
    while let Some(dir) = q.pop_front() {
        let entries = std::fs::read_dir(&dir)
            .map_err(|e| format!("read_dir {} failed: {e}", dir.display()))?;
        let mut paths = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            if path.is_dir() {
                q.push_back(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
    Ok(out)
}

fn all_dockerfiles(repo_root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let docker_root = repo_root.join("docker");
    if !docker_root.exists() {
        return Ok(files);
    }
    for file in walk_files(&docker_root)? {
        if file
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s == "Dockerfile")
        {
            files.push(file);
        }
    }
    files.sort();
    Ok(files)
}

fn parse_dockerfile(text: &str) -> Vec<DockerInstruction> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut start_line = 0usize;
    for (idx, raw) in text.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if buf.is_empty() {
            start_line = line_no;
        }
        if !buf.is_empty() {
            buf.push(' ');
        }
        if let Some(prefix) = trimmed.strip_suffix('\\') {
            buf.push_str(prefix.trim_end());
            continue;
        }
        buf.push_str(trimmed);
        let mut parts = buf.splitn(2, char::is_whitespace);
        let keyword = parts.next().unwrap_or("").to_ascii_uppercase();
        let args = parts.next().unwrap_or("").trim().to_string();
        if !keyword.is_empty() {
            out.push(DockerInstruction {
                keyword,
                args,
                line: start_line,
            });
        }
        buf.clear();
        start_line = 0;
    }
    out
}

fn parse_from_ref(args: &str) -> Option<String> {
    let tokens = args.split_whitespace().collect::<Vec<_>>();
    if tokens.is_empty() {
        return None;
    }
    let mut idx = 0usize;
    while idx < tokens.len() && tokens[idx].starts_with("--") {
        idx += 1;
    }
    tokens.get(idx).map(|v| (*v).to_string())
}

fn is_latest(from_ref: &str) -> bool {
    from_ref.ends_with(":latest") || from_ref == "latest"
}

fn has_digest(from_ref: &str) -> bool {
    from_ref.contains("@sha256:")
}

fn has_floating_tag(from_ref: &str) -> bool {
    if has_digest(from_ref) {
        return false;
    }
    let tag_sep = from_ref.rfind(':');
    let slash_sep = from_ref.rfind('/');
    match (tag_sep, slash_sep) {
        (Some(colon), Some(slash)) => colon > slash,
        (Some(_), None) => true,
        _ => false,
    }
}

fn extract_copy_sources(args: &str) -> Vec<String> {
    let trimmed = args.trim();
    if trimmed.starts_with('[') {
        if let Ok(values) = serde_json::from_str::<Vec<String>>(trimmed) {
            if values.len() >= 2 {
                return values[..values.len() - 1].to_vec();
            }
        }
        return Vec::new();
    }
    let mut tokens = trimmed.split_whitespace().collect::<Vec<_>>();
    while tokens.first().is_some_and(|tok| tok.starts_with("--")) {
        if tokens.first().is_some_and(|tok| tok.starts_with("--from=")) {
            return Vec::new();
        }
        if tokens.first().is_some_and(|tok| *tok == "--from") {
            return Vec::new();
        }
        tokens.remove(0);
    }
    if tokens.len() < 2 {
        return Vec::new();
    }
    tokens[..tokens.len() - 1]
        .iter()
        .map(|v| v.trim_matches('"').to_string())
        .collect::<Vec<_>>()
}

fn load_ctx(repo_root: &Path) -> Result<DockerCtx, String> {
    let docker_root = repo_root.join("docker");
    let policy_path = docker_root.join("policy.json");
    let policy_text = std::fs::read_to_string(&policy_path)
        .map_err(|e| format!("read {} failed: {e}", policy_path.display()))?;
    let policy = serde_json::from_str::<Value>(&policy_text)
        .map_err(|e| format!("parse {} failed: {e}", policy_path.display()))?;
    Ok(DockerCtx {
        repo_root: repo_root.to_path_buf(),
        docker_root,
        dockerfiles: all_dockerfiles(repo_root)?,
        policy,
    })
}

fn violation(
    contract_id: &str,
    test_id: &str,
    file: Option<String>,
    line: Option<usize>,
    message: &str,
    evidence: Option<String>,
) -> Violation {
    Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file,
        line,
        message: message.to_string(),
        evidence,
    }
}

fn test_dir_allowed_markdown(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
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

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![
        Contract {
            id: ContractId("DOCKER-000".to_string()),
            title: "docker directory contract",
            tests: vec![
                TestCase {
                    id: TestId("docker.dir.allowed_markdown".to_string()),
                    title: "only README.md and CONTRACT.md are allowed markdown files",
                    kind: TestKind::Pure,
                    run: test_dir_allowed_markdown,
                },
                TestCase {
                    id: TestId("docker.dir.no_contracts_subdir".to_string()),
                    title: "docker/contracts subdirectory is forbidden",
                    kind: TestKind::Pure,
                    run: test_dir_no_contracts_subdir,
                },
                TestCase {
                    id: TestId("docker.dir.dockerfiles_location".to_string()),
                    title: "Dockerfiles must be under docker/images/**",
                    kind: TestKind::Pure,
                    run: test_dir_dockerfiles_location,
                },
                TestCase {
                    id: TestId("docker.contract_doc.generated_match".to_string()),
                    title: "docker CONTRACT document matches generated registry",
                    kind: TestKind::Pure,
                    run: test_contract_doc_generated_match,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-003".to_string()),
            title: "root Dockerfile policy",
            tests: vec![
                TestCase {
                    id: TestId("docker.root_dockerfile.symlink_or_absent".to_string()),
                    title: "root Dockerfile is symlink or absent",
                    kind: TestKind::Pure,
                    run: test_root_dockerfile_symlink_or_absent,
                },
                TestCase {
                    id: TestId("docker.root_dockerfile.target_runtime".to_string()),
                    title: "root Dockerfile symlink target is runtime Dockerfile",
                    kind: TestKind::Pure,
                    run: test_root_dockerfile_target_runtime,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-004".to_string()),
            title: "dockerfile location policy",
            tests: vec![
                TestCase {
                    id: TestId("docker.dockerfiles.under_images_only".to_string()),
                    title: "Dockerfiles are only under docker/images/**",
                    kind: TestKind::Pure,
                    run: test_dockerfiles_under_images_only,
                },
                TestCase {
                    id: TestId("docker.dockerfiles.filename_convention".to_string()),
                    title: "Dockerfiles follow filename convention",
                    kind: TestKind::Pure,
                    run: test_dockerfiles_filename_convention,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-006".to_string()),
            title: "forbidden tags policy",
            tests: vec![
                TestCase {
                    id: TestId("docker.from.no_latest".to_string()),
                    title: "FROM does not use latest",
                    kind: TestKind::Pure,
                    run: test_from_no_latest,
                },
                TestCase {
                    id: TestId("docker.from.no_floating_tags".to_string()),
                    title: "FROM does not use floating tags unless allowlisted",
                    kind: TestKind::Pure,
                    run: test_from_no_floating_tags,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-007".to_string()),
            title: "digest pinning policy",
            tests: vec![
                TestCase {
                    id: TestId("docker.from.digest_required".to_string()),
                    title: "FROM images require digest pin unless allowlisted",
                    kind: TestKind::Pure,
                    run: test_from_digest_required,
                },
                TestCase {
                    id: TestId("docker.from.repo_digest_format".to_string()),
                    title: "FROM digest format is valid",
                    kind: TestKind::Pure,
                    run: test_from_repo_digest_format,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-008".to_string()),
            title: "required labels policy",
            tests: vec![
                TestCase {
                    id: TestId("docker.labels.required_present".to_string()),
                    title: "required OCI labels are present",
                    kind: TestKind::Pure,
                    run: test_labels_required_present,
                },
                TestCase {
                    id: TestId("docker.labels.required_nonempty".to_string()),
                    title: "required OCI labels are non-empty",
                    kind: TestKind::Pure,
                    run: test_labels_required_nonempty,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-009".to_string()),
            title: "build args defaults policy",
            tests: vec![
                TestCase {
                    id: TestId("docker.args.defaults_present".to_string()),
                    title: "required ARG directives include defaults",
                    kind: TestKind::Pure,
                    run: test_args_defaults_present,
                },
                TestCase {
                    id: TestId("docker.args.required_declared".to_string()),
                    title: "required ARG directives are declared",
                    kind: TestKind::Pure,
                    run: test_args_required_declared,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-010".to_string()),
            title: "forbidden pattern policy",
            tests: vec![
                TestCase {
                    id: TestId("docker.pattern.no_curl_pipe_sh".to_string()),
                    title: "RUN curl|sh is forbidden",
                    kind: TestKind::Pure,
                    run: test_pattern_no_curl_pipe_sh,
                },
                TestCase {
                    id: TestId("docker.pattern.no_add_remote_url".to_string()),
                    title: "ADD remote URL is forbidden",
                    kind: TestKind::Pure,
                    run: test_pattern_no_add_remote_url,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-011".to_string()),
            title: "copy source policy",
            tests: vec![
                TestCase {
                    id: TestId("docker.copy.sources_exist".to_string()),
                    title: "COPY sources must exist",
                    kind: TestKind::Pure,
                    run: test_copy_sources_exist,
                },
                TestCase {
                    id: TestId("docker.copy.no_absolute_sources".to_string()),
                    title: "COPY absolute sources are forbidden",
                    kind: TestKind::Pure,
                    run: test_copy_no_absolute_sources,
                },
                TestCase {
                    id: TestId("docker.copy.no_parent_traversal".to_string()),
                    title: "COPY sources must not use parent traversal",
                    kind: TestKind::Pure,
                    run: test_copy_no_parent_traversal,
                },
            ],
        },
    ])
}

pub fn render_contract_markdown(repo_root: &Path) -> Result<String, String> {
    let rows = contracts(repo_root)?;
    let mut out = String::new();
    out.push_str("# Docker Contract\n\n");
    out.push_str("- Owner: `bijux-atlas-platform`\n");
    out.push_str("- Enforced by: `bijux dev atlas contracts docker`\n\n");
    out.push_str("## Contract Registry\n\n");
    for contract in &rows {
        out.push_str(&format!("### {} {}\n\n", contract.id.0, contract.title));
        out.push_str("Tests:\n");
        for case in &contract.tests {
            out.push_str(&format!("- `{}`: {}\n", case.id.0, case.title));
        }
        out.push('\n');
    }
    out.push_str("## Rule\n\n");
    out.push_str("- Contract ID or test ID missing from this document means it does not exist.\n");
    Ok(out)
}

pub fn sync_contract_markdown(repo_root: &Path) -> Result<(), String> {
    let rendered = render_contract_markdown(repo_root)?;
    let path = repo_root.join("docker/CONTRACT.md");
    std::fs::write(&path, rendered).map_err(|e| format!("write {} failed: {e}", path.display()))
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "DOCKER-007" => [
            "Use digest-pinned base images in every FROM instruction.",
            "Format: registry/repo:tag@sha256:<digest>.",
            "If a temporary exception is required, add an exact image reference to docker/policy.json allow_tagged_images_exceptions.",
        ]
        .join("\n"),
        "DOCKER-006" => [
            "Do not use :latest or floating tags in FROM instructions.",
            "Pin the image with a digest and keep tags deterministic.",
        ]
        .join("\n"),
        "DOCKER-008" => [
            "Declare all required OCI labels with non-empty values.",
            "Use LABEL directives for org.opencontainers.image.* keys required by policy.",
        ]
        .join("\n"),
        _ => "Fix violations listed for this contract and rerun `bijux dev atlas contracts docker`."
            .to_string(),
    }
}

pub struct DockerContractRegistry;

impl ContractRegistry for DockerContractRegistry {
    fn contracts(repo_root: &Path) -> Result<Vec<Contract>, String> {
        contracts(repo_root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_repo(base: &Path, dockerfile: &str) {
        std::fs::create_dir_all(base.join("docker/images/runtime")).expect("mkdir docker runtime");
        std::fs::write(base.join("docker/images/runtime/Dockerfile"), dockerfile).expect("write dockerfile");
        std::fs::write(base.join("docker/README.md"), "# docker\n").expect("write readme");
        std::fs::write(
            base.join("docker/policy.json"),
            serde_json::json!({
                "schema_version": 1,
                "allow_tagged_images_exceptions": [],
                "required_oci_labels": [
                    "org.opencontainers.image.source",
                    "org.opencontainers.image.version",
                    "org.opencontainers.image.revision",
                    "org.opencontainers.image.created",
                    "org.opencontainers.image.ref.name"
                ]
            })
            .to_string(),
        )
        .expect("write policy");
    }

    #[test]
    fn detects_latest_tag_violation() {
        let tmp = tempfile::tempdir().expect("tempdir");
        mk_repo(
            tmp.path(),
            "FROM rust:latest\nARG RUST_VERSION=1\nARG IMAGE_VERSION=1\nARG VCS_REF=1\nARG BUILD_DATE=1\nLABEL org.opencontainers.image.source=\"x\"\nLABEL org.opencontainers.image.version=\"x\"\nLABEL org.opencontainers.image.revision=\"x\"\nLABEL org.opencontainers.image.created=\"x\"\nLABEL org.opencontainers.image.ref.name=\"x\"\n",
        );
        std::os::unix::fs::symlink("docker/images/runtime/Dockerfile", tmp.path().join("Dockerfile")).expect("symlink");
        sync_contract_markdown(tmp.path()).expect("sync contract doc");
        let report = crate::contracts::run(
            "docker",
            contracts,
            tmp.path(),
            &crate::contracts::RunOptions {
                mode: crate::contracts::Mode::Static,
                allow_subprocess: false,
                allow_network: false,
                fail_fast: false,
                contract_filter: Some("DOCKER-006".to_string()),
                test_filter: Some("docker.from.no_latest".to_string()),
                list_only: false,
                artifacts_root: None,
            },
        )
        .expect("run contracts");
        assert_eq!(report.fail_count(), 1);
    }

    #[test]
    fn allows_pinned_from_and_required_fields() {
        let tmp = tempfile::tempdir().expect("tempdir");
        mk_repo(
            tmp.path(),
            "ARG RUST_VERSION=1\nARG IMAGE_VERSION=1\nARG VCS_REF=1\nARG BUILD_DATE=1\nFROM rust:1@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa AS builder\nLABEL org.opencontainers.image.source=\"x\"\nLABEL org.opencontainers.image.version=\"x\"\nLABEL org.opencontainers.image.revision=\"x\"\nLABEL org.opencontainers.image.created=\"x\"\nLABEL org.opencontainers.image.ref.name=\"x\"\nCOPY Cargo.toml /workspace/Cargo.toml\n",
        );
        std::fs::write(tmp.path().join("Cargo.toml"), "[workspace]\n").expect("write cargo toml");
        std::os::unix::fs::symlink("docker/images/runtime/Dockerfile", tmp.path().join("Dockerfile")).expect("symlink");
        sync_contract_markdown(tmp.path()).expect("sync contract doc");
        let report = crate::contracts::run(
            "docker",
            contracts,
            tmp.path(),
            &crate::contracts::RunOptions {
                mode: crate::contracts::Mode::Static,
                allow_subprocess: false,
                allow_network: false,
                fail_fast: false,
                contract_filter: None,
                test_filter: None,
                list_only: false,
                artifacts_root: None,
            },
        )
        .expect("run contracts");
        assert_eq!(report.fail_count(), 0, "report had failures");
    }

    #[test]
    fn parser_handles_multiline_and_preserves_start_line() {
        let text = include_str!("../../../tests/fixtures/dockerfiles/parser_edge_cases.Dockerfile");
        let instructions = parse_dockerfile(text);
        let label = instructions
            .iter()
            .find(|ins| ins.keyword == "LABEL")
            .expect("label instruction");
        assert_eq!(label.line, 7);
        assert!(label.args.contains("org.opencontainers.image.ref.name"));
    }

    #[test]
    fn from_parser_handles_platform_prefix_and_alias() {
        let text = include_str!("../../../tests/fixtures/dockerfiles/parser_edge_cases.Dockerfile");
        let instructions = parse_dockerfile(text);
        let from = instructions
            .iter()
            .find(|ins| ins.keyword == "FROM")
            .expect("from instruction");
        let from_ref = parse_from_ref(&from.args).expect("from ref");
        assert_eq!(
            from_ref,
            "rust:${RUST_VERSION}@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
    }

    #[test]
    fn copy_parser_ignores_copy_from_and_reads_json_array_sources() {
        let text = include_str!("../../../tests/fixtures/dockerfiles/parser_edge_cases.Dockerfile");
        let instructions = parse_dockerfile(text);
        let copy_with_from = instructions
            .iter()
            .find(|ins| ins.keyword == "COPY" && ins.args.contains("--from=builder"))
            .expect("copy --from");
        assert!(extract_copy_sources(&copy_with_from.args).is_empty());

        let copy_json = instructions
            .iter()
            .find(|ins| ins.keyword == "COPY" && ins.args.starts_with('['))
            .expect("json copy");
        assert_eq!(
            extract_copy_sources(&copy_json.args),
            vec!["Cargo.toml".to_string(), "README.md".to_string()]
        );
    }

    #[test]
    fn parser_ignores_comments_and_blank_lines() {
        let instructions = parse_dockerfile("\n# header\n\nARG A=1\n\n# next\nFROM rust:1@sha256:abc\n");
        let keywords = instructions
            .iter()
            .map(|ins| ins.keyword.as_str())
            .collect::<Vec<_>>();
        assert_eq!(keywords, vec!["ARG", "FROM"]);
    }

    #[test]
    fn parser_supports_arg_before_from() {
        let instructions = parse_dockerfile("ARG BASE=rust:1\nFROM ${BASE}@sha256:abc\n");
        assert_eq!(instructions[0].keyword, "ARG");
        assert_eq!(instructions[1].keyword, "FROM");
    }

    #[test]
    fn labels_are_checked_case_insensitively() {
        let tmp = tempfile::tempdir().expect("tempdir");
        mk_repo(
            tmp.path(),
            "ARG RUST_VERSION=1\nARG IMAGE_VERSION=1\nARG VCS_REF=1\nARG BUILD_DATE=1\nFROM rust:1@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\nLABEL ORG.OPENCONTAINERS.IMAGE.SOURCE=\"x\"\nLABEL ORG.OPENCONTAINERS.IMAGE.VERSION=\"x\"\nLABEL ORG.OPENCONTAINERS.IMAGE.REVISION=\"x\"\nLABEL ORG.OPENCONTAINERS.IMAGE.CREATED=\"x\"\nLABEL ORG.OPENCONTAINERS.IMAGE.REF.NAME=\"x\"\n",
        );
        std::os::unix::fs::symlink("docker/images/runtime/Dockerfile", tmp.path().join("Dockerfile"))
            .expect("symlink");
        sync_contract_markdown(tmp.path()).expect("sync contract doc");

        let report = crate::contracts::run(
            "docker",
            contracts,
            tmp.path(),
            &crate::contracts::RunOptions {
                mode: crate::contracts::Mode::Static,
                allow_subprocess: false,
                allow_network: false,
                fail_fast: false,
                contract_filter: Some("DOCKER-008".to_string()),
                test_filter: None,
                list_only: false,
                artifacts_root: None,
            },
        )
        .expect("run contracts");
        assert_eq!(report.fail_count(), 0, "uppercase label keys should pass");
    }
}
