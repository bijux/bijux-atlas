#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use bijux_dev_atlas_adapters::{Capabilities, RealFs, RealProcessRunner};
use bijux_dev_atlas_core::{
    exit_code_for_report, explain_output, list_output, load_registry, registry_doctor, render_json,
    render_jsonl, render_text_with_durations, run_checks, select_checks, RunOptions, RunRequest,
    Selectors,
};
use bijux_dev_atlas_model::{CheckId, DomainId, RunId, SuiteId, Tag};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "bijux-dev-atlas", version)]
#[command(about = "Bijux Atlas development control-plane")]
struct Cli {
    #[arg(long, default_value_t = false)]
    quiet: bool,
    #[arg(long, default_value_t = false)]
    verbose: bool,
    #[arg(long, default_value_t = false)]
    debug: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    List {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long)]
        suite: Option<String>,
        #[arg(long, value_enum)]
        domain: Option<DomainArg>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long, value_name = "GLOB")]
        id: Option<String>,
        #[arg(long, default_value_t = false)]
        include_internal: bool,
        #[arg(long, default_value_t = false)]
        include_slow: bool,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Explain {
        check_id: String,
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Doctor {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Run {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long)]
        artifacts_root: Option<PathBuf>,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long)]
        suite: Option<String>,
        #[arg(long, value_enum)]
        domain: Option<DomainArg>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long, value_name = "GLOB")]
        id: Option<String>,
        #[arg(long, default_value_t = false)]
        include_internal: bool,
        #[arg(long, default_value_t = false)]
        include_slow: bool,
        #[arg(long, default_value_t = false)]
        allow_subprocess: bool,
        #[arg(long, default_value_t = false)]
        allow_git: bool,
        #[arg(long = "allow-write", default_value_t = false)]
        allow_write: bool,
        #[arg(long, default_value_t = false)]
        allow_network: bool,
        #[arg(long, default_value_t = false)]
        fail_fast: bool,
        #[arg(long)]
        max_failures: Option<usize>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value_t = 0)]
        durations: usize,
    },
    Ops {
        #[command(subcommand)]
        command: OpsCommand,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DomainArg {
    Ops,
    Repo,
    Docs,
    Make,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum FormatArg {
    Text,
    Json,
    Jsonl,
}

#[derive(Subcommand, Debug)]
enum OpsCommand {
    Doctor(OpsCommonArgs),
    Validate(OpsCommonArgs),
    Render(OpsCommonArgs),
    Install(OpsCommonArgs),
    Status(OpsCommonArgs),
    ListProfiles(OpsCommonArgs),
    ExplainProfile {
        name: String,
        #[command(flatten)]
        common: OpsCommonArgs,
    },
    ListTools(OpsCommonArgs),
    VerifyTools(OpsCommonArgs),
    ListActions(OpsCommonArgs),
    Up(OpsCommonArgs),
    Down(OpsCommonArgs),
    Clean(OpsCommonArgs),
    Pins {
        #[command(subcommand)]
        command: OpsPinsCommand,
    },
}

#[derive(Subcommand, Debug)]
enum OpsPinsCommand {
    Check(OpsCommonArgs),
    Update {
        #[arg(long, default_value_t = false)]
        i_know_what_im_doing: bool,
        #[command(flatten)]
        common: OpsCommonArgs,
    },
}

#[derive(Args, Debug, Clone)]
struct OpsCommonArgs {
    #[arg(long)]
    repo_root: Option<PathBuf>,
    #[arg(long)]
    ops_root: Option<PathBuf>,
    #[arg(long)]
    profile: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    format: FormatArg,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    run_id: Option<String>,
    #[arg(long, default_value_t = false)]
    strict: bool,
    #[arg(long, default_value_t = false)]
    allow_subprocess: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct StackProfiles {
    profiles: Vec<StackProfile>,
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
        Self { repo_root, ops_root }
    }

    fn read_ops_json<T: for<'de> Deserialize<'de>>(&self, rel: &str) -> Result<T, OpsCommandError> {
        let path = self.ops_root.join(rel);
        let text = std::fs::read_to_string(&path).map_err(|err| {
            OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display()))
        })?;
        serde_json::from_str(&text)
            .map_err(|err| OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display())))
    }

    fn write_artifact_json(&self, run_id: &RunId, rel: &str, payload: &serde_json::Value) -> Result<PathBuf, OpsCommandError> {
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
        std::fs::write(&path, content)
            .map_err(|err| OpsCommandError::Manifest(format!("failed to write {}: {err}", path.display())))?;
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

    fn probe_tool(&self, name: &str) -> Result<serde_json::Value, OpsCommandError> {
        if !self.allow_subprocess {
            return Err(OpsCommandError::Effect(
                "subprocess is denied; pass --allow-subprocess".to_string(),
            ));
        }
        match ProcessCommand::new(name).arg("--version").output() {
            Ok(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                let version = text.lines().next().unwrap_or("").trim().to_string();
                Ok(serde_json::json!({"name": name, "installed": true, "version": version}))
            }
            Ok(_) => Ok(serde_json::json!({"name": name, "installed": false, "version": serde_json::Value::Null})),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                Ok(serde_json::json!({"name": name, "installed": false, "version": serde_json::Value::Null}))
            }
            Err(err) => Err(OpsCommandError::Tool(format!(
                "failed to probe tool `{name}`: {err}"
            ))),
        }
    }
}

