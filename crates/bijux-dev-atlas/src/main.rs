#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::{fs, io::Write};

use bijux_dev_atlas_adapters::{Capabilities, RealFs, RealProcessRunner};
use bijux_dev_atlas_core::{
    exit_code_for_report, explain_output, list_output, load_registry, registry_doctor, render_json,
    render_jsonl, render_text_with_durations, run_checks, select_checks, RunOptions, RunRequest, Selectors,
};
use bijux_dev_atlas_core::ops_inventory::{ops_inventory_summary, validate_ops_inventory};
use bijux_dev_atlas_model::{CheckId, DomainId, RunId, SuiteId, Tag};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
    Render(OpsRenderArgs),
    Install(OpsInstallArgs),
    Status(OpsStatusArgs),
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
    Reset(OpsResetArgs),
    Pins {
        #[command(subcommand)]
        command: OpsPinsCommand,
    },
}

#[derive(Args, Debug, Clone)]
struct OpsRenderArgs {
    #[command(flatten)]
    common: OpsCommonArgs,
    #[arg(long, value_enum, default_value_t = OpsRenderTarget::Helm)]
    target: OpsRenderTarget,
    #[arg(long, default_value_t = false)]
    check: bool,
    #[arg(long, default_value_t = false)]
    write: bool,
    #[arg(long, default_value_t = false)]
    stdout: bool,
    #[arg(long, default_value_t = false)]
    diff: bool,
    #[arg(long)]
    helm_binary: Option<String>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OpsRenderTarget {
    Helm,
    Kustomize,
    Kind,
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
    #[arg(long, default_value_t = false)]
    allow_write: bool,
}

#[derive(Args, Debug, Clone)]
struct OpsInstallArgs {
    #[command(flatten)]
    common: OpsCommonArgs,
    #[arg(long, default_value_t = false)]
    kind: bool,
    #[arg(long, default_value_t = false)]
    apply: bool,
    #[arg(long, default_value_t = false)]
    plan: bool,
    #[arg(long, default_value = "none")]
    dry_run: String,
    #[arg(long, default_value_t = false)]
    force: bool,
}

#[derive(Args, Debug, Clone)]
struct OpsStatusArgs {
    #[command(flatten)]
    common: OpsCommonArgs,
    #[arg(long, value_enum, default_value_t = OpsStatusTarget::Local)]
    target: OpsStatusTarget,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OpsStatusTarget {
    Local,
    K8s,
    Pods,
    Endpoints,
}

#[derive(Args, Debug, Clone)]
struct OpsResetArgs {
    #[command(flatten)]
    common: OpsCommonArgs,
    #[arg(long = "reset-run-id")]
    reset_id: String,
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
        for key in ["PATH", "HOME", "KUBECONFIG", "HELM_CACHE_HOME", "HELM_CONFIG_HOME", "HELM_DATA_HOME"] {
            if let Ok(value) = std::env::var(key) {
                cmd.env(key, value);
            }
        }
        let output = cmd.output().map_err(|err| {
            OpsCommandError::Tool(format!("failed to run `{binary}`: {err}"))
        })?;
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
            errors.push(format!("rendered image uses forbidden latest tag: {}", line.trim()));
        }
    }
    for marker in ["generatedAt:", "timestamp:", "creationTimestamp:"] {
        if rendered.contains(marker) {
            errors.push(format!("render output contains forbidden timestamp marker `{marker}`"));
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

fn ensure_kind_context(process: &OpsProcess, profile: &StackProfile, force: bool) -> Result<(), OpsCommandError> {
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

fn ensure_namespace_exists(process: &OpsProcess, namespace: &str, dry_run: &str) -> Result<(), OpsCommandError> {
    let get_args = vec![
        "get".to_string(),
        "namespace".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "name".to_string(),
    ];
    if process.run_subprocess("kubectl", &get_args, Path::new(".")).is_ok() {
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
    let exit = if strict_failed {
        1
    } else if checks_exit != 0 || inventory_error_count > 0 {
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

fn run_ops_command(quiet: bool, debug: bool, command: OpsCommand) -> i32 {
    let run: Result<(String, i32), String> = (|| match command {
        OpsCommand::Doctor(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let inventory_errors = validate_ops_inventory(&repo_root);
            let summary = ops_inventory_summary(&repo_root).unwrap_or_else(|err| {
                serde_json::json!({"error": format!("OPS_MANIFEST_ERROR: {err}")})
            });
            let (checks_rendered, checks_exit) = run_ops_checks(&common, "ops_fast", false, false)?;
            render_ops_validation_output(&common, "doctor", &inventory_errors, &checks_rendered, checks_exit, summary)
        }
        OpsCommand::Validate(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let inventory_errors = validate_ops_inventory(&repo_root);
            let summary = ops_inventory_summary(&repo_root).unwrap_or_else(|err| {
                serde_json::json!({"error": format!("OPS_MANIFEST_ERROR: {err}")})
            });
            let (checks_rendered, checks_exit) = run_ops_checks(&common, "ops_all", true, true)?;
            render_ops_validation_output(&common, "validate", &inventory_errors, &checks_rendered, checks_exit, summary)
        }
        OpsCommand::Render(args) => {
            let common = &args.common;
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root =
                resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
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

            let (rendered_manifest, subprocess_events): (String, Vec<serde_json::Value>) = match args.target {
                OpsRenderTarget::Helm => {
                    if !common.allow_subprocess {
                        return Err(OpsCommandError::Effect(
                            "helm render requires --allow-subprocess".to_string(),
                        )
                        .to_stable_message());
                    }
                    let helm_binary = args.helm_binary.clone().unwrap_or_else(|| "helm".to_string());
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
                    (format!("# source: {}\n{content}", profile.cluster_config), Vec::new())
                }
                OpsRenderTarget::Kustomize => {
                    return Err(
                        OpsCommandError::Effect(
                            "kustomize render is not enabled; use --target helm or --target kind".to_string(),
                        )
                        .to_stable_message(),
                    )
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
                file.write_all(rendered_manifest.as_bytes()).map_err(|err| {
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
                        .strip_prefix(repo_root.join("artifacts/atlas-dev/ops").join(run_id.as_str()))
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
            let ops_root =
                resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
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
            let ops_root =
                resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
            let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let profile = resolve_profile(common.profile.clone(), &profiles)
                .map_err(|e| e.to_stable_message())?;
            let process = OpsProcess::new(common.allow_subprocess);
            let (payload, text) = match args.target {
                OpsStatusTarget::Local => {
                    let toolchain_path = ops_root.join("inventory/toolchain.json");
                    let toolchain = std::fs::read_to_string(&toolchain_path)
                        .map_err(|err| OpsCommandError::Manifest(format!("failed to read {}: {err}", toolchain_path.display())).to_stable_message())?;
                    let toolchain_json: serde_json::Value =
                        serde_json::from_str(&toolchain).map_err(|err| OpsCommandError::Schema(format!("failed to parse {}: {err}", toolchain_path.display())).to_stable_message())?;
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
                    let value: serde_json::Value = serde_json::from_str(&stdout)
                        .map_err(|err| OpsCommandError::Schema(format!("failed to parse kubectl json: {err}")).to_stable_message())?;
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
                    let value: serde_json::Value = serde_json::from_str(&stdout)
                        .map_err(|err| OpsCommandError::Schema(format!("failed to parse kubectl json: {err}")).to_stable_message())?;
                    let mut pods = value
                        .get("items")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    pods.sort_by(|a, b| {
                        a.get("metadata")
                            .and_then(|m| m.get("name"))
                            .and_then(|v| v.as_str())
                            .cmp(&b.get("metadata")
                                .and_then(|m| m.get("name"))
                                .and_then(|v| v.as_str()))
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
                    let value: serde_json::Value = serde_json::from_str(&stdout)
                        .map_err(|err| OpsCommandError::Schema(format!("failed to parse kubectl json: {err}")).to_stable_message())?;
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
            if !common.allow_subprocess {
                return Err(OpsCommandError::Effect(
                    "up requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            if !common.allow_write {
                return Err(OpsCommandError::Effect(
                    "up requires --allow-write".to_string(),
                )
                .to_stable_message());
            }
            let text = "ops up delegates to install --kind --apply --plan".to_string();
            let rendered = emit_payload(common.format, common.out.clone(), &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 0, "errors": 0, "warnings": 0}}))?;
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
            let ops_root =
                resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
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
            let text = format!("reset artifacts for run_id={} at {}", run_id.as_str(), target.display());
            let rendered = emit_payload(common.format, common.out.clone(), &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 1, "errors": 0, "warnings": 0}}))?;
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
            vec!["bijux-dev-atlas", "ops", "install", "--plan"],
            vec!["bijux-dev-atlas", "ops", "status"],
            vec!["bijux-dev-atlas", "ops", "status", "--target", "k8s", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "ops", "list-profiles"],
            vec!["bijux-dev-atlas", "ops", "explain-profile", "kind"],
            vec!["bijux-dev-atlas", "ops", "list-tools", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "ops", "verify-tools", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "ops", "list-actions"],
            vec!["bijux-dev-atlas", "ops", "up", "--allow-subprocess", "--allow-write"],
            vec!["bijux-dev-atlas", "ops", "down", "--allow-subprocess"],
            vec!["bijux-dev-atlas", "ops", "clean"],
            vec!["bijux-dev-atlas", "ops", "reset", "--reset-run-id", "ops_reset"],
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
