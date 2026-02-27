// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

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

fn effect_artifact_dir(ctx: &RunContext) -> Option<PathBuf> {
    let root = ctx.artifacts_root.as_ref()?;
    let dir = root.join("contracts/docker/effect");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn run_command_with_artifacts(
    ctx: &RunContext,
    program: &str,
    args: &[&str],
    stdout_name: &str,
    stderr_name: &str,
) -> Result<std::process::Output, String> {
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(&ctx.repo_root)
        .env("TZ", "UTC")
        .env("LC_ALL", "C")
        .env("LANG", "C");
    let mut child = match command.spawn() {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && ctx.skip_missing_tools => {
            return Err(format!("SKIP_MISSING_TOOL: `{program}` is not installed"));
        }
        Err(e) => return Err(format!("spawn `{program}` failed: {e}")),
    };
    let started = Instant::now();
    let timeout = Duration::from_secs(ctx.timeout_seconds.max(1));
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if started.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "command `{program}` timed out after {}s",
                        ctx.timeout_seconds.max(1)
                    ));
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(format!("wait for `{program}` failed: {e}")),
        }
    }
    let output = child
        .wait_with_output()
        .map_err(|e| format!("collect `{program}` output failed: {e}"))?;
    if let Some(dir) = effect_artifact_dir(ctx) {
        let _ = std::fs::write(dir.join(stdout_name), &output.stdout);
        let _ = std::fs::write(dir.join(stderr_name), &output.stderr);
    }
    Ok(output)
}

fn truncate_for_evidence(raw: &[u8]) -> String {
    const MAX_BYTES: usize = 4096;
    let text = String::from_utf8_lossy(raw);
    if text.len() <= MAX_BYTES {
        text.to_string()
    } else {
        format!("{}...[truncated]", &text[..MAX_BYTES])
    }
}

fn image_tag() -> String {
    "bijux-atlas-contracts:dev".to_string()
}

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

fn test_forbidden_extra_images(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let allowed = allowed_image_directories(&dctx.policy);
    let discovered = match image_directories_with_dockerfile(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for image in discovered {
        if !allowed.contains(&image) {
            violations.push(violation(
                "DOCKER-013",
                "docker.images.forbidden_extra",
                Some(format!("docker/images/{image}/Dockerfile")),
                Some(1),
                "docker image directory is not allowlisted",
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

fn test_effect_build_runtime_image(ctx: &RunContext) -> TestResult {
    let image = image_tag();
    let dockerfile = "docker/images/runtime/Dockerfile";
    let output = match run_command_with_artifacts(
        ctx,
        "docker",
        &["build", "-f", dockerfile, "-t", &image, "."],
        "docker-build-runtime.stdout.log",
        "docker-build-runtime.stderr.log",
    ) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    if output.status.success() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "DOCKER-100",
            "docker.build.runtime_image",
            Some(dockerfile.to_string()),
            Some(1),
            "docker build failed for runtime image",
            Some(truncate_for_evidence(&output.stderr)),
        )])
    }
}

fn test_effect_smoke_version(ctx: &RunContext) -> TestResult {
    let image = image_tag();
    let output = match run_command_with_artifacts(
        ctx,
        "docker",
        &[
            "run",
            "--rm",
            "--entrypoint",
            "/app/bijux-atlas",
            &image,
            "--version",
        ],
        "docker-smoke-version.stdout.log",
        "docker-smoke-version.stderr.log",
    ) {
        Ok(v) => v,
        Err(e) if e.starts_with("SKIP_MISSING_TOOL:") => {
            return TestResult::Skip(e.replacen("SKIP_MISSING_TOOL: ", "", 1));
        }
        Err(e) => return TestResult::Error(e),
    };
    if output.status.success() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "DOCKER-101",
            "docker.smoke.version",
            Some("docker/images/runtime/Dockerfile".to_string()),
            Some(1),
            "docker smoke version command failed",
            Some(truncate_for_evidence(&output.stderr)),
        )])
    }
}

