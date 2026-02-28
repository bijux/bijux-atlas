// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::SystemTime;
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
    let artifact_dir = effect_artifact_dir(ctx);
    let capture_root = artifact_dir.clone().unwrap_or_else(std::env::temp_dir);
    let nonce = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0);
    let stdout_capture = capture_root.join(format!(
        ".capture-{}-{}-{stdout_name}",
        std::process::id(),
        nonce
    ));
    let stderr_capture = capture_root.join(format!(
        ".capture-{}-{}-{stderr_name}",
        std::process::id(),
        nonce
    ));
    let stdout_file = File::create(&stdout_capture)
        .map_err(|e| format!("create {} failed: {e}", stdout_capture.display()))?;
    let stderr_file = File::create(&stderr_capture)
        .map_err(|e| format!("create {} failed: {e}", stderr_capture.display()))?;
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(&ctx.repo_root)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file));
    command.env("TZ", "UTC").env("LC_ALL", "C").env("LANG", "C");
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
    let status = child
        .wait()
        .map_err(|e| format!("collect `{program}` status failed: {e}"))?;
    let mut stdout = Vec::new();
    File::open(&stdout_capture)
        .and_then(|mut file| file.read_to_end(&mut stdout))
        .map_err(|e| format!("read {} failed: {e}", stdout_capture.display()))?;
    let mut stderr = Vec::new();
    File::open(&stderr_capture)
        .and_then(|mut file| file.read_to_end(&mut stderr))
        .map_err(|e| format!("read {} failed: {e}", stderr_capture.display()))?;
    let output = std::process::Output {
        status,
        stdout,
        stderr,
    };
    if let Some(dir) = artifact_dir {
        let _ = std::fs::copy(&stdout_capture, dir.join(stdout_name));
        let _ = std::fs::copy(&stderr_capture, dir.join(stderr_name));
    }
    let _ = std::fs::remove_file(&stdout_capture);
    let _ = std::fs::remove_file(&stderr_capture);
    if program == "docker" && !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Cannot connect to the Docker daemon")
            || stderr.contains("is the docker daemon running")
            || stderr.contains("docker daemon")
        {
            return Err(
                "docker daemon is unavailable; start Docker Desktop or provide a reachable Docker-compatible daemon"
                    .to_string(),
            );
        }
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

include!("contracts_registry.rs");

pub fn contract_gate_id(contract_id: &str) -> String {
    format!(
        "docker.contract.{}",
        contract_id.to_ascii_lowercase().replace('-', "_")
    )
}

fn exported_contract_effects(contract: &Contract) -> Vec<&'static str> {
    let mut effects = super::contract_effects(contract)
        .into_iter()
        .map(|effect| effect.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if super::contract_mode(contract) != super::ContractMode::Static {
        effects.insert(super::EffectKind::DockerDaemon.as_str());
    }
    if effects.contains(super::EffectKind::Network.as_str()) {
        effects.insert(super::EffectKind::Subprocess.as_str());
    }
    effects.into_iter().collect()
}

pub fn contract_gate_command(contract: &Contract) -> String {
    let effects = exported_contract_effects(contract);
    if effects.is_empty() {
        return "bijux dev atlas contracts docker --mode static".to_string();
    }
    let mut flags = vec!["bijux dev atlas contracts docker --mode effect".to_string()];
    if effects.contains(&super::EffectKind::Subprocess.as_str()) {
        flags.push("--allow-subprocess".to_string());
    }
    if effects.contains(&super::EffectKind::Network.as_str()) {
        flags.push("--allow-network".to_string());
    }
    if effects.contains(&super::EffectKind::DockerDaemon.as_str()) {
        flags.push("--allow-docker-daemon".to_string());
    }
    flags.join(" ")
}

pub fn render_contract_registry_json(repo_root: &Path) -> Result<String, String> {
    let rows = contracts(repo_root)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "docker",
        "contracts": rows.iter().map(|contract| serde_json::json!({
            "id": contract.id.0,
            "title": contract.title,
            "mode": super::contract_mode(contract).as_str(),
            "effects": exported_contract_effects(contract),
            "tests": contract.tests.iter().map(|test| serde_json::json!({
                "id": test.id.0,
                "title": test.title
            })).collect::<Vec<_>>(),
            "gate_id": contract_gate_id(&contract.id.0),
            "command": contract_gate_command(contract)
        })).collect::<Vec<_>>()
    });
    serde_json::to_string_pretty(&payload)
        .map_err(|e| format!("encode docker contract registry failed: {e}"))
}

pub fn render_contract_gate_map_json(repo_root: &Path) -> Result<String, String> {
    let rows = contracts(repo_root)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "mappings": rows.iter().map(|contract| serde_json::json!({
            "contract_id": contract.id.0,
            "gate_id": contract_gate_id(&contract.id.0),
            "command": contract_gate_command(contract)
        })).collect::<Vec<_>>()
    });
    serde_json::to_string_pretty(&payload)
        .map_err(|e| format!("encode docker gate map failed: {e}"))
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
    out.push_str("## Mapping\n\n");
    out.push_str("| Contract | Gate | Command |\n");
    out.push_str("| --- | --- | --- |\n");
    for contract in &rows {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` |\n",
            contract.id.0,
            contract_gate_id(&contract.id.0),
            contract_gate_command(contract)
        ));
    }
    out.push('\n');
    out.push_str("## Rule\n\n");
    out.push_str("- Contract ID or test ID missing from this document means it does not exist.\n");
    Ok(out)
}

pub fn sync_contract_markdown(repo_root: &Path) -> Result<(), String> {
    let rendered = render_contract_markdown(repo_root)?;
    let path = repo_root.join("docker/CONTRACT.md");
    std::fs::write(&path, rendered).map_err(|e| format!("write {} failed: {e}", path.display()))
}

pub fn sync_contract_registry_json(repo_root: &Path) -> Result<(), String> {
    let rendered = render_contract_registry_json(repo_root)?;
    let path = repo_root.join("docker/docker.contracts.json");
    std::fs::write(&path, format!("{rendered}\n"))
        .map_err(|e| format!("write {} failed: {e}", path.display()))
}

pub fn sync_contract_gate_map_json(repo_root: &Path) -> Result<(), String> {
    let rendered = render_contract_gate_map_json(repo_root)?;
    let path = repo_root.join("ops/inventory/docker-contract-gate-map.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create {} failed: {e}", parent.display()))?;
    }
    std::fs::write(&path, format!("{rendered}\n"))
        .map_err(|e| format!("write {} failed: {e}", path.display()))
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
    include!("contracts_tests.rs");
}
