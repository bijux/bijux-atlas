#![forbid(unsafe_code)]

mod cli;
mod dispatch;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::{fs, io::Write};

use bijux_dev_atlas_adapters::{Capabilities, RealFs, RealProcessRunner};
use bijux_dev_atlas_core::ops_inventory::{ops_inventory_summary, validate_ops_inventory};
use bijux_dev_atlas_core::{
    exit_code_for_report, explain_output, load_registry, registry_doctor, render_json,
    render_jsonl, render_text_with_durations, run_checks, select_checks, RunOptions, RunRequest,
    Selectors,
};
use bijux_dev_atlas_model::{CheckId, CheckSpec, DomainId, RunId, SuiteId, Tag};
use bijux_dev_atlas_policies::{canonical_policy_json, DevAtlasPolicySet};
use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_yaml::Value as YamlValue;
use sha2::{Digest, Sha256};
use crate::cli::{
    Cli, DocsCommand, DocsCommonArgs, DomainArg, FormatArg, OpsCommand, OpsCommonArgs, OpsGenerateCommand,
    OpsPinsCommand, OpsRenderTarget, OpsStatusTarget,
};

#[derive(Debug, Deserialize, Clone)]
struct StackProfiles {
    profiles: Vec<StackProfile>,
}

#[derive(Debug, Deserialize, Clone)]
struct ToolchainInventory {
    tools: BTreeMap<String, ToolDefinition>,
}

#[derive(Debug, Deserialize, Clone)]
struct ToolDefinition {
    required: bool,
    version_regex: String,
    probe_argv: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct StackProfile {
    name: String,
    kind_profile: String,
    cluster_config: String,
}

#[derive(Debug, Deserialize, Clone)]
struct SurfacesInventory {
    actions: Vec<SurfaceAction>,
}

#[derive(Debug, Deserialize, Clone)]
struct SurfaceAction {
    id: String,
    domain: String,
    command: Vec<String>,
    argv: Vec<String>,
}

#[derive(Debug)]
enum OpsCommandError {
    Manifest(String),
    Schema(String),
    Tool(String),
    Profile(String),
    Effect(String),
}

impl OpsCommandError {
    fn code(&self) -> &'static str {
        match self {
            Self::Manifest(_) => "OPS_MANIFEST_ERROR",
            Self::Schema(_) => "OPS_SCHEMA_ERROR",
            Self::Tool(_) => "OPS_TOOL_ERROR",
            Self::Profile(_) => "OPS_PROFILE_ERROR",
            Self::Effect(_) => "OPS_EFFECT_ERROR",
        }
    }

    fn to_stable_message(&self) -> String {
        let detail = match self {
            Self::Manifest(v)
            | Self::Schema(v)
            | Self::Tool(v)
            | Self::Profile(v)
            | Self::Effect(v) => v,
        };
        format!("{}: {}", self.code(), detail)
    }
}

struct OpsFs {
    repo_root: PathBuf,
    ops_root: PathBuf,
}

impl OpsFs {
    fn new(repo_root: PathBuf, ops_root: PathBuf) -> Self {
        Self {
            repo_root,
            ops_root,
        }
    }

    fn read_ops_json<T: for<'de> Deserialize<'de>>(&self, rel: &str) -> Result<T, OpsCommandError> {
        let path = self.ops_root.join(rel);
        let text = std::fs::read_to_string(&path).map_err(|err| {
            OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display()))
        })?;
        serde_json::from_str(&text).map_err(|err| {
            OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display()))
        })
    }

    fn write_artifact_json(
        &self,
        run_id: &RunId,
        rel: &str,
        payload: &serde_json::Value,
    ) -> Result<PathBuf, OpsCommandError> {
        let path = self
            .repo_root
            .join("artifacts/atlas-dev/ops")
            .join(run_id.as_str())
            .join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| {
                OpsCommandError::Manifest(format!("failed to create {}: {err}", parent.display()))
            })?;
        }
        let content = serde_json::to_string_pretty(payload)
            .map_err(|err| OpsCommandError::Schema(format!("failed to serialize json: {err}")))?;
        std::fs::write(&path, content).map_err(|err| {
            OpsCommandError::Manifest(format!("failed to write {}: {err}", path.display()))
        })?;
        Ok(path)
    }
}

struct OpsProcess {
    allow_subprocess: bool,
}

impl OpsProcess {
    fn new(allow_subprocess: bool) -> Self {
        Self { allow_subprocess }
    }

    fn probe_tool(
        &self,
        name: &str,
        probe_argv: &[String],
        version_regex: &str,
    ) -> Result<serde_json::Value, OpsCommandError> {
        if !self.allow_subprocess {
            return Err(OpsCommandError::Effect(
                "subprocess is denied; pass --allow-subprocess".to_string(),
            ));
        }
        let mut cmd = ProcessCommand::new(name);
        if probe_argv.is_empty() {
            cmd.arg("--version");
        } else {
            cmd.args(probe_argv);
        }
        match cmd.output() {
            Ok(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                let raw = text.lines().next().unwrap_or("").trim().to_string();
                let version = normalize_tool_version_with_regex(&raw, version_regex);
                Ok(serde_json::json!({"name": name, "installed": true, "version_raw": raw, "version": version, "version_regex": version_regex}))
            }
            Ok(_) => Ok(
                serde_json::json!({"name": name, "installed": false, "version_raw": serde_json::Value::Null, "version": serde_json::Value::Null}),
            ),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(
                serde_json::json!({"name": name, "installed": false, "version_raw": serde_json::Value::Null, "version": serde_json::Value::Null}),
            ),
            Err(err) => Err(OpsCommandError::Tool(format!(
                "failed to probe tool `{name}`: {err}"
            ))),
        }
    }

