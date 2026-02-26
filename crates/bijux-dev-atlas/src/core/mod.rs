// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
//! `core` contains the pure control-plane engine, registries, and checks.
//!
//! Boundary: core may depend on `model`, `policies`, and `ports`; direct host effects belong in
//! `adapters` implementations.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use bijux_dev_atlas_model::{
    ArtifactsRoot, CheckId, CheckResult, CheckSpec, CheckStatus, DomainId, Effect, RunId,
    RunReport, RunSummary, Severity, SuiteId, Tag, Violation, Visibility,
};
use serde::Deserialize;
use std::borrow::Cow;

mod check_runner;
pub mod checks;
pub mod logging;
pub mod ops_inventory;
pub mod ports;
mod report_rendering;
pub use ports::{Capabilities, Fs, ProcessRunner};

pub const DEFAULT_REGISTRY_PATH: &str = "ops/inventory/registry.toml";

pub fn load_dev_policy_set(
    repo_root: &Path,
) -> Result<bijux_dev_atlas_policies::DevAtlasPolicySet, Cow<'static, str>> {
    bijux_dev_atlas_policies::DevAtlasPolicySet::load(repo_root)
        .map_err(|err| Cow::Owned(err.to_string()))
}

#[derive(Debug, Clone)]
pub struct RunRequest {
    pub repo_root: PathBuf,
    pub domain: Option<DomainId>,
    pub capabilities: Capabilities,
    pub artifacts_root: Option<PathBuf>,
    pub run_id: Option<RunId>,
    pub command: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Selectors {
    pub id_glob: Option<String>,
    pub domain: Option<DomainId>,
    pub tag: Option<Tag>,
    pub suite: Option<SuiteId>,
    pub include_internal: bool,
    pub include_slow: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub fail_fast: bool,
    pub max_failures: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct RegistryDoctorReport {
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Registry {
    pub checks: Vec<CheckSpec>,
    pub suites: Vec<SuiteSpec>,
    pub tags_vocabulary: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct SuiteSpec {
    pub id: SuiteId,
    pub checks: Vec<CheckId>,
    pub domains: Vec<DomainId>,
    pub tags_any: Vec<Tag>,
}

#[derive(Debug)]
pub enum CheckError {
    Failed(String),
}

pub type CheckFn = fn(&CheckContext<'_>) -> Result<Vec<Violation>, CheckError>;

pub trait EffectsBoundary {
    fn filesystem(&self) -> &dyn Fs;
    fn process_runner(&self) -> &dyn ProcessRunner;
}

pub struct AdapterSet<'a> {
    pub fs: &'a dyn Fs,
    pub process: &'a dyn ProcessRunner,
}

impl EffectsBoundary for AdapterSet<'_> {
    fn filesystem(&self) -> &dyn Fs {
        self.fs
    }

    fn process_runner(&self) -> &dyn ProcessRunner {
        self.process
    }
}

pub struct CheckContext<'a> {
    pub repo_root: &'a Path,
    pub artifacts_root: PathBuf,
    pub run_id: RunId,
    pub adapters: AdapterSet<'a>,
    pub registry: &'a Registry,
}

#[derive(Debug, Clone)]
pub struct RuntimeContext {
    pub repo_root: PathBuf,
    pub artifacts_root: ArtifactsRoot,
    pub run_id: RunId,
    pub capabilities: Capabilities,
}

impl RuntimeContext {
    pub fn from_run_request(request: &RunRequest) -> Result<Self, String> {
        let artifacts_root = match &request.artifacts_root {
            Some(path) => ArtifactsRoot::parse(&path.display().to_string())?,
            None => ArtifactsRoot::default_for_repo(&request.repo_root),
        };
        let run_id = request
            .run_id
            .clone()
            .unwrap_or_else(|| RunId::from_seed("registry_run"));
        Ok(Self {
            repo_root: request.repo_root.clone(),
            artifacts_root,
            run_id,
            capabilities: request.capabilities,
        })
    }

    pub fn check_artifacts_run_root(&self) -> PathBuf {
        self.artifacts_root.to_path_buf().join(self.run_id.as_str())
    }
}

#[derive(Debug, Deserialize)]
struct RawRegistry {
    checks: Vec<RawCheck>,
    suites: Vec<RawSuite>,
    tags: Option<RawTags>,
}

#[derive(Debug, Deserialize)]
struct RawTags {
    vocabulary: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawCheck {
    id: String,
    domain: String,
    title: String,
    docs: String,
    tags: Vec<String>,
    suites: Vec<String>,
    effects_required: Option<Vec<String>>,
    budget_ms: Option<u64>,
    visibility: String,
}

#[derive(Debug, Deserialize)]
struct RawSuite {
    id: String,
    checks: Vec<String>,
    domains: Vec<String>,
    tags_any: Vec<String>,
}

fn parse_domain(raw: &str) -> Result<DomainId, String> {
    match raw.trim() {
        "root" => Ok(DomainId::Root),
        "workflows" => Ok(DomainId::Workflows),
        "configs" => Ok(DomainId::Configs),
        "docker" => Ok(DomainId::Docker),
        "crates" => Ok(DomainId::Crates),
        "ops" => Ok(DomainId::Ops),
        "repo" => Ok(DomainId::Repo),
        "docs" => Ok(DomainId::Docs),
        "make" => Ok(DomainId::Make),
        other => Err(format!("invalid domain `{other}`")),
    }
}

fn parse_effect(raw: &str) -> Result<Effect, String> {
    match raw.trim() {
        "fs_read" => Ok(Effect::FsRead),
        "fs_write" => Ok(Effect::FsWrite),
        "subprocess" => Ok(Effect::Subprocess),
        "git" => Ok(Effect::Git),
        "network" => Ok(Effect::Network),
        other => Err(format!("invalid effect `{other}`")),
    }
}

fn parse_visibility(raw: &str) -> Result<Visibility, String> {
    match raw.trim() {
        "public" => Ok(Visibility::Public),
        "internal" => Ok(Visibility::Internal),
        other => Err(format!("invalid visibility `{other}`")),
    }
}

pub fn load_registry(repo_root: &Path) -> Result<Registry, String> {
    let path = repo_root.join(DEFAULT_REGISTRY_PATH);
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let raw: RawRegistry = toml::from_str(&text)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;

    let checks = raw
        .checks
        .into_iter()
        .map(|row| {
            let effects = row
                .effects_required
                .ok_or_else(|| format!("{}: missing effects_required", row.id))?;
            Ok(CheckSpec {
                id: CheckId::parse(&row.id)?,
                domain: parse_domain(&row.domain)?,
                title: row.title.trim().to_string(),
                docs: row.docs.trim().to_string(),
                tags: row
                    .tags
                    .iter()
                    .map(|v| Tag::parse(v))
                    .collect::<Result<Vec<_>, _>>()?,
                suites: row
                    .suites
                    .iter()
                    .map(|v| SuiteId::parse(v))
                    .collect::<Result<Vec<_>, _>>()?,
                effects_required: effects
                    .iter()
                    .map(|v| parse_effect(v))
                    .collect::<Result<Vec<_>, _>>()?,
                budget_ms: row
                    .budget_ms
                    .ok_or_else(|| format!("{}: missing budget_ms", row.id))?,
                visibility: parse_visibility(&row.visibility)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let suites = raw
        .suites
        .into_iter()
        .map(|row| {
            Ok(SuiteSpec {
                id: SuiteId::parse(&row.id)?,
                checks: row
                    .checks
                    .iter()
                    .map(|v| CheckId::parse(v))
                    .collect::<Result<Vec<_>, _>>()?,
                domains: row
                    .domains
                    .iter()
                    .map(|v| parse_domain(v))
                    .collect::<Result<Vec<_>, _>>()?,
                tags_any: row
                    .tags_any
                    .iter()
                    .map(|v| Tag::parse(v))
                    .collect::<Result<Vec<_>, _>>()?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let tags_vocabulary = raw
        .tags
        .map(|t| {
            t.vocabulary
                .iter()
                .map(|v| Tag::parse(v).map(|tv| tv.as_str().to_string()))
                .collect::<Result<BTreeSet<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();

    let registry = Registry {
        checks,
        suites,
        tags_vocabulary,
    };
    let errors = validate_registry(&registry);
    if errors.is_empty() {
        Ok(registry)
    } else {
        Err(errors.join("; "))
    }
}

pub fn validate_registry(registry: &Registry) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    let mut seen = BTreeSet::new();
    let mut purposes = BTreeMap::<String, String>::new();
    let known_ids: BTreeSet<String> = registry
        .checks
        .iter()
        .map(|c| c.id.as_str().to_string())
        .collect();

    for check in &registry.checks {
        if !seen.insert(check.id.as_str().to_string()) {
            errors.push(format!("duplicate check id `{}`", check.id));
        }
        let purpose_key = check
            .title
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_ascii_lowercase();
        if let Some(existing) = purposes.insert(purpose_key.clone(), check.id.as_str().to_string())
        {
            errors.push(format!(
                "duplicate check purpose `{}` for ids `{}` and `{}`",
                purpose_key, existing, check.id
            ));
        }
        if check.budget_ms == 0 {
            errors.push(format!("{}: budget_ms must be > 0", check.id));
        }
        if check.effects_required.is_empty() {
            errors.push(format!("{}: effects_required must be declared", check.id));
        }
        for tag in &check.tags {
            if !registry.tags_vocabulary.is_empty()
                && !registry.tags_vocabulary.contains(tag.as_str())
            {
                errors.push(format!("{}: tag `{}` not in vocabulary", check.id, tag));
            }
        }
    }

    let mut suite_ids = BTreeSet::new();
    for suite in &registry.suites {
        if !suite_ids.insert(suite.id.as_str().to_string()) {
            errors.push(format!("duplicate suite id `{}`", suite.id));
        }
        for check_id in &suite.checks {
            if !known_ids.contains(check_id.as_str()) {
                errors.push(format!(
                    "suite {} references unknown check {}",
                    suite.id, check_id
                ));
            }
        }
    }

    errors
}

fn registry_ordering_errors(registry: &Registry) -> Vec<String> {
    let mut errors = Vec::new();
    if !registry
        .checks
        .windows(2)
        .all(|pair| pair[0].id.as_str() <= pair[1].id.as_str())
    {
        errors.push("registry checks must be sorted by id".to_string());
    }
    if !registry
        .suites
        .windows(2)
        .all(|pair| pair[0].id.as_str() <= pair[1].id.as_str())
    {
        errors.push("registry suites must be sorted by id".to_string());
    }
    errors
}

fn wildcard_matches(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == value;
    }
    let mut cursor = 0usize;
    for (idx, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if idx == 0 && !pattern.starts_with('*') {
            if !value[cursor..].starts_with(part) {
                return false;
            }
            cursor += part.len();
            continue;
        }
        if idx == parts.len() - 1 && !pattern.ends_with('*') {
            return value.ends_with(part) && value[cursor..].contains(part);
        }
        if let Some(pos) = value[cursor..].find(part) {
            cursor += pos + part.len();
        } else {
            return false;
        }
    }
    true
}

pub fn expand_suite(registry: &Registry, suite_id: &SuiteId) -> Result<Vec<CheckSpec>, String> {
    let suite = registry
        .suites
        .iter()
        .find(|suite| suite.id == *suite_id)
        .ok_or_else(|| format!("unknown suite `{suite_id}`"))?;

    let mut out: BTreeMap<String, CheckSpec> = BTreeMap::new();
    for check in &registry.checks {
        let in_domain = suite.domains.is_empty() || suite.domains.contains(&check.domain);
        let in_tag =
            suite.tags_any.is_empty() || check.tags.iter().any(|tag| suite.tags_any.contains(tag));
        let explicit = suite.checks.iter().any(|check_id| check_id == &check.id);
        let filters_present = !suite.domains.is_empty() || !suite.tags_any.is_empty();
        if explicit || (filters_present && in_domain && in_tag) {
            out.insert(check.id.as_str().to_string(), check.clone());
        }
    }
    Ok(out.into_values().collect())
}

pub fn select_checks(registry: &Registry, selectors: &Selectors) -> Result<Vec<CheckSpec>, String> {
    let suite_selected = if let Some(suite_id) = &selectors.suite {
        expand_suite(registry, suite_id)?
    } else {
        registry.checks.clone()
    };

    let mut out: Vec<CheckSpec> = suite_selected
        .into_iter()
        .filter(|check| selectors.include_internal || check.visibility == Visibility::Public)
        .filter(|check| {
            selectors.include_slow || !check.tags.iter().any(|tag| tag.as_str() == "slow")
        })
        .filter(|check| selectors.domain.is_none_or(|domain| check.domain == domain))
        .filter(|check| {
            selectors
                .tag
                .as_ref()
                .is_none_or(|tag| check.tags.iter().any(|ctag| ctag == tag))
        })
        .filter(|check| {
            selectors
                .id_glob
                .as_ref()
                .is_none_or(|glob| wildcard_matches(glob, check.id.as_str()))
        })
        .collect();

    out.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    Ok(out)
}

pub fn list_output(checks: &[CheckSpec]) -> String {
    checks
        .iter()
        .map(|check| format!("{}\t{}", check.id, check.title))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn explain_output(registry: &Registry, check_id: &CheckId) -> Result<String, String> {
    let check = registry
        .checks
        .iter()
        .find(|check| check.id == *check_id)
        .ok_or_else(|| format!("unknown check id `{check_id}`"))?;
    let tags = check
        .tags
        .iter()
        .map(Tag::as_str)
        .collect::<Vec<_>>()
        .join(",");
    let suites = check
        .suites
        .iter()
        .map(SuiteId::as_str)
        .collect::<Vec<_>>()
        .join(",");
    let effects = check
        .effects_required
        .iter()
        .map(|v| format!("{v:?}").to_lowercase())
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!(
        "id: {}\ndomain: {:?}\ntitle: {}\ndocs: {}\ntags: {}\nsuites: {}\neffects_required: {}\nbudget_ms: {}\nvisibility: {:?}",
        check.id,
        check.domain,
        check.title,
        check.docs,
        tags,
        suites,
        effects,
        check.budget_ms,
        check.visibility
    ))
}

pub fn registry_doctor(repo_root: &Path) -> RegistryDoctorReport {
    match load_registry(repo_root) {
        Ok(registry) => {
            let mut errors = validate_registry(&registry);
            errors.extend(registry_ordering_errors(&registry));
            let registered_ids: BTreeSet<String> = registry
                .checks
                .iter()
                .map(|check| check.id.as_str().to_string())
                .collect();
            let implemented_ids = check_runner::builtin_check_ids();
            for missing in registered_ids.difference(&implemented_ids) {
                errors.push(format!(
                    "registered check missing implementation `{missing}`"
                ));
            }
            for floating in implemented_ids.difference(&registered_ids) {
                errors.push(format!("check implementation not registered `{floating}`"));
            }
            errors.sort();
            errors.dedup();
            RegistryDoctorReport { errors }
        }
        Err(err) => RegistryDoctorReport { errors: vec![err] },
    }
}

#[cfg(test)]
pub(crate) use check_runner::evidence_path_has_timestamp;
pub use check_runner::{run_checks, Check, CheckRunner};
pub use report_rendering::{
    exit_code_for_report, render_ci_summary_line, render_json, render_jsonl, render_text_summary,
    render_text_with_durations,
};

#[cfg(test)]
mod lib_tests;