fn test_effect_smoke_help(ctx: &RunContext) -> TestResult {
    let image = image_tag();
    let output = match run_command_with_artifacts(
        ctx,
        "docker",
        &[
            "run",
            "--rm",
            "--entrypoint",
            "/app/bijux-atlas",
            &image,
            "--help",
        ],
        "docker-smoke-help.stdout.log",
        "docker-smoke-help.stderr.log",
    ) {
        Ok(v) => v,
        Err(e) if e.starts_with("SKIP_MISSING_TOOL:") => {
            return TestResult::Skip(e.replacen("SKIP_MISSING_TOOL: ", "", 1));
        }
        Err(e) => return TestResult::Error(e),
    };
    if output.status.success() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "DOCKER-101",
            "docker.smoke.help",
            Some("docker/images/runtime/Dockerfile".to_string()),
            Some(1),
            "docker smoke help command failed",
            Some(truncate_for_evidence(&output.stderr)),
        )])
    }
}

fn test_effect_sbom_generated(ctx: &RunContext) -> TestResult {
    let image = image_tag();
    let output = match run_command_with_artifacts(
        ctx,
        "syft",
        &["-o", "json", &format!("docker:{image}")],
        "docker-sbom.json",
        "docker-sbom.stderr.log",
    ) {
        Ok(v) => v,
        Err(e) if e.starts_with("SKIP_MISSING_TOOL:") => {
            return TestResult::Skip(e.replacen("SKIP_MISSING_TOOL: ", "", 1));
        }
        Err(e) => return TestResult::Error(e),
    };
    if !output.status.success() {
        return TestResult::Fail(vec![violation(
            "DOCKER-102",
            "docker.sbom.generated",
            Some("docker/images/runtime/Dockerfile".to_string()),
            Some(1),
            "syft SBOM generation failed",
            Some(truncate_for_evidence(&output.stderr)),
        )]);
    }
    match serde_json::from_slice::<Value>(&output.stdout) {
        Ok(_) => TestResult::Pass,
        Err(err) => TestResult::Fail(vec![violation(
            "DOCKER-102",
            "docker.sbom.generated",
            Some("docker/images/runtime/Dockerfile".to_string()),
            Some(1),
            "syft output is not valid JSON",
            Some(err.to_string()),
        )]),
    }
}