    fn run_subprocess(
        &self,
        binary: &str,
        args: &[String],
        cwd: &Path,
    ) -> Result<(String, serde_json::Value), OpsCommandError> {
        if !self.allow_subprocess {
            return Err(OpsCommandError::Effect(
                "subprocess is denied; pass --allow-subprocess".to_string(),
            ));
        }
        let mut cmd = ProcessCommand::new(binary);
        cmd.args(args).current_dir(cwd);
        cmd.env_clear();
        for key in [
            "PATH",
            "HOME",
            "KUBECONFIG",
            "HELM_CACHE_HOME",
            "HELM_CONFIG_HOME",
            "HELM_DATA_HOME",
        ] {
            if let Ok(value) = std::env::var(key) {
                cmd.env(key, value);
            }
        }
        let output = cmd
            .output()
            .map_err(|err| OpsCommandError::Tool(format!("failed to run `{binary}`: {err}")))?;
        let stdout = String::from_utf8(output.stdout).map_err(|err| {
            OpsCommandError::Tool(format!("`{binary}` emitted non-utf8 stdout: {err}"))
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(OpsCommandError::Tool(format!(
                "subprocess `{binary}` failed: status={} stderr={stderr}",
                output.status
            )));
        }
        let event = serde_json::json!({
            "binary": binary,
            "argv": args,
            "cwd": cwd.display().to_string(),
            "env_allowlist": ["PATH", "HOME", "KUBECONFIG", "HELM_CACHE_HOME", "HELM_CONFIG_HOME", "HELM_DATA_HOME"]
        });
        Ok((stdout, event))
    }
}

impl From<DomainArg> for DomainId {
    fn from(value: DomainArg) -> Self {
        match value {
            DomainArg::Root => Self::Root,
            DomainArg::Workflows => Self::Workflows,
            DomainArg::Configs => Self::Configs,
            DomainArg::Docker => Self::Docker,
            DomainArg::Crates => Self::Crates,
            DomainArg::Ops => Self::Ops,
            DomainArg::Repo => Self::Repo,
            DomainArg::Docs => Self::Docs,
            DomainArg::Make => Self::Make,
        }
    }
}

fn discover_repo_root(start: &Path) -> Result<PathBuf, String> {
    let mut current = start.canonicalize().map_err(|err| err.to_string())?;
    loop {
        if current.join("ops/atlas-dev/registry.toml").exists() {
            return Ok(current);
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            return Err(
                "could not discover repo root (no ops/atlas-dev/registry.toml found)".to_string(),
            );
        }
    }
}

fn resolve_repo_root(arg: Option<PathBuf>) -> Result<PathBuf, String> {
    match arg {
        Some(path) => discover_repo_root(&path),
        None => {
            let cwd = std::env::current_dir().map_err(|err| err.to_string())?;
            discover_repo_root(&cwd)
        }
    }
}

fn parse_selectors(
    suite: Option<String>,
    domain: Option<DomainArg>,
    tag: Option<String>,
    id: Option<String>,
    include_internal: bool,
    include_slow: bool,
) -> Result<Selectors, String> {
    let normalized_suite = suite
        .as_deref()
        .map(normalize_suite_name)
        .transpose()?
        .map(std::string::ToString::to_string);
    Ok(Selectors {
        suite: normalized_suite
            .as_ref()
            .map(|v| SuiteId::parse(v))
            .transpose()?,
        domain: domain.map(Into::into),
        tag: tag.map(|v| Tag::parse(&v)).transpose()?,
        id_glob: id,
        include_internal,
        include_slow,
    })
}

fn normalize_suite_name(raw: &str) -> Result<&str, String> {
    match raw {
        "ci-fast" => Ok("ci_fast"),
        "ci" => Ok("ci"),
        "local" => Ok("local"),
        "deep" => Ok("deep"),
        other => Ok(other),
    }
}

fn write_output_if_requested(out: Option<PathBuf>, rendered: &str) -> Result<(), String> {
    if let Some(path) = out {
        std::fs::write(&path, format!("{rendered}\n"))
            .map_err(|err| format!("cannot write {}: {err}", path.display()))?;
    }
    Ok(())
}

fn render_list_output(checks: &[CheckSpec], format: FormatArg) -> Result<String, String> {
    match format {
        FormatArg::Text => {
            let mut lines = Vec::new();
            let mut current_domain = String::new();
            for check in checks {
                let domain = format!("{:?}", check.domain).to_ascii_lowercase();
                if domain != current_domain {
                    if !current_domain.is_empty() {
                        lines.push(String::new());
                    }
                    lines.push(format!("[{domain}]"));
                    current_domain = domain;
                }
                let tags = check
                    .tags
                    .iter()
                    .map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                let suites = check
                    .suites
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                lines.push(format!(
                    "{}\ttags={}\tsuites={}\t{}",
                    check.id, tags, suites, check.title
                ));
            }
            Ok(lines.join("\n"))
        }
        FormatArg::Json => {
            let rows: Vec<serde_json::Value> = checks
                .iter()
                .map(|check| {
                    serde_json::json!({
                        "id": check.id.as_str(),
                        "domain": format!("{:?}", check.domain).to_ascii_lowercase(),
                        "tags": check.tags.iter().map(|v| v.as_str()).collect::<Vec<_>>(),
                        "suites": check.suites.iter().map(|v| v.as_str()).collect::<Vec<_>>(),
                        "title": check.title,
                    })
                })
                .collect();
            serde_json::to_string_pretty(&serde_json::json!({"checks": rows}))
                .map_err(|err| err.to_string())
        }
        FormatArg::Jsonl => Err("jsonl output is not supported for list".to_string()),
    }
}

fn render_explain_output(explain_text: String, format: FormatArg) -> Result<String, String> {
    match format {
        FormatArg::Text => Ok(explain_text),
        FormatArg::Json => {
            let mut map = serde_json::Map::new();
            for line in explain_text.lines() {
                if let Some((key, value)) = line.split_once(": ") {
                    map.insert(
                        key.to_string(),
                        serde_json::Value::String(value.to_string()),
                    );
                }
            }
            serde_json::to_string_pretty(&serde_json::Value::Object(map))
                .map_err(|err| err.to_string())
        }
        FormatArg::Jsonl => Err("jsonl output is not supported for explain".to_string()),
    }
}

pub(crate) struct CheckListOptions {
    repo_root: Option<PathBuf>,
    suite: Option<String>,
    domain: Option<DomainArg>,
    tag: Option<String>,
    id: Option<String>,
    include_internal: bool,
    include_slow: bool,
    format: FormatArg,
    out: Option<PathBuf>,
}

pub(crate) fn run_check_list(options: CheckListOptions) -> Result<(String, i32), String> {
    let root = resolve_repo_root(options.repo_root)?;
    let selectors = parse_selectors(
        options.suite,
        options.domain,
        options.tag,
        options.id,
        options.include_internal,
        options.include_slow,
    )?;
    let registry = load_registry(&root)?;
    let checks = select_checks(&registry, &selectors)?;
    let rendered = render_list_output(&checks, options.format)?;
    write_output_if_requested(options.out, &rendered)?;
    Ok((rendered, 0))
}

pub(crate) fn run_check_explain(
    check_id: String,
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let registry = load_registry(&root)?;
    let id = CheckId::parse(&check_id)?;
    let rendered = render_explain_output(explain_output(&registry, &id)?, format)?;
    write_output_if_requested(out, &rendered)?;
    Ok((rendered, 0))
}

pub(crate) struct CheckRunOptions {
    repo_root: Option<PathBuf>,
    artifacts_root: Option<PathBuf>,
    run_id: Option<String>,
    suite: Option<String>,
    domain: Option<DomainArg>,
    tag: Option<String>,
    id: Option<String>,
    include_internal: bool,
    include_slow: bool,
    allow_subprocess: bool,
    allow_git: bool,
    allow_write: bool,
    allow_network: bool,
    fail_fast: bool,
    max_failures: Option<usize>,
    format: FormatArg,
    out: Option<PathBuf>,
    durations: usize,
}

#[derive(Debug, Serialize)]
struct DocsPageRow {
    path: String,
    in_nav: bool,
}

#[derive(Debug)]
struct DocsContext {
    repo_root: PathBuf,
    docs_root: PathBuf,
    artifacts_root: PathBuf,
    run_id: RunId,
}

#[derive(Default)]
struct DocsIssues {
    errors: Vec<String>,
    warnings: Vec<String>,
}

pub(crate) fn run_check_run(options: CheckRunOptions) -> Result<(String, i32), String> {
    let root = resolve_repo_root(options.repo_root)?;
    let selectors = parse_selectors(
        options.suite,
        options.domain,
        options.tag,
        options.id,
        options.include_internal,
        options.include_slow,
    )?;
    let request = RunRequest {
        repo_root: root.clone(),
        domain: selectors.domain,
        capabilities: Capabilities::from_cli_flags(
            options.allow_write,
            options.allow_subprocess,
            options.allow_git,
            options.allow_network,
        ),
        artifacts_root: options
            .artifacts_root
            .or_else(|| Some(root.join("artifacts"))),
        run_id: options.run_id.map(|rid| RunId::parse(&rid)).transpose()?,
        command: Some("bijux dev atlas check run".to_string()),
    };
    let run_options = RunOptions {
        fail_fast: options.fail_fast,
        max_failures: options.max_failures,
    };
    let report = run_checks(
        &RealProcessRunner,
        &RealFs,
        &request,
        &selectors,
        &run_options,
    )?;
    let rendered = match options.format {
        FormatArg::Text => render_text_with_durations(&report, options.durations),
        FormatArg::Json => render_json(&report)?,
        FormatArg::Jsonl => render_jsonl(&report)?,
    };
    write_output_if_requested(options.out, &rendered)?;
    Ok((rendered, exit_code_for_report(&report)))
}

pub(crate) fn run_check_doctor(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let registry_report = registry_doctor(&root);
    let inventory_errors = validate_ops_inventory(&root);
    let selectors = parse_selectors(Some("doctor".to_string()), None, None, None, false, false)?;
    let request = RunRequest {
        repo_root: root.clone(),
        domain: None,
        capabilities: Capabilities::deny_all(),
        artifacts_root: Some(root.join("artifacts")),
        run_id: Some(RunId::from_seed("doctor_run")),
        command: Some("bijux dev atlas doctor".to_string()),
    };
    let report = run_checks(
        &RealProcessRunner,
        &RealFs,
        &request,
        &selectors,
        &RunOptions::default(),
    )?;
    let docs_common = DocsCommonArgs {
        repo_root: Some(root.clone()),
        artifacts_root: Some(root.join("artifacts")),
        run_id: Some("doctor_docs".to_string()),
        format,
        out: None,
        allow_subprocess: false,
        allow_write: false,
        allow_network: false,
        strict: false,
        include_drafts: false,
    };
    let docs_ctx = docs_context(&docs_common)?;
    let docs_validate = docs_validate_payload(&docs_ctx, &docs_common)?;
    let docs_links = docs_links_payload(&docs_ctx, &docs_common)?;
    let docs_lint = docs_lint_payload(&docs_ctx, &docs_common)?;
    let check_exit = exit_code_for_report(&report);
    let status =
        if registry_report.errors.is_empty() && inventory_errors.is_empty() && check_exit == 0 {
            "ok"
        } else {
            "failed"
        };
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": status,
        "registry_errors": registry_report.errors,
        "inventory_errors": inventory_errors,
        "docs_doctor": {
            "validate_errors": docs_validate.get("errors").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "links_errors": docs_links.get("errors").and_then(|v| v.as_array()).map_or(0, Vec::len),
            "lint_errors": docs_lint.get("errors").and_then(|v| v.as_array()).map_or(0, Vec::len)
        },
        "check_report": report,
    });

    let evidence_dir = root.join("artifacts/atlas-dev/doctor");
    fs::create_dir_all(&evidence_dir)
        .map_err(|err| format!("failed to create {}: {err}", evidence_dir.display()))?;
    let evidence_path = evidence_dir.join("doctor.report.json");
    fs::write(
        &evidence_path,
        serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("failed to write {}: {err}", evidence_path.display()))?;

    let rendered = match format {
        FormatArg::Text => format!(
            "status: {status}\nregistry_errors: {}\ninventory_errors: {}\ncheck_summary: passed={} failed={} skipped={} errors={} total={}\nevidence: {}",
            payload["registry_errors"].as_array().map_or(0, Vec::len),
            payload["inventory_errors"].as_array().map_or(0, Vec::len),
            report.summary.passed,
            report.summary.failed,
            report.summary.skipped,
            report.summary.errors,
            report.summary.total,
            evidence_path.display(),
        ),
        FormatArg::Json => serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
        FormatArg::Jsonl => serde_json::to_string(&payload).map_err(|err| err.to_string())?,
    };
    write_output_if_requested(out, &rendered)?;
    let exit = if status == "ok" { 0 } else { 1 };
    Ok((rendered, exit))
}

pub(crate) fn run_print_policies(repo_root: Option<PathBuf>) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let policies = DevAtlasPolicySet::load(&root).map_err(|err| err.to_string())?;
    let rendered = canonical_policy_json(&policies.to_document()).map_err(|err| err.to_string())?;
    Ok((rendered, 0))
}

fn docs_context(common: &DocsCommonArgs) -> Result<DocsContext, String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let artifacts_root = common
        .artifacts_root
        .clone()
        .unwrap_or_else(|| repo_root.join("artifacts"));
    let run_id = common
        .run_id
        .as_ref()
        .map(|v| RunId::parse(v))
        .transpose()?
        .unwrap_or_else(|| RunId::from_seed("docs_run"));
    Ok(DocsContext {
        docs_root: repo_root.join("docs"),
        repo_root,
        artifacts_root,
        run_id,
    })
}

