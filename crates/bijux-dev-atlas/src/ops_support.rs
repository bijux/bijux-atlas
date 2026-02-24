// SPDX-License-Identifier: Apache-2.0

use crate::*;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct StackProfiles {
    pub(crate) profiles: Vec<StackProfile>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct StackManifestToml {
    pub(crate) profiles: BTreeMap<String, StackManifestProfile>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct StackManifestProfile {
    pub(crate) kind_profile: String,
    pub(crate) cluster_config: String,
    pub(crate) namespace: String,
    pub(crate) components: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ToolchainInventory {
    pub(crate) tools: BTreeMap<String, ToolDefinition>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ToolDefinition {
    pub(crate) required: bool,
    pub(crate) version_regex: String,
    pub(crate) probe_argv: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct StackProfile {
    pub(crate) name: String,
    pub(crate) kind_profile: String,
    pub(crate) cluster_config: String,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct SurfacesInventory {
    pub(crate) actions: Vec<SurfaceAction>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct SurfaceAction {
    pub(crate) id: String,
    pub(crate) domain: String,
    pub(crate) command: Vec<String>,
    pub(crate) argv: Vec<String>,
}

#[derive(Debug)]
pub(crate) enum OpsCommandError {
    Manifest(String),
    Schema(String),
    Tool(String),
    Profile(String),
    Effect(String),
}

impl OpsCommandError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::Manifest(_) => "OPS_MANIFEST_ERROR",
            Self::Schema(_) => "OPS_SCHEMA_ERROR",
            Self::Tool(_) => "OPS_TOOL_ERROR",
            Self::Profile(_) => "OPS_PROFILE_ERROR",
            Self::Effect(_) => "OPS_EFFECT_ERROR",
        }
    }

    pub(crate) fn to_stable_message(&self) -> String {
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

pub(crate) struct OpsFs {
    repo_root: PathBuf,
    ops_root: PathBuf,
}

impl OpsFs {
    pub(crate) fn new(repo_root: PathBuf, ops_root: PathBuf) -> Self {
        Self {
            repo_root,
            ops_root,
        }
    }

    pub(crate) fn read_ops_json<T: for<'de> Deserialize<'de>>(
        &self,
        rel: &str,
    ) -> Result<T, OpsCommandError> {
        let path = self.ops_root.join(rel);
        let text = std::fs::read_to_string(&path).map_err(|err| {
            OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display()))
        })?;
        serde_json::from_str(&text).map_err(|err| {
            OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display()))
        })
    }

    pub(crate) fn write_artifact_json(
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

pub(crate) struct OpsProcess {
    allow_subprocess: bool,
}

impl OpsProcess {
    pub(crate) fn new(allow_subprocess: bool) -> Self {
        Self { allow_subprocess }
    }

    pub(crate) fn probe_tool(
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
                Ok(
                    serde_json::json!({"name": name, "installed": true, "version_raw": raw, "version": version, "version_regex": version_regex}),
                )
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

    pub(crate) fn run_subprocess(
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
            "HOME", "PATH", "TMPDIR", "TEMP", "TMP", "USER", "LOGNAME", "SHELL",
        ] {
            if let Ok(value) = std::env::var(key) {
                cmd.env(key, value);
            }
        }
        let output = cmd.output().map_err(|err| {
            OpsCommandError::Tool(format!(
                "failed to execute `{}` with {} args: {err}",
                binary,
                args.len()
            ))
        })?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let event = serde_json::json!({
            "binary": binary,
            "args": redact_argv(args),
            "cwd": cwd.display().to_string(),
            "status": output.status.code(),
            "stdout_bytes": output.stdout.len(),
            "stderr_bytes": output.stderr.len(),
        });
        if output.status.success() {
            Ok((stdout, event))
        } else {
            Err(OpsCommandError::Tool(format!(
                "subprocess failed `{binary}` exit={:?} stderr={}",
                output.status.code(),
                stderr.lines().next().unwrap_or_default()
            )))
        }
    }
}

fn redact_argv(args: &[String]) -> Vec<String> {
    let sensitive_prefixes = ["--set", "--set-string", "--password", "--token"];
    let mut out = Vec::with_capacity(args.len());
    let mut redact_next = false;
    for arg in args {
        if redact_next {
            out.push("<redacted>".to_string());
            redact_next = false;
            continue;
        }
        if sensitive_prefixes.iter().any(|p| arg == p) {
            out.push(arg.clone());
            redact_next = true;
            continue;
        }
        if sensitive_prefixes
            .iter()
            .any(|p| arg.starts_with(&format!("{p}=")))
        {
            out.push(format!(
                "{}=<redacted>",
                arg.split_once('=').map(|(k, _)| k).unwrap_or(arg)
            ));
            continue;
        }
        out.push(arg.clone());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::redact_argv;

    #[test]
    fn redact_argv_masks_sensitive_values() {
        let args = vec![
            "--set".to_string(),
            "secret=abc".to_string(),
            "--token=xyz".to_string(),
            "--namespace".to_string(),
            "bijux-atlas".to_string(),
        ];
        let redacted = redact_argv(&args);
        assert_eq!(redacted[0], "--set");
        assert_eq!(redacted[1], "<redacted>");
        assert_eq!(redacted[2], "--token=<redacted>");
        assert_eq!(redacted[4], "bijux-atlas");
    }
}