fn test_effect_scan_passes_policy(ctx: &RunContext) -> TestResult {
    let image = image_tag();
    let output = match run_command_with_artifacts(
        ctx,
        "trivy",
        &[
            "image",
            "--severity",
            "HIGH,CRITICAL",
            "--ignore-unfixed",
            "--exit-code",
            "1",
            "--format",
            "json",
            &image,
        ],
        "docker-scan.json",
        "docker-scan.stderr.log",
    ) {
        Ok(v) => v,
        Err(e) if e.starts_with("SKIP_MISSING_TOOL:") => {
            return TestResult::Skip(e.replacen("SKIP_MISSING_TOOL: ", "", 1));
        }
        Err(e) => return TestResult::Error(e),
    };
    if output.status.success() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "DOCKER-103",
            "docker.scan.severity_threshold",
            Some("docker/images/runtime/Dockerfile".to_string()),
            Some(1),
            "trivy scan failed severity threshold",
            Some(truncate_for_evidence(&output.stderr)),
        )])
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
        Contract {
            id: ContractId("DOCKER-012".to_string()),
            title: "required images exist",
            tests: vec![TestCase {
                id: TestId("docker.images.required_exist".to_string()),
                title: "required image directories include Dockerfile",
                kind: TestKind::Pure,
                run: test_required_images_exist,
            }],
        },
        Contract {
            id: ContractId("DOCKER-013".to_string()),
            title: "forbidden extra images",
            tests: vec![TestCase {
                id: TestId("docker.images.forbidden_extra".to_string()),
                title: "docker image directories are allowlisted",
                kind: TestKind::Pure,
                run: test_forbidden_extra_images,
            }],
        },
        Contract {
            id: ContractId("DOCKER-100".to_string()),
            title: "build succeeds",
            tests: vec![TestCase {
                id: TestId("docker.build.runtime_image".to_string()),
                title: "runtime image build succeeds",
                kind: TestKind::Subprocess,
                run: test_effect_build_runtime_image,
            }],
        },
        Contract {
            id: ContractId("DOCKER-101".to_string()),
            title: "runtime smoke checks",
            tests: vec![
                TestCase {
                    id: TestId("docker.smoke.version".to_string()),
                    title: "runtime image prints version",
                    kind: TestKind::Subprocess,
                    run: test_effect_smoke_version,
                },
                TestCase {
                    id: TestId("docker.smoke.help".to_string()),
                    title: "runtime image prints help",
                    kind: TestKind::Subprocess,
                    run: test_effect_smoke_help,
                },
            ],
        },
        Contract {
            id: ContractId("DOCKER-102".to_string()),
            title: "sbom generated",
            tests: vec![TestCase {
                id: TestId("docker.sbom.generated".to_string()),
                title: "syft generates a JSON SBOM",
                kind: TestKind::Subprocess,
                run: test_effect_sbom_generated,
            }],
        },
        Contract {
            id: ContractId("DOCKER-103".to_string()),
            title: "scan passes policy",
            tests: vec![TestCase {
                id: TestId("docker.scan.severity_threshold".to_string()),
                title: "trivy scan passes configured severity threshold",
                kind: TestKind::Network,
                run: test_effect_scan_passes_policy,
            }],
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
            let mode = match case.kind {
                TestKind::Pure => "static",
                TestKind::Subprocess | TestKind::Network => "effect",
            };
            out.push_str(&format!(
                "- `{}` ({mode}, {:?}): {}\n",
                case.id.0, case.kind, case.title
            ));
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
                skip_missing_tools: false,
                timeout_seconds: 300,
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
                skip_missing_tools: false,
                timeout_seconds: 300,
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
                skip_missing_tools: false,
                timeout_seconds: 300,
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

    #[test]
    fn required_image_contract_fails_when_runtime_missing() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join("docker/images/dev")).expect("mkdir image");
        std::fs::write(
            tmp.path().join("docker/images/dev/Dockerfile"),
            "FROM scratch\n",
        )
        .expect("write dockerfile");
        std::fs::write(tmp.path().join("docker/README.md"), "# docker\n").expect("write readme");
        std::fs::write(
            tmp.path().join("docker/policy.json"),
            serde_json::json!({
                "schema_version": 1,
                "required_image_directories": ["runtime"],
                "allowed_image_directories": ["runtime", "dev"],
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
        sync_contract_markdown(tmp.path()).expect("sync contract");
        let report = crate::contracts::run(
            "docker",
            contracts,
            tmp.path(),
            &crate::contracts::RunOptions {
                mode: crate::contracts::Mode::Static,
                allow_subprocess: false,
                allow_network: false,
                skip_missing_tools: false,
                timeout_seconds: 300,
                fail_fast: false,
                contract_filter: Some("DOCKER-012".to_string()),
                test_filter: None,
                list_only: false,
                artifacts_root: None,
            },
        )
        .expect("run");
        assert!(report.fail_count() > 0, "expected missing runtime violation");
    }

    #[test]
    fn forbidden_extra_images_contract_detects_unallowlisted_directory() {
        let tmp = tempfile::tempdir().expect("tempdir");
        mk_repo(
            tmp.path(),
            "ARG RUST_VERSION=1\nARG IMAGE_VERSION=1\nARG VCS_REF=1\nARG BUILD_DATE=1\nFROM rust:1@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\nLABEL org.opencontainers.image.source=\"x\"\nLABEL org.opencontainers.image.version=\"x\"\nLABEL org.opencontainers.image.revision=\"x\"\nLABEL org.opencontainers.image.created=\"x\"\nLABEL org.opencontainers.image.ref.name=\"x\"\n",
        );
        std::fs::create_dir_all(tmp.path().join("docker/images/extra")).expect("mkdir extra image");
        std::fs::write(
            tmp.path().join("docker/images/extra/Dockerfile"),
            "FROM scratch\n",
        )
        .expect("write extra dockerfile");
        std::fs::write(
            tmp.path().join("docker/policy.json"),
            serde_json::json!({
                "schema_version": 1,
                "required_image_directories": ["runtime"],
                "allowed_image_directories": ["runtime"],
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
        .expect("overwrite policy");
        sync_contract_markdown(tmp.path()).expect("sync contract");
        let report = crate::contracts::run(
            "docker",
            contracts,
            tmp.path(),
            &crate::contracts::RunOptions {
                mode: crate::contracts::Mode::Static,
                allow_subprocess: false,
                allow_network: false,
                skip_missing_tools: false,
                timeout_seconds: 300,
                fail_fast: false,
                contract_filter: Some("DOCKER-013".to_string()),
                test_filter: None,
                list_only: false,
                artifacts_root: None,
            },
        )
        .expect("run");
        assert!(report.fail_count() > 0, "expected forbidden extra image violation");
    }
}