fn slugify_anchor(text: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for c in text.chars().flat_map(|c| c.to_lowercase()) {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if (c.is_whitespace() || c == '-' || c == '_') && !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn docs_markdown_files(docs_root: &Path, include_drafts: bool) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if docs_root.exists() {
        for file in walk_files_local(docs_root) {
            if file.extension().and_then(|v| v.to_str()) == Some("md") {
                if !include_drafts {
                    if let Ok(rel) = file.strip_prefix(docs_root) {
                        if rel.to_string_lossy().starts_with("_drafts/") {
                            continue;
                        }
                    }
                }
                files.push(file);
            }
        }
    }
    files.sort();
    files
}

fn walk_files_local(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn parse_mkdocs_yaml(repo_root: &Path) -> Result<YamlValue, String> {
    let path = repo_root.join("mkdocs.yml");
    let text = fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    serde_yaml::from_str(&text).map_err(|e| format!("failed to parse {}: {e}", path.display()))
}

fn collect_nav_refs(node: &YamlValue, out: &mut Vec<(String, String)>) {
    match node {
        YamlValue::Sequence(seq) => {
            for item in seq {
                collect_nav_refs(item, out);
            }
        }
        YamlValue::Mapping(map) => {
            for (k, v) in map {
                let title = k.as_str().unwrap_or_default().to_string();
                if let Some(path) = v.as_str() {
                    out.push((title, path.to_string()));
                } else {
                    collect_nav_refs(v, out);
                }
            }
        }
        _ => {}
    }
}

fn mkdocs_nav_refs(repo_root: &Path) -> Result<Vec<(String, String)>, String> {
    let yaml = parse_mkdocs_yaml(repo_root)?;
    let nav = yaml
        .get("nav")
        .ok_or_else(|| "mkdocs.yml missing `nav`".to_string())?;
    let mut refs = Vec::new();
    collect_nav_refs(nav, &mut refs);
    refs.sort();
    Ok(refs)
}

fn docs_inventory_payload(ctx: &DocsContext, common: &DocsCommonArgs) -> Result<serde_json::Value, String> {
    let nav_refs = mkdocs_nav_refs(&ctx.repo_root)?;
    let nav_set = nav_refs.iter().map(|(_, p)| p.clone()).collect::<std::collections::BTreeSet<_>>();
    let rows = docs_markdown_files(&ctx.docs_root, common.include_drafts)
        .into_iter()
        .filter_map(|p| p.strip_prefix(&ctx.docs_root).ok().map(|r| r.display().to_string()))
        .map(|rel| DocsPageRow {
            in_nav: nav_set.contains(&rel),
            path: rel,
        })
        .collect::<Vec<_>>();
    let orphan_pages = rows
        .iter()
        .filter(|r| !r.in_nav && !r.path.starts_with("_assets/") && (common.include_drafts || !r.path.starts_with("_drafts/")))
        .map(|r| r.path.clone())
        .collect::<Vec<_>>();
    let duplicate_titles = {
        let mut seen = BTreeMap::<String, usize>::new();
        for (title, _) in &nav_refs {
            *seen.entry(title.clone()).or_default() += 1;
        }
        let mut d = seen.into_iter().filter(|(_, n)| *n > 1).map(|(k, _)| k).collect::<Vec<_>>();
        d.sort();
        d
    };
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},
        "options": {"strict": common.strict, "include_drafts": common.include_drafts},
        "nav": nav_refs.iter().map(|(title, path)| serde_json::json!({"title": title, "path": path})).collect::<Vec<_>>(),
        "pages": rows,
        "orphan_pages": orphan_pages,
        "duplicate_nav_titles": duplicate_titles
    }))
}

fn docs_validate_payload(ctx: &DocsContext, common: &DocsCommonArgs) -> Result<serde_json::Value, String> {
    let yaml = parse_mkdocs_yaml(&ctx.repo_root)?;
    let mut issues = DocsIssues::default();
    let docs_dir = yaml.get("docs_dir").and_then(|v| v.as_str()).unwrap_or_default();
    if docs_dir != "docs" {
        issues.errors.push(format!("DOCS_NAV_ERROR: mkdocs.yml docs_dir must be `docs`, got `{docs_dir}`"));
    }
    for (_, rel) in mkdocs_nav_refs(&ctx.repo_root)? {
        if !ctx.docs_root.join(&rel).exists() {
            issues.errors.push(format!("DOCS_NAV_ERROR: mkdocs nav references missing file `{rel}`"));
        }
    }
    let inv = docs_inventory_payload(ctx, common)?;
    for dup in inv["duplicate_nav_titles"].as_array().into_iter().flatten() {
        if let Some(title) = dup.as_str() {
            issues.warnings.push(format!("DOCS_NAV_ERROR: duplicate mkdocs nav title `{title}`"));
        }
    }
    if common.strict {
        issues.errors.append(&mut issues.warnings);
    }
    let text = if issues.errors.is_empty() {
        format!("docs validate passed (warnings={})", issues.warnings.len())
    } else {
        format!("docs validate failed (errors={} warnings={})", issues.errors.len(), issues.warnings.len())
    };
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": text,
        "errors": issues.errors,
        "warnings": issues.warnings,
        "rows": inv["nav"].as_array().cloned().unwrap_or_default(),
        "summary": {"total": inv["nav"].as_array().map(|v| v.len()).unwrap_or(0), "errors": inv["errors"].as_array().map(|v| v.len()).unwrap_or(0), "warnings": inv["warnings"].as_array().map(|v| v.len()).unwrap_or(0)},
        "capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},
        "options": {"strict": common.strict, "include_drafts": common.include_drafts}
    }))
}

fn markdown_anchors(text: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix('#') {
            let heading = rest.trim_start_matches('#').trim();
            if !heading.is_empty() {
                out.insert(slugify_anchor(heading));
            }
        }
    }
    out
}

fn docs_links_payload(ctx: &DocsContext, common: &DocsCommonArgs) -> Result<serde_json::Value, String> {
    let mut rows = Vec::<serde_json::Value>::new();
    let mut issues = DocsIssues::default();
    let link_re = Regex::new(r"\[[^\]]+\]\(([^)]+)\)").map_err(|e| e.to_string())?;
    for file in docs_markdown_files(&ctx.docs_root, common.include_drafts) {
        let rel = file.strip_prefix(&ctx.repo_root).unwrap_or(&file).display().to_string();
        let text = fs::read_to_string(&file).map_err(|e| format!("failed to read {rel}: {e}"))?;
        let anchors = markdown_anchors(&text);
        for (idx, line) in text.lines().enumerate() {
            for cap in link_re.captures_iter(line) {
                let target = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                if target.starts_with("http://") || target.starts_with("https://") || target.starts_with("mailto:") {
                    if common.allow_network {
                        rows.push(serde_json::json!({"file": rel, "line": idx + 1, "target": target, "ok": true, "external": true, "checked_network": false}));
                    }
                    continue;
                }
                if let Some(anchor) = target.strip_prefix('#') {
                    let ok = anchors.contains(anchor);
                    if !ok {
                        issues.errors.push(format!("DOCS_LINK_ERROR: {rel}:{} missing same-file anchor `#{anchor}`", idx + 1));
                    }
                    rows.push(serde_json::json!({"file": rel, "line": idx + 1, "target": target, "ok": ok}));
                    continue;
                }
                let (path_part, anchor_part) = target.split_once('#').map_or((target, None), |(a, b)| (a, Some(b)));
                if path_part.is_empty() || path_part.ends_with('/') {
                    continue;
                }
                let resolved = file.parent().unwrap_or(&ctx.docs_root).join(path_part);
                let exists = resolved.exists();
                let mut ok = exists;
                if exists {
                    if let Some(anchor) = anchor_part {
                        if resolved.extension().and_then(|v| v.to_str()) == Some("md") {
                            let target_text = fs::read_to_string(&resolved).unwrap_or_default();
                            ok = markdown_anchors(&target_text).contains(anchor);
                        }
                    }
                }
                if !ok {
                    issues.errors.push(format!("DOCS_LINK_ERROR: {rel}:{} unresolved link `{target}`", idx + 1));
                }
                rows.push(serde_json::json!({"file": rel, "line": idx + 1, "target": target, "ok": ok}));
            }
        }
    }
    rows.sort_by(|a,b| a["file"].as_str().cmp(&b["file"].as_str()).then(a["line"].as_u64().cmp(&b["line"].as_u64())).then(a["target"].as_str().cmp(&b["target"].as_str())));
    issues.errors.sort();
    issues.errors.dedup();
    Ok(serde_json::json!({
        "schema_version":1,
        "run_id":ctx.run_id.as_str(),
        "text": if issues.errors.is_empty() {"docs links passed"} else {"docs links failed"},
        "rows":rows,
        "errors":issues.errors,
        "warnings": issues.warnings,
        "capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},
        "options": {"strict": common.strict, "include_drafts": common.include_drafts},
        "external_link_check": {"enabled": common.allow_network, "mode": "disabled_best_effort"}
    }))
}

