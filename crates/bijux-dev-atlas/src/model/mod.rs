// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
//! `model` defines serde-facing report and identifier types shared across the crate.
//!
//! Boundary: model is a leaf module; it must not depend on `core`, `commands`, or `adapters`.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const CONTRACT_SCHEMA_VERSION: u64 = 1;
pub const fn schema_version() -> u64 {
    CONTRACT_SCHEMA_VERSION
}

fn is_lower_snake(input: &str) -> bool {
    !input.is_empty()
        && input
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn is_canonical_check_id(raw: &str) -> bool {
    let mut parts = raw.split('_');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some("checks"), Some(_domain), Some(_area), Some(_name)) => parts.all(|p| !p.is_empty()),
        _ => false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CheckId(String);

impl CheckId {
    pub fn parse(value: &str) -> Result<Self, String> {
        let raw = value.trim();
        if raw.is_empty() {
            return Err("check id cannot be empty".to_string());
        }
        if !is_lower_snake(raw) {
            return Err(format!(
                "invalid check id `{raw}`: expected lowercase snake_case"
            ));
        }
        if !is_canonical_check_id(raw) {
            return Err(format!(
                "invalid check id `{raw}`: expected checks_<domain>_<area>_<name>"
            ));
        }
        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ViolationId(String);

impl ViolationId {
    pub fn parse(value: &str) -> Result<Self, String> {
        let raw = value.trim();
        if raw.is_empty() {
            return Err("violation id cannot be empty".to_string());
        }
        if !is_lower_snake(raw) {
            return Err(format!(
                "invalid violation id `{raw}`: expected lowercase snake_case"
            ));
        }
        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ViolationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArtifactPath(String);

impl ArtifactPath {
    pub fn parse(value: &str) -> Result<Self, String> {
        let raw = value.trim();
        if raw.is_empty() {
            return Err("artifact path cannot be empty".to_string());
        }
        if raw.contains('\n') || raw.contains('\r') {
            return Err("artifact path must be single-line".to_string());
        }
        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ArtifactPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for CheckId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainId {
    Root,
    Workflows,
    Configs,
    Docker,
    Crates,
    Ops,
    Repo,
    Docs,
    Make,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Tag(String);

impl Tag {
    pub const MAX_LEN: usize = 48;

    pub fn parse(value: &str) -> Result<Self, String> {
        let raw = value.trim();
        if raw.is_empty() {
            return Err("tag cannot be empty".to_string());
        }
        if raw.len() > Self::MAX_LEN {
            return Err(format!("tag `{raw}` exceeds max length {}", Self::MAX_LEN));
        }
        if !is_lower_snake(raw) {
            return Err(format!(
                "invalid tag `{raw}`: expected lowercase snake_case"
            ));
        }
        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SuiteId(String);

impl SuiteId {
    pub fn parse(value: &str) -> Result<Self, String> {
        let raw = value.trim();
        if raw.is_empty() {
            return Err("suite id cannot be empty".to_string());
        }
        if !is_lower_snake(raw) {
            return Err(format!(
                "invalid suite id `{raw}`: expected lowercase snake_case"
            ));
        }
        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SuiteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    FsRead,
    FsWrite,
    Subprocess,
    Git,
    Network,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Violation {
    #[serde(default = "schema_version")]
    pub schema_version: u64,
    pub code: ViolationId,
    pub message: String,
    pub hint: Option<String>,
    pub path: Option<ArtifactPath>,
    pub line: Option<u32>,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pass,
    Fail,
    Skip,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    PolicyFail,
    UsageError,
    IoError,
    SchemaError,
    ToolError,
    EffectDenied,
    CheckExecutionError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRef {
    #[serde(default = "schema_version")]
    pub schema_version: u64,
    pub kind: String,
    pub path: ArtifactPath,
    pub content_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Public,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckSpec {
    pub id: CheckId,
    pub domain: DomainId,
    pub title: String,
    pub docs: String,
    pub tags: Vec<Tag>,
    pub suites: Vec<SuiteId>,
    pub effects_required: Vec<Effect>,
    pub budget_ms: u64,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckResult {
    #[serde(default = "schema_version")]
    pub schema_version: u64,
    pub id: CheckId,
    pub status: CheckStatus,
    pub skip_reason: Option<String>,
    pub violations: Vec<Violation>,
    pub duration_ms: u64,
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RunId(String);

impl RunId {
    pub fn parse(value: &str) -> Result<Self, String> {
        let raw = value.trim();
        if raw.is_empty() {
            return Err("run id cannot be empty".to_string());
        }
        if !is_lower_snake(raw) {
            return Err(format!(
                "invalid run id `{raw}`: expected lowercase snake_case"
            ));
        }
        Ok(Self(raw.to_string()))
    }

    pub fn from_seed(seed: &str) -> Self {
        let mut out = String::with_capacity(seed.len());
        for c in seed.chars() {
            if c.is_ascii_alphanumeric() {
                out.push(c.to_ascii_lowercase());
            } else {
                out.push('_');
            }
        }
        let compact = out
            .split('_')
            .filter(|seg| !seg.is_empty())
            .collect::<Vec<_>>()
            .join("_");
        if compact.is_empty() {
            return Self("run".to_string());
        }
        Self(compact)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn generate(timestamp_unix_secs: u64, git_short_hash: Option<&str>, entropy: u32) -> Self {
        let ts = format!("t{timestamp_unix_secs}");
        let git = git_short_hash
            .unwrap_or("nogit")
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .take(8)
            .collect::<String>()
            .to_ascii_lowercase();
        let rand = format!("r{entropy:08x}");
        Self::from_seed(&format!("{ts}_{git}_{rand}"))
    }
}

impl fmt::Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArtifactsRoot(String);

impl ArtifactsRoot {
    pub fn parse(value: &str) -> Result<Self, String> {
        let raw = value.trim();
        if raw.is_empty() {
            return Err("artifacts root cannot be empty".to_string());
        }
        Ok(Self(raw.to_string()))
    }

    pub fn default_for_repo(repo_root: &Path) -> Self {
        Self(
            repo_root
                .join("artifacts")
                .join("atlas-dev")
                .display()
                .to_string(),
        )
    }

    pub fn as_path(&self) -> &Path {
        Path::new(&self.0)
    }

    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunSummary {
    #[serde(default = "schema_version")]
    pub schema_version: u64,
    pub passed: u64,
    pub failed: u64,
    pub skipped: u64,
    pub errors: u64,
    pub total: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunReport {
    #[serde(default = "schema_version")]
    pub schema_version: u64,
    pub run_id: RunId,
    pub repo_root: String,
    pub command: String,
    pub selections: BTreeMap<String, String>,
    pub capabilities: BTreeMap<String, bool>,
    pub results: Vec<CheckResult>,
    pub durations_ms: BTreeMap<CheckId, u64>,
    pub counts: RunSummary,
    pub summary: RunSummary,
    pub timings_ms: BTreeMap<CheckId, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpsRunReport {
    #[serde(default = "schema_version")]
    pub schema_version: u64,
    pub kind: String,
    pub command: String,
    pub run_id: RunId,
    pub repo_root: String,
    pub ops_root: String,
    pub profile: Option<String>,
    pub suite: Option<String>,
    pub status: String,
    pub exit_code: i32,
    pub checks: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub capabilities: BTreeMap<String, String>,
    pub summary: BTreeMap<String, u64>,
    pub rows: Vec<serde_json::Value>,
}

pub mod fingerprint {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    pub fn violation_fingerprint(violation: &Violation) -> String {
        let mut hasher = DefaultHasher::new();
        violation.code.hash(&mut hasher);
        violation.message.hash(&mut hasher);
        violation.path.hash(&mut hasher);
        violation.line.hash(&mut hasher);
        violation.severity.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    pub fn check_result_fingerprints(result: &CheckResult) -> Vec<String> {
        result
            .violations
            .iter()
            .map(violation_fingerprint)
            .collect::<Vec<_>>()
    }
}

pub mod display {
    use super::*;

    pub fn summary_line(report: &RunReport) -> String {
        format!(
            "run={} passed={} failed={} skipped={} errors={} total={}",
            report.run_id,
            report.summary.passed,
            report.summary.failed,
            report.summary.skipped,
            report.summary.errors,
            report.summary.total
        )
    }

    pub fn check_result_line(result: &CheckResult) -> String {
        format!(
            "check={} status={:?} violations={}",
            result.id,
            result.status,
            result.violations.len()
        )
    }
}

pub fn report_json_schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "bijux-dev-atlas run report",
        "type": "object",
        "required": ["schema_version", "run_id", "repo_root", "command", "selections", "capabilities", "results", "durations_ms", "counts", "summary", "timings_ms"],
        "properties": {
            "schema_version": {"type": "integer", "const": CONTRACT_SCHEMA_VERSION},
            "run_id": {"type": "string"},
            "repo_root": {"type": "string"},
            "command": {"type": "string"},
            "selections": {"type": "object", "additionalProperties": {"type": "string"}},
            "capabilities": {"type": "object", "additionalProperties": {"type": "boolean"}},
            "results": {"type": "array"},
            "durations_ms": {"type": "object", "additionalProperties": {"type": "integer", "minimum": 0}},
            "counts": {"type": "object"},
            "summary": {"type": "object"},
            "timings_ms": {"type": "object", "additionalProperties": {"type": "integer", "minimum": 0}}
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_id_validation() {
        assert!(CheckId::parse("checks_ops_surface_manifest").is_ok());
        assert!(CheckId::parse("ops_surface_manifest").is_err());
        assert!(CheckId::parse("OPS_SURFACE").is_err());
        assert!(CheckId::parse("").is_err());
    }

    #[test]
    fn tag_validation() {
        assert!(Tag::parse("lint").is_ok());
        assert!(Tag::parse("lint-fast").is_err());
        assert!(Tag::parse("Lint").is_err());
    }

    #[test]
    fn suite_validation() {
        assert!(SuiteId::parse("ops_fast").is_ok());
        assert!(SuiteId::parse("ops-fast").is_err());
    }

    #[test]
    fn run_id_validation_and_seed() {
        assert!(RunId::parse("stable_run").is_ok());
        assert!(RunId::parse("stable-run").is_err());
        let seeded = RunId::from_seed("Ops: Daily Run 001");
        assert_eq!(seeded.as_str(), "ops_daily_run_001");
    }

    #[test]
    fn run_id_generate_is_deterministic_for_same_inputs() {
        let one = RunId::generate(1_700_000_000, Some("abc123de"), 0x10);
        let two = RunId::generate(1_700_000_000, Some("abc123de"), 0x10);
        assert_eq!(one, two);
        assert!(one.as_str().contains("abc123de"));
    }

    #[test]
    fn report_schema_contains_required_fields() {
        let schema = report_json_schema();
        let required = schema.get("required");
        assert!(required.is_some());
        let required_text = required.map(Value::to_string).unwrap_or_default();
        assert!(required_text.contains("run_id"));
        assert!(required_text.contains("results"));
        assert!(required_text.contains("command"));
        assert!(required_text.contains("selections"));
        assert!(required_text.contains("capabilities"));
        assert!(required_text.contains("durations_ms"));
        assert!(required_text.contains("counts"));
        assert!(required_text.contains("schema_version"));
    }

    #[test]
    fn typed_ids_validate() {
        assert!(ViolationId::parse("make_contract_missing").is_ok());
        assert!(ViolationId::parse("MAKE_CONTRACT").is_err());
        assert!(ArtifactPath::parse("ops/CONTRACT.md").is_ok());
        assert!(ArtifactPath::parse("").is_err());
    }

    #[test]
    fn violation_fingerprint_is_stable() {
        let violation = Violation {
            schema_version: schema_version(),
            code: ViolationId::parse("ops_contract_missing").expect("id"),
            message: "missing contract".to_string(),
            hint: None,
            path: Some(ArtifactPath::parse("ops/CONTRACT.md").expect("path")),
            line: Some(1),
            severity: Severity::Error,
        };
        let one = fingerprint::violation_fingerprint(&violation);
        let two = fingerprint::violation_fingerprint(&violation);
        assert_eq!(one, two);
    }
}