impl From<DomainArg> for DomainId {
    fn from(value: DomainArg) -> Self {
        match value {
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
                "could not discover repo root (no ops/atlas-dev/registry.toml found)"
                    .to_string(),
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
    Ok(Selectors {
        suite: suite.map(|v| SuiteId::parse(&v)).transpose()?,
        domain: domain.map(Into::into),
        tag: tag.map(|v| Tag::parse(&v)).transpose()?,
        id_glob: id,
        include_internal,
        include_slow,
    })
}

fn write_output_if_requested(out: Option<PathBuf>, rendered: &str) -> Result<(), String> {
    if let Some(path) = out {
        std::fs::write(&path, format!("{rendered}\n"))
            .map_err(|err| format!("cannot write {}: {err}", path.display()))?;
    }
    Ok(())
}

fn render_list_output(checks_text: String, format: FormatArg) -> Result<String, String> {
    match format {
        FormatArg::Text => Ok(checks_text),
        FormatArg::Json => {
            let rows: Vec<serde_json::Value> = checks_text
                .lines()
                .filter_map(|line| {
                    let (id, title) = line.split_once('\t')?;
                    Some(serde_json::json!({"id": id, "title": title}))
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
                    map.insert(key.to_string(), serde_json::Value::String(value.to_string()));
                }
            }
            serde_json::to_string_pretty(&serde_json::Value::Object(map)).map_err(|err| err.to_string())
        }
        FormatArg::Jsonl => Err("jsonl output is not supported for explain".to_string()),
    }
}

fn render_doctor_output(
    report: &bijux_dev_atlas_core::RegistryDoctorReport,
    format: FormatArg,
) -> Result<String, String> {
    match format {
        FormatArg::Text => {
            if report.errors.is_empty() {
                Ok(String::new())
            } else {
                Ok(report.errors.join("\n"))
            }
        }
        FormatArg::Json => serde_json::to_string_pretty(&serde_json::json!({
            "status": if report.errors.is_empty() { "ok" } else { "failed" },
            "errors": report.errors,
        }))
        .map_err(|err| err.to_string()),
        FormatArg::Jsonl => Err("jsonl output is not supported for doctor".to_string()),
    }
}

const REQUIRED_OPS_TOOLS: &[&str] = &["kind", "kubectl", "helm", "curl"];
const OPTIONAL_OPS_TOOLS: &[&str] = &["k6", "kubeconform"];

fn resolve_ops_root(repo_root: &Path, ops_root: Option<PathBuf>) -> Result<PathBuf, OpsCommandError> {
    let path = ops_root.unwrap_or_else(|| repo_root.join("ops"));
    path.canonicalize()
        .map_err(|err| OpsCommandError::Manifest(format!("cannot resolve ops root {}: {err}", path.display())))
}

fn load_profiles(ops_root: &Path) -> Result<Vec<StackProfile>, OpsCommandError> {
    let path = ops_root.join("stack/profiles.json");
    let text =
        std::fs::read_to_string(&path).map_err(|err| OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display())))?;
    let payload: StackProfiles =
        serde_json::from_str(&text).map_err(|err| OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display())))?;
    Ok(payload.profiles)
}