fn docs_lint_payload(ctx: &DocsContext, common: &DocsCommonArgs) -> Result<serde_json::Value, String> {
    let mut errors = Vec::<String>::new();
    for file in docs_markdown_files(&ctx.docs_root, common.include_drafts) {
        let rel = file.strip_prefix(&ctx.docs_root).unwrap_or(&file).display().to_string();
        if rel.contains(' ') {
            errors.push(format!("docs filename must not contain spaces: `{rel}`"));
        }
        let name = file.file_name().and_then(|v| v.to_str()).unwrap_or_default();
        if name != "README.md" && name != "INDEX.md" && name.chars().any(|c| c.is_ascii_uppercase()) {
            errors.push(format!("docs filename should use lowercase intent-based naming: `{rel}`"));
        }
        let text = fs::read_to_string(&file).map_err(|e| format!("failed to read {rel}: {e}"))?;
        for (idx, line) in text.lines().enumerate() {
            if line.ends_with(' ') || line.contains('\t') {
                errors.push(format!("{rel}:{} formatting lint failure (tab/trailing-space)", idx + 1));
            }
        }
    }
    errors.sort();
    errors.dedup();
    Ok(serde_json::json!({"schema_version":1,"run_id":ctx.run_id.as_str(),"text": if errors.is_empty() {"docs lint passed"} else {"docs lint failed"},"rows":[],"errors":errors,"warnings":[],"capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},"options": {"strict": common.strict, "include_drafts": common.include_drafts}}))
}

fn docs_grep_payload(ctx: &DocsContext, common: &DocsCommonArgs, pattern: &str) -> Result<serde_json::Value, String> {
    let mut rows = Vec::<serde_json::Value>::new();
    for file in docs_markdown_files(&ctx.docs_root, common.include_drafts) {
        let rel = file.strip_prefix(&ctx.repo_root).unwrap_or(&file).display().to_string();
        let text = fs::read_to_string(&file).map_err(|e| format!("failed to read {rel}: {e}"))?;
        for (idx, line) in text.lines().enumerate() {
            if line.contains(pattern) {
                rows.push(serde_json::json!({"file": rel, "line": idx + 1, "text": line.trim()}));
            }
        }
    }
    rows.sort_by(|a,b| a["file"].as_str().cmp(&b["file"].as_str()).then(a["line"].as_u64().cmp(&b["line"].as_u64())));
    Ok(serde_json::json!({"schema_version":1,"run_id":ctx.run_id.as_str(),"text": format!("{} matches", rows.len()),"rows":rows,"capabilities": {"network": common.allow_network, "subprocess": common.allow_subprocess, "fs_write": common.allow_write},"options": {"strict": common.strict, "include_drafts": common.include_drafts}}))
}

fn docs_build_or_serve_subprocess(args: &[String], common: &DocsCommonArgs, label: &str) -> Result<(serde_json::Value, i32), String> {
    if !common.allow_subprocess {
        return Err(format!("{label} requires --allow-subprocess"));
    }
    if label == "docs build" && !common.allow_write {
        return Err("docs build requires --allow-write".to_string());
    }
    let ctx = docs_context(common)?;
    let output_dir = ctx.artifacts_root.join("atlas-dev").join("docs").join(ctx.run_id.as_str()).join("site");
    if label == "docs build" {
        fs::create_dir_all(&output_dir).map_err(|e| format!("failed to create {}: {e}", output_dir.display()))?;
    }
    let mut cmd = ProcessCommand::new("mkdocs");
    cmd.args(args).current_dir(&ctx.repo_root);
    if label == "docs build" {
        cmd.args(["--site-dir", output_dir.to_str().unwrap_or("artifacts/atlas-dev/docs/site")]);
    }
    let out = cmd.output().map_err(|e| format!("failed to run mkdocs: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let code = out.status.code().unwrap_or(1);
    let mut files = Vec::<serde_json::Value>::new();
    if label == "docs build" && output_dir.exists() {
        for path in walk_files_local(&output_dir) {
            let Ok(bytes) = fs::read(&path) else { continue };
            let rel = path
                .strip_prefix(&output_dir)
                .unwrap_or(&path)
                .display()
                .to_string();
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            files.push(serde_json::json!({
                "path": rel,
                "sha256": format!("{:x}", hasher.finalize()),
                "bytes": bytes.len()
            }));
        }
        files.sort_by(|a,b| a["path"].as_str().cmp(&b["path"].as_str()));
        let index_path = ctx.artifacts_root.join("atlas-dev").join("docs").join(ctx.run_id.as_str()).join("build.index.json");
        if common.allow_write {
            if let Some(parent) = index_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&index_path, serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "run_id": ctx.run_id.as_str(),
                "files": files
            })).unwrap_or_default());
        }
    }
    Ok((serde_json::json!({
        "schema_version":1,
        "run_id": ctx.run_id.as_str(),
        "error_code": if code == 0 { serde_json::Value::Null } else { serde_json::Value::String("DOCS_BUILD_ERROR".to_string()) },
        "text": format!("{label} {}", if code==0 {"ok"} else {"failed"}),
        "rows":[{"command": args, "exit_code": code, "stdout": stdout, "stderr": stderr, "site_dir": output_dir.display().to_string()}],
        "artifacts": {"site_dir": output_dir.display().to_string(), "build_index": ctx.artifacts_root.join("atlas-dev").join("docs").join(ctx.run_id.as_str()).join("build.index.json").display().to_string(), "files": files},
        "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
        "options": {"strict": common.strict, "include_drafts": common.include_drafts}
    }), code))
}

pub(crate) fn run_docs_command(quiet: bool, command: DocsCommand) -> i32 {
    let run = (|| -> Result<(String, i32), String> {
        let started = std::time::Instant::now();
        match command {
            DocsCommand::Validate(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_validate_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) { 1 } else { 0 };
                if code != 0 { payload["error_code"] = serde_json::json!("DOCS_NAV_ERROR"); }
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::Inventory(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_inventory_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            DocsCommand::Links(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_links_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) { 1 } else { 0 };
                if code != 0 { payload["error_code"] = serde_json::json!("DOCS_LINK_ERROR"); }
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::Lint(common) => {
                let ctx = docs_context(&common)?;
                let mut payload = docs_lint_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) { 1 } else { 0 };
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::Grep(args) => {
                let ctx = docs_context(&args.common)?;
                let mut payload = docs_grep_payload(&ctx, &args.common, &args.pattern)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(args.common.format, args.common.out, &payload)?, 0))
            }
            DocsCommand::Build(common) => {
                let (mut payload, code) = docs_build_or_serve_subprocess(&["build".to_string()], &common, "docs build")?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            DocsCommand::Serve(args) => {
                let (mut payload, code) = docs_build_or_serve_subprocess(
                    &["serve".to_string(), "--dev-addr".to_string(), format!("{}:{}", args.host, args.port)],
                    &args.common,
                    "docs serve",
                )?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(args.common.format, args.common.out, &payload)?, code))
            }
            DocsCommand::Doctor(common) => {
                let ctx = docs_context(&common)?;
                let validate = docs_validate_payload(&ctx, &common)?;
                let links = docs_links_payload(&ctx, &common)?;
                let lint = docs_lint_payload(&ctx, &common)?;
                let mut rows = Vec::<serde_json::Value>::new();
                rows.push(serde_json::json!({"name":"validate","errors":validate["errors"].as_array().map(|v| v.len()).unwrap_or(0)}));
                rows.push(serde_json::json!({"name":"links","errors":links["errors"].as_array().map(|v| v.len()).unwrap_or(0)}));
                rows.push(serde_json::json!({"name":"lint","errors":lint["errors"].as_array().map(|v| v.len()).unwrap_or(0)}));
                let mut build_status = "skipped";
                if common.allow_subprocess && common.allow_write {
                    let (_payload, code) = docs_build_or_serve_subprocess(&["build".to_string()], &common, "docs build")?;
                    build_status = if code == 0 { "ok" } else { "failed" };
                }
                rows.push(serde_json::json!({"name":"build","status":build_status}));
                let errors = validate["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + links["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + lint["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + usize::from(build_status == "failed");
                let payload = serde_json::json!({
                    "schema_version":1,
                    "run_id":ctx.run_id.as_str(),
                    "text": if errors==0 {
                        format!("docs: 4 checks collected, 0 failed, build={build_status}")
                    } else {
                        format!("docs: 4 checks collected, {errors} failed, build={build_status}")
                    },
                    "rows":rows,
                    "counts":{"errors":errors},
                    "capabilities":{"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "options":{"strict": common.strict, "include_drafts": common.include_drafts},
                    "duration_ms": started.elapsed().as_millis() as u64,
                    "error_code": if errors == 0 { serde_json::Value::Null } else { serde_json::Value::String("DOCS_NAV_ERROR".to_string()) }
                });
                Ok((emit_payload(common.format, common.out, &payload)?, if errors == 0 {0} else {1}))
            }
        }
    })();
    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                if code == 0 { println!("{rendered}"); } else { eprintln!("{rendered}"); }
            }
            code
        }
        Err(err) => {
            eprintln!("bijux-dev-atlas docs failed: {err}");
            1
        }
    }
}

fn normalize_tool_version_with_regex(raw: &str, pattern: &str) -> Option<String> {
    let re = Regex::new(pattern).ok()?;
    re.captures(raw)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

fn resolve_ops_root(
    repo_root: &Path,
    ops_root: Option<PathBuf>,
) -> Result<PathBuf, OpsCommandError> {
    let path = ops_root.unwrap_or_else(|| repo_root.join("ops"));
    path.canonicalize().map_err(|err| {
        OpsCommandError::Manifest(format!("cannot resolve ops root {}: {err}", path.display()))
    })
}

fn load_profiles(ops_root: &Path) -> Result<Vec<StackProfile>, OpsCommandError> {
    let path = ops_root.join("stack/profiles.json");
    let text = std::fs::read_to_string(&path).map_err(|err| {
        OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display()))
    })?;
    let payload: StackProfiles = serde_json::from_str(&text).map_err(|err| {
        OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display()))
    })?;
    Ok(payload.profiles)
}

fn load_toolchain_inventory(ops_root: &Path) -> Result<ToolchainInventory, OpsCommandError> {
    let path = ops_root.join("inventory/toolchain.json");
    let text = std::fs::read_to_string(&path).map_err(|err| {
        OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display()))
    })?;
    serde_json::from_str(&text)
        .map_err(|err| OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display())))
}

fn tool_definitions_sorted(inventory: &ToolchainInventory) -> Vec<(String, ToolDefinition)> {
    inventory
        .tools
        .iter()
        .map(|(name, definition)| (name.clone(), definition.clone()))
        .collect()
}

fn resolve_profile(
    requested: Option<String>,
    profiles: &[StackProfile],
) -> Result<StackProfile, OpsCommandError> {
    if profiles.is_empty() {
        return Err(OpsCommandError::Profile(
            "no profiles declared in ops/stack/profiles.json".to_string(),
        ));
    }
    if let Some(name) = requested {
        return profiles
            .iter()
            .find(|p| p.name == name)
            .cloned()
            .ok_or_else(|| OpsCommandError::Profile(format!("unknown profile `{name}`")));
    }
    profiles
        .iter()
        .find(|p| p.name == "developer")
        .cloned()
        .or_else(|| profiles.first().cloned())
        .ok_or_else(|| OpsCommandError::Profile("no default profile available".to_string()))
}

fn run_id_or_default(raw: Option<String>) -> Result<RunId, String> {
    raw.map(|v| RunId::parse(&v))
        .transpose()?
        .map_or_else(|| Ok(RunId::from_seed("ops_run")), Ok)
}

fn emit_payload(
    format: FormatArg,
    out: Option<PathBuf>,
    payload: &serde_json::Value,
) -> Result<String, String> {
    let rendered = match format {
        FormatArg::Text => payload
            .get("text")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .unwrap_or_else(|| serde_json::to_string_pretty(payload).unwrap_or_default()),
        FormatArg::Json => serde_json::to_string_pretty(payload).map_err(|err| err.to_string())?,
        FormatArg::Jsonl => {
            if let Some(rows) = payload.get("rows").and_then(|v| v.as_array()) {
                rows.iter()
                    .map(serde_json::to_string)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|err| err.to_string())?
                    .join("\n")
            } else {
                serde_json::to_string(payload).map_err(|err| err.to_string())?
            }
        }
    };
    write_output_if_requested(out, &rendered)?;
    Ok(rendered)
}

fn sha256_hex(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn validate_render_output(rendered: &str, target: OpsRenderTarget) -> Vec<String> {
    let mut errors = Vec::new();
    let required_kinds = match target {
        OpsRenderTarget::Helm => ["Namespace", "Deployment", "Service"].to_vec(),
        OpsRenderTarget::Kind | OpsRenderTarget::Kustomize => Vec::new(),
    };
    for kind in required_kinds {
        let needle = format!("kind: {kind}");
        if !rendered.contains(&needle) {
            errors.push(format!("missing required rendered resource `{needle}`"));
        }
    }
    if rendered.contains("kind: ClusterRole") {
        errors.push("rendered output includes forbidden resource `kind: ClusterRole`".to_string());
    }
    for line in rendered.lines() {
        if line.trim_start().starts_with("image:") && line.contains(":latest") {
            errors.push(format!(
                "rendered image uses forbidden latest tag: {}",
                line.trim()
            ));
        }
    }
    for marker in ["generatedAt:", "timestamp:", "creationTimestamp:"] {
        if rendered.contains(marker) {
            errors.push(format!(
                "render output contains forbidden timestamp marker `{marker}`"
            ));
        }
    }
    errors.sort();
    errors.dedup();
    errors
}

fn validate_helm_dependencies(ops_root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let chart_dir = ops_root.join("k8s/charts/bijux-atlas");
    let chart_yaml_path = chart_dir.join("Chart.yaml");
    let chart_yaml = match fs::read_to_string(&chart_yaml_path) {
        Ok(value) => value,
        Err(err) => {
            return vec![format!(
                "failed to read {}: {err}",
                chart_yaml_path.display()
            )];
        }
    };
    if chart_yaml.contains("\ndependencies:") {
        let lock_path = chart_dir.join("Chart.lock");
        if !lock_path.exists() {
            errors.push(format!(
                "helm dependencies are declared but {} is missing",
                lock_path.display()
            ));
        }
    }
    errors
}

fn render_profile_artifact_base(profile: &str, target: OpsRenderTarget) -> String {
    let target = match target {
        OpsRenderTarget::Helm => "helm",
        OpsRenderTarget::Kustomize => "kustomize",
        OpsRenderTarget::Kind => "kind",
    };
    format!("render/{profile}/{target}")
}

fn expected_kind_context(profile: &StackProfile) -> String {
    format!("kind-{}", profile.kind_profile)
}

fn ensure_kind_context(
    process: &OpsProcess,
    profile: &StackProfile,
    force: bool,
) -> Result<(), OpsCommandError> {
    let args = vec!["config".to_string(), "current-context".to_string()];
    let (stdout, _) = process.run_subprocess("kubectl", &args, Path::new("."))?;
    let current = stdout.trim();
    let expected = expected_kind_context(profile);
    if current == expected || force {
        Ok(())
    } else {
        Err(OpsCommandError::Effect(format!(
            "kubectl context guard failed: expected `{expected}` got `{current}`; pass --force to override"
        )))
    }
}

fn ensure_namespace_exists(
    process: &OpsProcess,
    namespace: &str,
    dry_run: &str,
) -> Result<(), OpsCommandError> {
    let get_args = vec![
        "get".to_string(),
        "namespace".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "name".to_string(),
    ];
    if process
        .run_subprocess("kubectl", &get_args, Path::new("."))
        .is_ok()
    {
        return Ok(());
    }
    let mut create_args = vec![
        "create".to_string(),
        "namespace".to_string(),
        namespace.to_string(),
    ];
    if dry_run == "client" {
        create_args.push("--dry-run=client".to_string());
    }
    let _ = process.run_subprocess("kubectl", &create_args, Path::new("."))?;
    Ok(())
}

fn run_ops_checks(
    common: &OpsCommonArgs,
    suite: &str,
    include_internal: bool,
    include_slow: bool,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let selectors = parse_selectors(
        Some(suite.to_string()),
        Some(DomainArg::Ops),
        None,
        None,
        include_internal,
        include_slow,
    )?;
    let request = RunRequest {
        repo_root: repo_root.clone(),
        domain: Some(DomainId::Ops),
        capabilities: Capabilities::deny_all(),
        artifacts_root: Some(repo_root.join("artifacts")),
        run_id: Some(run_id_or_default(common.run_id.clone())?),
        command: Some(format!("bijux dev atlas ops {suite}")),
    };
    let report = run_checks(
        &RealProcessRunner,
        &RealFs,
        &request,
        &selectors,
        &RunOptions::default(),
    )?;
    let rendered = match common.format {
        FormatArg::Text => render_text_with_durations(&report, 10),
        FormatArg::Json => render_json(&report)?,
        FormatArg::Jsonl => render_jsonl(&report)?,
    };
    write_output_if_requested(common.out.clone(), &rendered)?;
    Ok((rendered, exit_code_for_report(&report)))
}

fn verify_tools_snapshot(
    allow_subprocess: bool,
    inventory: &ToolchainInventory,
) -> Result<serde_json::Value, String> {
    if !allow_subprocess {
        return Ok(serde_json::json!({
            "enabled": false,
            "text": "tool verification skipped (pass --allow-subprocess)",
            "missing_required": [],
            "rows": []
        }));
    }
    let process = OpsProcess::new(true);
    let mut rows = Vec::new();
    let mut missing_required = Vec::new();
    for (name, definition) in tool_definitions_sorted(inventory) {
        let mut row = process
            .probe_tool(&name, &definition.probe_argv, &definition.version_regex)
            .map_err(|e| e.to_stable_message())?;
        row["required"] = serde_json::Value::Bool(definition.required);
        if definition.required && row["installed"] != serde_json::Value::Bool(true) {
            missing_required.push(name.clone());
        }
        rows.push(row);
    }
    rows.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    Ok(serde_json::json!({
        "enabled": true,
        "text": if missing_required.is_empty() { "all required tools available" } else { "missing required tools" },
        "missing_required": missing_required,
        "rows": rows
    }))
}

fn render_ops_validation_output(
    common: &OpsCommonArgs,
    mode: &str,
    inventory_errors: &[String],
    checks_rendered: &str,
    checks_exit: i32,
    summary: serde_json::Value,
) -> Result<(String, i32), String> {
    let inventory_error_count = inventory_errors.len();
    let checks_error_count = if checks_exit == 0 { 0 } else { 1 };
    let error_count = inventory_error_count + checks_error_count;
    let status = if error_count == 0 { "ok" } else { "failed" };
    let strict_failed = common.strict && error_count > 0;
    let exit = if strict_failed || checks_exit != 0 || inventory_error_count > 0 {
        1
    } else {
        0
    };
    let text = format!(
        "ops {mode}: status={status} inventory_errors={inventory_error_count} checks_exit={checks_exit}"
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "mode": mode,
        "status": status,
        "text": text,
        "rows": [{
            "inventory_errors": inventory_errors,
            "checks_exit": checks_exit,
            "checks_output": checks_rendered,
            "inventory_summary": summary
        }],
        "summary": {
            "total": 1,
            "errors": error_count,
            "warnings": 0
        }
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit))
}