fn resolve_profile(requested: Option<String>, profiles: &[StackProfile]) -> Result<StackProfile, OpsCommandError> {
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

fn emit_payload(format: FormatArg, out: Option<PathBuf>, payload: &serde_json::Value) -> Result<String, String> {
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

fn run_ops_checks(common: &OpsCommonArgs, suite: &str, include_internal: bool, include_slow: bool) -> Result<(String, i32), String> {
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

fn run_ops_command(quiet: bool, debug: bool, command: OpsCommand) -> i32 {
    let run: Result<(String, i32), String> = (|| match command {
        OpsCommand::Doctor(common) => run_ops_checks(&common, "ops_fast", false, false),
        OpsCommand::Validate(common) => run_ops_checks(&common, "ops_all", true, true),
        OpsCommand::Render(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            if !common.allow_subprocess {
                return Err(OpsCommandError::Effect(
                    "render requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            let ops_root =
                resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
            let fs_adapter = OpsFs::new(repo_root.clone(), ops_root.clone());
            let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let profile = resolve_profile(common.profile.clone(), &profiles)
                .map_err(|e| e.to_stable_message())?;
            let run_id = run_id_or_default(common.run_id.clone())?;
            let payload = serde_json::json!({
                "repo_root": repo_root.display().to_string(),
                "ops_root": ops_root.display().to_string(),
                "profile": profile.name,
                "kind_profile": profile.kind_profile,
                "cluster_config": profile.cluster_config,
                "run_id": run_id.as_str(),
            });
            let render_path = fs_adapter
                .write_artifact_json(&run_id, "render/render.summary.json", &payload)
                .map_err(|e| e.to_stable_message())?;
            let text = format!("rendered ops profile `{}` to {}", payload["profile"].as_str().unwrap_or(""), render_path.display());
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": [payload], "summary": {"total": 1, "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::Install(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            if !common.allow_subprocess {
                return Err(OpsCommandError::Effect(
                    "install requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            let ops_root =
                resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
            let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let profile = resolve_profile(common.profile.clone(), &profiles)
                .map_err(|e| e.to_stable_message())?;
            let run_id = run_id_or_default(common.run_id.clone())?;
            let payload = serde_json::json!({
                "mode": "validate-only",
                "profile": profile.name,
                "run_id": run_id.as_str(),
                "next_steps": [
                    "run `bijux dev atlas ops render --profile <name>`",
                    "run `bijux dev atlas ops status --profile <name>`",
                ],
            });
            let text = format!("install is validate-only for profile `{}`; see next_steps", payload["profile"].as_str().unwrap_or(""));
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": [payload], "summary": {"total": 1, "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::Status(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root =
                resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
            let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let profile = resolve_profile(common.profile.clone(), &profiles)
                .map_err(|e| e.to_stable_message())?;
            let toolchain_path = ops_root.join("inventory/toolchain.json");
            let toolchain = std::fs::read_to_string(&toolchain_path)
                .map_err(|err| OpsCommandError::Manifest(format!("failed to read {}: {err}", toolchain_path.display())).to_stable_message())?;
            let toolchain_json: serde_json::Value =
                serde_json::from_str(&toolchain).map_err(|err| OpsCommandError::Schema(format!("failed to parse {}: {err}", toolchain_path.display())).to_stable_message())?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "repo_root": repo_root.display().to_string(),
                "ops_root": ops_root.display().to_string(),
                "profile": profile,
                "toolchain": toolchain_json,
            });
            let text = format!(
                "ops status: profile={} repo_root={} ops_root={}",
                payload["profile"]["name"].as_str().unwrap_or(""),
                payload["repo_root"].as_str().unwrap_or(""),
                payload["ops_root"].as_str().unwrap_or(""),
            );
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": [payload], "summary": {"total": 1, "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::ListProfiles(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root =
                resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
            let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let rows = profiles
                .iter()
                .map(|p| serde_json::json!({"name": p.name, "kind_profile": p.kind_profile, "cluster_config": p.cluster_config}))
                .collect::<Vec<_>>();
            let text = profiles.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join("\n");
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": rows, "summary": {"total": profiles.len(), "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::ExplainProfile { name, common } => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root =
                resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
            let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let profile = resolve_profile(Some(name), &profiles).map_err(|e| e.to_stable_message())?;
            let text = format!(
                "profile={} kind_profile={} cluster_config={}",
                profile.name, profile.kind_profile, profile.cluster_config
            );
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": [profile], "summary": {"total": 1, "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::ListTools(common) => {
            let process = OpsProcess::new(common.allow_subprocess);
            let mut rows = Vec::new();
            for name in REQUIRED_OPS_TOOLS {
                let mut row = process
                    .probe_tool(name)
                    .map_err(|e| e.to_stable_message())?;
                row["required"] = serde_json::Value::Bool(true);
                rows.push(row);
            }
            for name in OPTIONAL_OPS_TOOLS {
                let mut row = process
                    .probe_tool(name)
                    .map_err(|e| e.to_stable_message())?;
                row["required"] = serde_json::Value::Bool(false);
                rows.push(row);
            }
            rows.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
            let text = rows
                .iter()
                .map(|r| format!("{} required={} installed={}", r["name"].as_str().unwrap_or(""), r["required"], r["installed"]))
                .collect::<Vec<_>>()
                .join("\n");
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": rows, "summary": {"total": rows.len(), "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::VerifyTools(common) => {
            let process = OpsProcess::new(common.allow_subprocess);
            let mut rows = Vec::new();
            let mut missing = Vec::new();
            let mut warnings = Vec::new();
            for name in REQUIRED_OPS_TOOLS {
                let row = process
                    .probe_tool(name)
                    .map_err(|e| e.to_stable_message())?;
                if row["installed"] == serde_json::Value::Bool(false) {
                    missing.push((*name).to_string());
                }
                rows.push(row);
            }
            for name in OPTIONAL_OPS_TOOLS {
                let row = process
                    .probe_tool(name)
                    .map_err(|e| e.to_stable_message())?;
                if row["installed"] == serde_json::Value::Bool(false) {
                    warnings.push((*name).to_string());
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
            let ops_root =
                resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
            let fs_adapter = OpsFs::new(repo_root, ops_root);
            let mut payload: SurfacesInventory = fs_adapter
                .read_ops_json("inventory/surfaces.json")
                .map_err(|e| e.to_stable_message())?;
            payload.actions.sort_by(|a, b| a.id.cmp(&b.id));
            let rows = payload.actions.iter()
                .map(|a| serde_json::json!({"id": a.id, "domain": a.domain, "command": a.command, "argv": a.argv}))
                .collect::<Vec<_>>();
            let text = payload.actions.iter().map(|a| a.id.clone()).collect::<Vec<_>>().join("\n");
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": rows, "summary": {"total": payload.actions.len(), "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::Up(common) => {
            let text = "ops up not yet implemented in rust control-plane; use render/status while migration completes".to_string();
            if !common.allow_subprocess {
                return Err(OpsCommandError::Effect(
                    "up requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            let rendered = emit_payload(common.format, common.out.clone(), &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 0, "errors": 0, "warnings": 0}}))?;
            Ok((rendered, 0))
        }
        OpsCommand::Down(common) => {
            let text = "ops down not yet implemented in rust control-plane; no resources were changed".to_string();
            if !common.allow_subprocess {
                return Err(OpsCommandError::Effect(
                    "down requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            let rendered = emit_payload(common.format, common.out.clone(), &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 0, "errors": 0, "warnings": 0}}))?;
            Ok((rendered, 0))
        }
        OpsCommand::Clean(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let path = repo_root.join("artifacts/atlas-dev/ops");
            if path.exists() {
                std::fs::remove_dir_all(&path).map_err(|err| format!("failed to remove {}: {err}", path.display()))?;
            }
            let text = format!("cleaned {}", path.display());
            let rendered = emit_payload(common.format, common.out.clone(), &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 0, "errors": 0, "warnings": 0}}))?;
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
                let rendered = emit_payload(common.format, common.out.clone(), &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 1, "errors": if ok {0} else {1}, "warnings": 0}}))?;
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
                    let text = "ops pins update is migration-gated; no mutation performed".to_string();
                    let rendered = emit_payload(common.format, common.out.clone(), &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 1, "errors": 0, "warnings": 0}}))?;
                    Ok((rendered, 0))
                }
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
    let exit = match cli.command {
        Command::List {
            repo_root,
            suite,
            domain,
            tag,
            id,
            include_internal,
            include_slow,
            format,
            out,
        } => {
            match resolve_repo_root(repo_root).and_then(|root| {
                let selectors =
                    parse_selectors(suite, domain, tag, id, include_internal, include_slow)?;
                let registry = load_registry(&root)?;
                let checks = select_checks(&registry, &selectors)?;
                let rendered = render_list_output(list_output(&checks), format)?;
                write_output_if_requested(out, &rendered)?;
                Ok(rendered)
            }) {
                Ok(text) => {
                    if !cli.quiet && !text.is_empty() {
                        println!("{text}");
                    }
                    0
                }
                Err(err) => {
                    eprintln!("bijux-dev-atlas list failed: {err}");
                    1
                }
            }
        }
        Command::Explain {
            check_id,
            repo_root,
            format,
            out,
        } => match resolve_repo_root(repo_root).and_then(|root| {
            let registry = load_registry(&root)?;
            let id = CheckId::parse(&check_id)?;
            let rendered = render_explain_output(explain_output(&registry, &id)?, format)?;
            write_output_if_requested(out, &rendered)?;
            Ok(rendered)
        }) {
            Ok(text) => {
                if !cli.quiet && !text.is_empty() {
                    println!("{text}");
                }
                0
            }
            Err(err) => {
                eprintln!("bijux-dev-atlas explain failed: {err}");
                1
            }
        },
        Command::Doctor {
            repo_root,
            format,
            out,
        } => match resolve_repo_root(repo_root) {
            Ok(root) => {
                let report = registry_doctor(&root);
                match render_doctor_output(&report, format).and_then(|rendered| {
                    write_output_if_requested(out, &rendered)?;
                    Ok(rendered)
                }) {
                    Ok(rendered) => {
                        if !cli.quiet && !rendered.is_empty() {
                            if report.errors.is_empty() {
                                println!("{rendered}");
                            } else {
                                eprintln!("{rendered}");
                            }
                        }
                        if report.errors.is_empty() { 0 } else { 1 }
                    }
                    Err(err) => {
                        eprintln!("bijux-dev-atlas doctor failed: {err}");
                        1
                    }
                }
            }
            Err(err) => {
                eprintln!("bijux-dev-atlas doctor failed: {err}");
                1
            }
        },
        Command::Run {
            repo_root,
            artifacts_root,
            run_id,
            suite,
            domain,
            tag,
            id,
            include_internal,
            include_slow,
            allow_subprocess,
            allow_git,
            allow_write,
            allow_network,
            fail_fast,
            max_failures,
            format,
            out,
            durations,
        } => {
            let result = resolve_repo_root(repo_root).and_then(|root| {
                let selectors =
                    parse_selectors(suite, domain, tag, id, include_internal, include_slow)?;
                let request = RunRequest {
                    repo_root: root.clone(),
                    domain: selectors.domain,
                    capabilities: Capabilities::from_cli_flags(
                        allow_write,
                        allow_subprocess,
                        allow_git,
                        allow_network,
                    ),
                    artifacts_root: artifacts_root.or_else(|| Some(root.join("artifacts"))),
                    run_id: run_id.map(|rid| RunId::parse(&rid)).transpose()?,
                };
                let options = RunOptions {
                    fail_fast,
                    max_failures,
                };
                let report =
                    run_checks(&RealProcessRunner, &RealFs, &request, &selectors, &options)?;
                let rendered = match format {
                    FormatArg::Text => render_text_with_durations(&report, durations),
                    FormatArg::Json => render_json(&report)?,
                    FormatArg::Jsonl => render_jsonl(&report)?,
                };
                write_output_if_requested(out, &rendered)?;
                Ok((rendered, exit_code_for_report(&report)))
            });

            match result {
                Ok((rendered, code)) => {
                    if !cli.quiet {
                        println!("{rendered}");
                    }
                    code
                }
                Err(err) => {
                    eprintln!("bijux-dev-atlas run failed: {err}");
                    1
                }
            }
        }
        Command::Ops { command } => run_ops_command(cli.quiet, cli.debug, command),
    };

    if cli.verbose {
        eprintln!("bijux-dev-atlas exit={exit}");
    }
    std::process::exit(exit);
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
            vec!["bijux-dev-atlas", "ops", "install", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "ops", "status"],
            vec!["bijux-dev-atlas", "ops", "list-profiles"],
            vec!["bijux-dev-atlas", "ops", "explain-profile", "kind"],
            vec!["bijux-dev-atlas", "ops", "list-tools", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "ops", "verify-tools", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "ops", "list-actions"],
            vec!["bijux-dev-atlas", "ops", "up", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "ops", "down", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "ops", "clean"],
            vec!["bijux-dev-atlas", "ops", "pins", "check"],
            vec![
                "bijux-dev-atlas",
                "ops",
                "pins",
                "update",
                "--allow-subprocess",
                "--i-know-what-im-doing",
            ],
        ];
        for argv in commands {
            let cli = super::Cli::try_parse_from(argv).expect("parse");
            match cli.command {
                super::Command::Ops { .. } => {}
                _ => panic!("expected ops command"),
            }
        }
    }
}