pub(crate) fn run_ops_command(quiet: bool, debug: bool, command: OpsCommand) -> i32 {
    let run: Result<(String, i32), String> = (|| match command {
        OpsCommand::Doctor(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let inventory_errors = match bijux_dev_atlas_core::ops_inventory::OpsInventory::load_and_validate(&ops_root) {
                Ok(_) => Vec::new(),
                Err(err) => vec![err],
            };
            let summary = ops_inventory_summary(&repo_root).unwrap_or_else(
                |err| serde_json::json!({"error": format!("OPS_MANIFEST_ERROR: {err}")}),
            );
            let (checks_rendered, checks_exit) = run_ops_checks(&common, "ops_fast", false, false)?;
            let toolchain = load_toolchain_inventory(&ops_root).map_err(|e| e.to_stable_message())?;
            let tools_snapshot = verify_tools_snapshot(common.allow_subprocess, &toolchain)?;
            let mut inventory_errors = inventory_errors;
            if tools_snapshot
                .get("missing_required")
                .and_then(|v| v.as_array())
                .is_some_and(|v| !v.is_empty())
            {
                inventory_errors.push("required external tools are missing".to_string());
            }
            let summary = serde_json::json!({
                "inventory": summary,
                "tools": tools_snapshot
            });
            render_ops_validation_output(
                &common,
                "doctor",
                &inventory_errors,
                &checks_rendered,
                checks_exit,
                summary,
            )
        }
        OpsCommand::Validate(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let inventory_errors = match bijux_dev_atlas_core::ops_inventory::OpsInventory::load_and_validate(&ops_root) {
                Ok(_) => Vec::new(),
                Err(err) => vec![err],
            };
            let summary = ops_inventory_summary(&repo_root).unwrap_or_else(
                |err| serde_json::json!({"error": format!("OPS_MANIFEST_ERROR: {err}")}),
            );
            let (checks_rendered, checks_exit) = run_ops_checks(&common, "ops_all", true, true)?;
            render_ops_validation_output(
                &common,
                "validate",
                &inventory_errors,
                &checks_rendered,
                checks_exit,
                summary,
            )
        }
        OpsCommand::Render(args) => {
            let common = &args.common;
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let fs_adapter = OpsFs::new(repo_root.clone(), ops_root.clone());
            let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let profile = resolve_profile(common.profile.clone(), &profiles)
                .map_err(|e| e.to_stable_message())?;
            let run_id = run_id_or_default(common.run_id.clone())?;
            let process = OpsProcess::new(common.allow_subprocess);
            let target_name = match args.target {
                OpsRenderTarget::Helm => "helm",
                OpsRenderTarget::Kustomize => "kustomize",
                OpsRenderTarget::Kind => "kind",
            };

            let (rendered_manifest, subprocess_events): (String, Vec<serde_json::Value>) =
                match args.target {
                    OpsRenderTarget::Helm => {
                        if !common.allow_subprocess {
                            return Err(OpsCommandError::Effect(
                                "helm render requires --allow-subprocess".to_string(),
                            )
                            .to_stable_message());
                        }
                        let helm_binary = args
                            .helm_binary
                            .clone()
                            .unwrap_or_else(|| "helm".to_string());
                        let chart_path = ops_root.join("k8s/charts/bijux-atlas");
                        let values_path = ops_root.join("k8s/charts/bijux-atlas/values.yaml");
                        let cmd_args = vec![
                            "template".to_string(),
                            "bijux-atlas".to_string(),
                            chart_path.display().to_string(),
                            "--namespace".to_string(),
                            "bijux-atlas".to_string(),
                            "-f".to_string(),
                            values_path.display().to_string(),
                        ];
                        let (stdout, event) = process
                            .run_subprocess(&helm_binary, &cmd_args, &repo_root)
                            .map_err(|e| e.to_stable_message())?;
                        (stdout, vec![event])
                    }
                    OpsRenderTarget::Kind => {
                        let cluster_config_path = repo_root.join(&profile.cluster_config);
                        let content = fs::read_to_string(&cluster_config_path).map_err(|err| {
                            OpsCommandError::Manifest(format!(
                                "failed to read cluster config {}: {err}",
                                cluster_config_path.display()
                            ))
                            .to_stable_message()
                        })?;
                        (
                            format!("# source: {}\n{content}", profile.cluster_config),
                            Vec::new(),
                        )
                    }
                    OpsRenderTarget::Kustomize => {
                        return Err(OpsCommandError::Effect(
                            "kustomize render is not enabled; use --target helm or --target kind"
                                .to_string(),
                        )
                        .to_stable_message())
                    }
                };

            let mut validation_errors = validate_render_output(&rendered_manifest, args.target);
            if matches!(args.target, OpsRenderTarget::Helm) {
                validation_errors.extend(validate_helm_dependencies(&ops_root));
            }
            validation_errors.sort();
            validation_errors.dedup();

            let write_enabled = args.write || (!args.check && !args.stdout);
            let rel_base = render_profile_artifact_base(&profile.name, args.target);
            let rel_yaml = format!("{rel_base}/render.yaml");
            let rel_index = format!("{rel_base}/render.index.json");
            let mut written_files = Vec::new();
            let mut rows = Vec::new();

            let render_sha = sha256_hex(&rendered_manifest);
            let manifest_row = serde_json::json!({
                "path": rel_yaml,
                "sha256": render_sha,
                "bytes": rendered_manifest.len(),
            });
            rows.push(manifest_row.clone());

            if write_enabled {
                let yaml_path = repo_root
                    .join("artifacts/atlas-dev/ops")
                    .join(run_id.as_str())
                    .join(&rel_yaml);
                if let Some(parent) = yaml_path.parent() {
                    fs::create_dir_all(parent).map_err(|err| {
                        OpsCommandError::Manifest(format!(
                            "failed to create {}: {err}",
                            parent.display()
                        ))
                        .to_stable_message()
                    })?;
                }
                let mut file = fs::File::create(&yaml_path).map_err(|err| {
                    OpsCommandError::Manifest(format!(
                        "failed to create {}: {err}",
                        yaml_path.display()
                    ))
                    .to_stable_message()
                })?;
                file.write_all(rendered_manifest.as_bytes())
                    .map_err(|err| {
                        OpsCommandError::Manifest(format!(
                            "failed to write {}: {err}",
                            yaml_path.display()
                        ))
                        .to_stable_message()
                    })?;
                written_files.push(rel_yaml.clone());

                let index_payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": run_id.as_str(),
                    "profile": profile.name,
                    "target": target_name,
                    "files": rows
                });
                let index_path = fs_adapter
                    .write_artifact_json(&run_id, &rel_index, &index_payload)
                    .map_err(|e| e.to_stable_message())?;
                written_files.push(
                    index_path
                        .strip_prefix(
                            repo_root
                                .join("artifacts/atlas-dev/ops")
                                .join(run_id.as_str()),
                        )
                        .unwrap_or(index_path.as_path())
                        .display()
                        .to_string(),
                );
            }

            let text = if args.stdout {
                rendered_manifest.clone()
            } else {
                format!(
                    "render target={target_name} profile={} run_id={} wrote={} validation_errors={}",
                    profile.name,
                    run_id.as_str(),
                    write_enabled,
                    validation_errors.len()
                )
            };
            let payload = serde_json::json!({
                "schema_version": 1,
                "text": text,
                "rows": [{
                    "repo_root": repo_root.display().to_string(),
                    "ops_root": ops_root.display().to_string(),
                    "profile": profile.name,
                    "kind_profile": profile.kind_profile,
                    "cluster_config": profile.cluster_config,
                    "run_id": run_id.as_str(),
                    "target": target_name,
                    "write_enabled": write_enabled,
                    "check_only": args.check,
                    "stdout_mode": args.stdout,
                    "diff_mode": args.diff,
                    "written_files": written_files,
                    "render_index_files": rows,
                    "validation_errors": validation_errors,
                    "subprocess_events": subprocess_events
                }],
                "summary": {
                    "total": 1,
                    "errors": if validation_errors.is_empty() { 0 } else { validation_errors.len() },
                    "warnings": 0
                }
            });
            let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
            let exit = if validation_errors.is_empty() { 0 } else { 1 };
            Ok((rendered, exit))
        }
        OpsCommand::Install(args) => {
            let common = &args.common;
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let profile = resolve_profile(common.profile.clone(), &profiles)
                .map_err(|e| e.to_stable_message())?;
            let run_id = run_id_or_default(common.run_id.clone())?;
            if !args.plan && !common.allow_subprocess {
                return Err(OpsCommandError::Effect(
                    "install execution requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            if (args.apply || args.kind) && !common.allow_write {
                return Err(OpsCommandError::Effect(
                    "install apply/kind requires --allow-write".to_string(),
                )
                .to_stable_message());
            }

            let mut steps = Vec::new();
            let process = OpsProcess::new(common.allow_subprocess);
            if args.kind {
                steps.push("kind cluster ensure".to_string());
                if !args.plan {
                    let kind_config = repo_root.join(&profile.cluster_config);
                    let kind_args = vec![
                        "create".to_string(),
                        "cluster".to_string(),
                        "--name".to_string(),
                        profile.kind_profile.clone(),
                        "--config".to_string(),
                        kind_config.display().to_string(),
                    ];
                    let _ = process
                        .run_subprocess("kind", &kind_args, &repo_root)
                        .map_err(|e| e.to_stable_message())?;
                }
            }
            if args.apply {
                steps.push("kubectl apply".to_string());
                if !args.plan {
                    ensure_kind_context(&process, &profile, args.force)
                        .map_err(|e| e.to_stable_message())?;
                    ensure_namespace_exists(&process, "bijux-atlas", &args.dry_run)
                        .map_err(|e| e.to_stable_message())?;
                    let render_path = repo_root
                        .join("artifacts/atlas-dev/ops")
                        .join(run_id.as_str())
                        .join(format!("render/{}/helm/render.yaml", profile.name));
                    let mut apply_args = vec![
                        "apply".to_string(),
                        "-n".to_string(),
                        "bijux-atlas".to_string(),
                        "-f".to_string(),
                        render_path.display().to_string(),
                    ];
                    if args.dry_run == "client" {
                        apply_args.push("--dry-run=client".to_string());
                    }
                    let _ = process
                        .run_subprocess("kubectl", &apply_args, &repo_root)
                        .map_err(|e| e.to_stable_message())?;
                }
            }
            if !args.kind && !args.apply {
                steps.push("validate-only".to_string());
            }
            let payload = serde_json::json!({
                "schema_version": 1,
                "profile": profile.name,
                "run_id": run_id.as_str(),
                "plan_mode": args.plan,
                "dry_run": args.dry_run,
                "steps": steps,
                "kind_context_expected": expected_kind_context(&profile),
            });
            let text = if args.plan {
                format!("install plan generated for profile `{}`", profile.name)
            } else {
                format!("install completed for profile `{}`", profile.name)
            };
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": [payload], "summary": {"total": 1, "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::Status(args) => {
            let common = &args.common;
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let profile = resolve_profile(common.profile.clone(), &profiles)
                .map_err(|e| e.to_stable_message())?;
            let process = OpsProcess::new(common.allow_subprocess);
            let (payload, text) = match args.target {
                OpsStatusTarget::Local => {
                    let toolchain_path = ops_root.join("inventory/toolchain.json");
                    let toolchain = std::fs::read_to_string(&toolchain_path).map_err(|err| {
                        OpsCommandError::Manifest(format!(
                            "failed to read {}: {err}",
                            toolchain_path.display()
                        ))
                        .to_stable_message()
                    })?;
                    let toolchain_json: serde_json::Value = serde_json::from_str(&toolchain)
                        .map_err(|err| {
                            OpsCommandError::Schema(format!(
                                "failed to parse {}: {err}",
                                toolchain_path.display()
                            ))
                            .to_stable_message()
                        })?;
                    (
                        serde_json::json!({
                            "schema_version": 1,
                            "target": "local",
                            "repo_root": repo_root.display().to_string(),
                            "ops_root": ops_root.display().to_string(),
                            "profile": profile,
                            "toolchain": toolchain_json,
                        }),
                        format!(
                            "ops status local: profile={} repo_root={} ops_root={}",
                            profile.name,
                            repo_root.display(),
                            ops_root.display(),
                        ),
                    )
                }
                OpsStatusTarget::K8s => {
                    if !common.allow_subprocess {
                        return Err(OpsCommandError::Effect(
                            "status k8s requires --allow-subprocess".to_string(),
                        )
                        .to_stable_message());
                    }
                    let args = vec![
                        "get".to_string(),
                        "all".to_string(),
                        "-n".to_string(),
                        "bijux-atlas".to_string(),
                        "-o".to_string(),
                        "json".to_string(),
                    ];
                    let (stdout, _) = process
                        .run_subprocess("kubectl", &args, &repo_root)
                        .map_err(|e| e.to_stable_message())?;
                    let value: serde_json::Value =
                        serde_json::from_str(&stdout).map_err(|err| {
                            OpsCommandError::Schema(format!("failed to parse kubectl json: {err}"))
                                .to_stable_message()
                        })?;
                    (
                        serde_json::json!({
                            "schema_version": 1,
                            "target": "k8s",
                            "profile": profile.name,
                            "resources": value
                        }),
                        "ops status k8s collected".to_string(),
                    )
                }
                OpsStatusTarget::Pods => {
                    if !common.allow_subprocess {
                        return Err(OpsCommandError::Effect(
                            "status pods requires --allow-subprocess".to_string(),
                        )
                        .to_stable_message());
                    }
                    let args = vec![
                        "get".to_string(),
                        "pods".to_string(),
                        "-n".to_string(),
                        "bijux-atlas".to_string(),
                        "-o".to_string(),
                        "json".to_string(),
                    ];
                    let (stdout, _) = process
                        .run_subprocess("kubectl", &args, &repo_root)
                        .map_err(|e| e.to_stable_message())?;
                    let value: serde_json::Value =
                        serde_json::from_str(&stdout).map_err(|err| {
                            OpsCommandError::Schema(format!("failed to parse kubectl json: {err}"))
                                .to_stable_message()
                        })?;
                    let mut pods = value
                        .get("items")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    pods.sort_by(|a, b| {
                        a.get("metadata")
                            .and_then(|m| m.get("name"))
                            .and_then(|v| v.as_str())
                            .cmp(
                                &b.get("metadata")
                                    .and_then(|m| m.get("name"))
                                    .and_then(|v| v.as_str()),
                            )
                    });
                    (
                        serde_json::json!({
                            "schema_version": 1,
                            "target": "pods",
                            "profile": profile.name,
                            "pods": pods
                        }),
                        "ops status pods collected".to_string(),
                    )
                }
                OpsStatusTarget::Endpoints => {
                    if !common.allow_subprocess {
                        return Err(OpsCommandError::Effect(
                            "status endpoints requires --allow-subprocess".to_string(),
                        )
                        .to_stable_message());
                    }
                    let args = vec![
                        "get".to_string(),
                        "endpoints".to_string(),
                        "-n".to_string(),
                        "bijux-atlas".to_string(),
                        "-o".to_string(),
                        "json".to_string(),
                    ];
                    let (stdout, _) = process
                        .run_subprocess("kubectl", &args, &repo_root)
                        .map_err(|e| e.to_stable_message())?;
                    let value: serde_json::Value =
                        serde_json::from_str(&stdout).map_err(|err| {
                            OpsCommandError::Schema(format!("failed to parse kubectl json: {err}"))
                                .to_stable_message()
                        })?;
                    (
                        serde_json::json!({
                            "schema_version": 1,
                            "target": "endpoints",
                            "profile": profile.name,
                            "resources": value
                        }),
                        "ops status endpoints collected".to_string(),
                    )
                }
            };
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": [payload], "summary": {"total": 1, "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::ListProfiles(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let rows = profiles
                .iter()
                .map(|p| serde_json::json!({"name": p.name, "kind_profile": p.kind_profile, "cluster_config": p.cluster_config}))
                .collect::<Vec<_>>();
            let text = profiles
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join("\n");
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": rows, "summary": {"total": profiles.len(), "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::ExplainProfile { name, common } => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let profile =
                resolve_profile(Some(name), &profiles).map_err(|e| e.to_stable_message())?;
            let text = format!(
                "profile={} kind_profile={} cluster_config={}",
                profile.name, profile.kind_profile, profile.cluster_config
            );
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": [profile], "summary": {"total": 1, "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::ListTools(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let inventory = load_toolchain_inventory(&ops_root).map_err(|e| e.to_stable_message())?;
            let process = OpsProcess::new(common.allow_subprocess);
            let mut rows = Vec::new();
            for (name, definition) in tool_definitions_sorted(&inventory) {
                let mut row = process
                    .probe_tool(&name, &definition.probe_argv, &definition.version_regex)
                    .map_err(|e| e.to_stable_message())?;
                row["required"] = serde_json::Value::Bool(definition.required);
                rows.push(row);
            }
            rows.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
            let text = rows
                .iter()
                .map(|r| {
                    format!(
                        "{} required={} installed={}",
                        r["name"].as_str().unwrap_or(""),
                        r["required"],
                        r["installed"]
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": rows, "summary": {"total": rows.len(), "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::VerifyTools(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let inventory = load_toolchain_inventory(&ops_root).map_err(|e| e.to_stable_message())?;
            let process = OpsProcess::new(common.allow_subprocess);
            let mut rows = Vec::new();
            let mut missing = Vec::new();
            let mut warnings = Vec::new();
            for (name, definition) in tool_definitions_sorted(&inventory) {
                let row = process
                    .probe_tool(&name, &definition.probe_argv, &definition.version_regex)
                    .map_err(|e| e.to_stable_message())?;
                if row["installed"] == serde_json::Value::Bool(false) {
                    if definition.required {
                        missing.push(name.clone());
                    } else {
                        warnings.push(name.clone());
                    }
                }
                rows.push(row);
            }
            rows.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
            let text = if missing.is_empty() {
                "all required ops tools are installed".to_string()
            } else {
                format!("missing required ops tools: {}", missing.join(", "))
            };
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": rows, "missing": missing, "warnings": warnings, "summary": {"total": rows.len(), "errors": missing.len(), "warnings": warnings.len()}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            let has_errors = !envelope["missing"]
                .as_array()
                .map(|v| v.is_empty())
                .unwrap_or(true);
            let has_warnings = !envelope["warnings"]
                .as_array()
                .map(|v| v.is_empty())
                .unwrap_or(true);
            let code = if has_errors || (common.strict && has_warnings) {
                1
            } else {
                0
            };
            Ok((rendered, code))
        }
        OpsCommand::ListActions(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let fs_adapter = OpsFs::new(repo_root, ops_root);
            let mut payload: SurfacesInventory = fs_adapter
                .read_ops_json("inventory/surfaces.json")
                .map_err(|e| e.to_stable_message())?;
            payload.actions.sort_by(|a, b| a.id.cmp(&b.id));
            let rows = payload.actions.iter()
                .map(|a| serde_json::json!({"id": a.id, "domain": a.domain, "command": a.command, "argv": a.argv}))
                .collect::<Vec<_>>();
            let text = payload
                .actions
                .iter()
                .map(|a| a.id.clone())
                .collect::<Vec<_>>()
                .join("\n");
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": rows, "summary": {"total": payload.actions.len(), "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::Up(common) => {
            if !common.allow_subprocess {
                return Err(
                    OpsCommandError::Effect("up requires --allow-subprocess".to_string())
                        .to_stable_message(),
                );
            }
            if !common.allow_write {
                return Err(
                    OpsCommandError::Effect("up requires --allow-write".to_string())
                        .to_stable_message(),
                );
            }
            let text = "ops up delegates to install --kind --apply --plan".to_string();
            let rendered = emit_payload(
                common.format,
                common.out.clone(),
                &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 0, "errors": 0, "warnings": 0}}),
            )?;
            Ok((rendered, 0))
        }
        OpsCommand::Down(common) => {
            if !common.allow_subprocess {
                return Err(OpsCommandError::Effect(
                    "down requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let profile = resolve_profile(common.profile.clone(), &profiles)
                .map_err(|e| e.to_stable_message())?;
            let process = OpsProcess::new(common.allow_subprocess);
            let args = vec![
                "delete".to_string(),
                "cluster".to_string(),
                "--name".to_string(),
                profile.kind_profile.clone(),
            ];
            let _ = process
                .run_subprocess("kind", &args, &repo_root)
                .map_err(|e| e.to_stable_message())?;
            let text = format!("ops down deleted kind cluster `{}`", profile.kind_profile);
            let rendered = emit_payload(
                common.format,
                common.out.clone(),
                &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 0, "errors": 0, "warnings": 0}}),
            )?;
            Ok((rendered, 0))
        }
        OpsCommand::Clean(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let path = repo_root.join("artifacts/atlas-dev/ops");
            if path.exists() {
                std::fs::remove_dir_all(&path)
                    .map_err(|err| format!("failed to remove {}: {err}", path.display()))?;
            }
            let text = format!("cleaned {}", path.display());
            let rendered = emit_payload(
                common.format,
                common.out.clone(),
                &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 0, "errors": 0, "warnings": 0}}),
            )?;
            Ok((rendered, 0))
        }
        OpsCommand::Reset(args) => {
            let common = &args.common;
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let run_id = RunId::parse(&args.reset_id).map_err(|err| err.to_string())?;
            let target = repo_root
                .join("artifacts/atlas-dev/ops")
                .join(run_id.as_str());
            if !target.starts_with(repo_root.join("artifacts/atlas-dev/ops")) {
                return Err("reset path guard failed".to_string());
            }
            if target.exists() {
                std::fs::remove_dir_all(&target)
                    .map_err(|err| format!("failed to remove {}: {err}", target.display()))?;
            }
            let text = format!(
                "reset artifacts for run_id={} at {}",
                run_id.as_str(),
                target.display()
            );
            let rendered = emit_payload(
                common.format,
                common.out.clone(),
                &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 1, "errors": 0, "warnings": 0}}),
            )?;
            Ok((rendered, 0))
        }
        OpsCommand::Pins { command } => match command {
            OpsPinsCommand::Check(common) => {
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let path = repo_root.join("ops/inventory/toolchain.json");
                let ok = path.exists();
                let text = if ok {
                    format!("pins check passed: {}", path.display())
                } else {
                    format!("pins check failed: missing {}", path.display())
                };
                let rendered = emit_payload(
                    common.format,
                    common.out.clone(),
                    &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 1, "errors": if ok {0} else {1}, "warnings": 0}}),
                )?;
                Ok((rendered, if ok { 0 } else { 1 }))
            }
            OpsPinsCommand::Update {
                i_know_what_im_doing,
                common,
            } => {
                if !i_know_what_im_doing {
                    Err("ops pins update requires --i-know-what-im-doing".to_string())
                } else if !common.allow_subprocess {
                    Err(OpsCommandError::Effect(
                        "pins update requires --allow-subprocess".to_string(),
                    )
                    .to_stable_message())
                } else {
                    let text =
                        "ops pins update is migration-gated; no mutation performed".to_string();
                    let rendered = emit_payload(
                        common.format,
                        common.out.clone(),
                        &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 1, "errors": 0, "warnings": 0}}),
                    )?;
                    Ok((rendered, 0))
                }
            }
        },
        OpsCommand::Generate { command } => match command {
            OpsGenerateCommand::PinsIndex(common) => {
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let run_id = run_id_or_default(common.run_id.clone())?;
                let fs_adapter = OpsFs::new(repo_root.clone(), repo_root.join("ops"));
                let pins_rel = "ops/inventory/pins.yaml";
                let toolchain_rel = "ops/inventory/toolchain.json";
                let stack_rel = "ops/stack/version-manifest.json";
                let pins_raw = fs::read_to_string(repo_root.join(pins_rel))
                    .map_err(|err| format!("failed to read {pins_rel}: {err}"))?;
                let toolchain_raw = fs::read_to_string(repo_root.join(toolchain_rel))
                    .map_err(|err| format!("failed to read {toolchain_rel}: {err}"))?;
                let stack_raw = fs::read_to_string(repo_root.join(stack_rel))
                    .map_err(|err| format!("failed to read {stack_rel}: {err}"))?;
                let mut files = vec![
                    serde_json::json!({"path": pins_rel, "sha256": sha256_hex(&pins_raw), "bytes": pins_raw.len()}),
                    serde_json::json!({"path": stack_rel, "sha256": sha256_hex(&stack_raw), "bytes": stack_raw.len()}),
                    serde_json::json!({"path": toolchain_rel, "sha256": sha256_hex(&toolchain_raw), "bytes": toolchain_raw.len()}),
                ];
                files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": run_id.as_str(),
                    "generator": "ops generate pins-index",
                    "files": files
                });
                let rel = "generate/pins.index.json";
                let out = fs_adapter
                    .write_artifact_json(&run_id, rel, &payload)
                    .map_err(|e| e.to_stable_message())?;
                let text = format!("generated deterministic pins index at {}", out.display());
                let rendered = emit_payload(
                    common.format,
                    common.out.clone(),
                    &serde_json::json!({"schema_version": 1, "text": text, "rows": [payload], "summary": {"total": 1, "errors": 0, "warnings": 0}}),
                )?;
                Ok((rendered, 0))
            }
        },
    })();

    if debug {
        eprintln!(
            "{}",
            serde_json::json!({
                "event": "ops.command.completed",
                "ok": run.is_ok(),
            })
        );
    }

    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                println!("{rendered}");
            }
            code
        }
        Err(err) => {
            eprintln!("bijux-dev-atlas ops failed: {err}");
            1
        }
    }
}

fn main() {
    let cli = Cli::parse();
    std::process::exit(dispatch::run_cli(cli));
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn source_does_not_reference_atlasctl_runtime() {
        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let forbidden_python_module = ["python -m ", "atlasctl"].concat();
        let forbidden_wrapper = ["/bin/", "atlasctl"].concat();
        let mut stack = vec![src];
        while let Some(path) = stack.pop() {
            for entry in fs::read_dir(path).expect("read_dir") {
                let entry = entry.expect("entry");
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().and_then(|v| v.to_str()) != Some("rs") {
                    continue;
                }
                let text = fs::read_to_string(&path).expect("read file");
                assert!(
                    !text.contains(&forbidden_python_module),
                    "new rust dev tool must not invoke python atlas runtime: {}",
                    path.display()
                );
                assert!(
                    !text.contains(&forbidden_wrapper),
                    "new rust dev tool must not invoke atlasctl binary wrapper: {}",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn ops_subcommands_parse() {
        let commands = [
            vec!["bijux-dev-atlas", "ops", "doctor"],
            vec!["bijux-dev-atlas", "ops", "validate"],
            vec!["bijux-dev-atlas", "ops", "render", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "ops", "install", "--plan"],
            vec!["bijux-dev-atlas", "ops", "status"],
            vec![
                "bijux-dev-atlas",
                "ops",
                "status",
                "--target",
                "k8s",
                "--allow-subprocess",
            ],
            vec!["bijux-dev-atlas", "ops", "list-profiles"],
            vec!["bijux-dev-atlas", "ops", "explain-profile", "kind"],
            vec!["bijux-dev-atlas", "ops", "list-tools", "--allow-subprocess"],
            vec![
                "bijux-dev-atlas",
                "ops",
                "verify-tools",
                "--allow-subprocess",
            ],
            vec!["bijux-dev-atlas", "ops", "list-actions"],
            vec![
                "bijux-dev-atlas",
                "ops",
                "up",
                "--allow-subprocess",
                "--allow-write",
            ],
            vec!["bijux-dev-atlas", "ops", "down", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "ops", "clean"],
            vec![
                "bijux-dev-atlas",
                "ops",
                "reset",
                "--reset-run-id",
                "ops_reset",
            ],
            vec!["bijux-dev-atlas", "ops", "pins", "check"],
            vec![
                "bijux-dev-atlas",
                "ops",
                "pins",
                "update",
                "--allow-subprocess",
                "--i-know-what-im-doing",
            ],
            vec!["bijux-dev-atlas", "ops", "generate", "pins-index"],
        ];
        for argv in commands {
            let cli = super::Cli::try_parse_from(argv).expect("parse");
            match cli.command {
                Some(super::cli::Command::Ops { .. }) => {}
                _ => panic!("expected ops command"),
            }
        }
    }

    #[test]
    fn check_subcommands_parse() {
        let commands = [
            vec!["bijux-dev-atlas", "check", "list"],
            vec![
                "bijux-dev-atlas",
                "check",
                "explain",
                "checks_ops_surface_manifest",
            ],
            vec!["bijux-dev-atlas", "check", "doctor"],
            vec!["bijux-dev-atlas", "check", "run", "--suite", "ci_fast"],
        ];
        for argv in commands {
            let cli = super::Cli::try_parse_from(argv).expect("parse");
            match cli.command {
                Some(super::cli::Command::Check { .. }) => {}
                _ => panic!("expected check command"),
            }
        }
    }

    #[test]
    fn docs_subcommands_parse() {
        let commands = [
            vec!["bijux-dev-atlas", "docs", "doctor"],
            vec!["bijux-dev-atlas", "docs", "validate"],
            vec!["bijux-dev-atlas", "docs", "lint"],
            vec!["bijux-dev-atlas", "docs", "links"],
            vec!["bijux-dev-atlas", "docs", "inventory"],
            vec!["bijux-dev-atlas", "docs", "grep", "atlasctl"],
            vec!["bijux-dev-atlas", "docs", "build", "--allow-subprocess", "--allow-write", "--strict"],
            vec!["bijux-dev-atlas", "docs", "serve", "--allow-subprocess", "--include-drafts"],
        ];
        for argv in commands {
            let cli = super::Cli::try_parse_from(argv).expect("parse");
            match cli.command {
                Some(super::cli::Command::Docs { .. }) => {}
                _ => panic!("expected docs command"),
            }
        }
    }

    #[test]
    fn mkdocs_nav_parser_extracts_refs() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/docs-mini");
        let refs = super::mkdocs_nav_refs(&root).expect("mkdocs nav");
        let paths = refs.into_iter().map(|(_, p)| p).collect::<Vec<_>>();
        assert_eq!(paths, vec!["index.md".to_string(), "sub/intro.md".to_string()]);
    }

    #[test]
    fn docs_link_resolver_accepts_fixture_links() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/docs-mini");
        let ctx = super::DocsContext {
            docs_root: repo_root.join("docs"),
            artifacts_root: repo_root.join("artifacts"),
            run_id: super::RunId::from_seed("docs_fixture"),
            repo_root: repo_root.clone(),
        };
        let common = super::cli::DocsCommonArgs {
            repo_root: Some(repo_root),
            artifacts_root: None,
            run_id: None,
            format: super::cli::FormatArg::Json,
            out: None,
            allow_subprocess: false,
            allow_write: false,
            allow_network: false,
            strict: false,
            include_drafts: false,
        };
        let payload = super::docs_links_payload(&ctx, &common).expect("links payload");
        assert_eq!(
            payload.get("errors").and_then(|v| v.as_array()).map(|v| v.len()),
            Some(0)
        );
        assert_eq!(
            payload
                .get("external_link_check")
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
    }
}
