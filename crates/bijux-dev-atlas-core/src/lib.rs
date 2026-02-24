#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use bijux_dev_atlas_adapters::{Capabilities, Fs, ProcessRunner};
use bijux_dev_atlas_model::{
    CheckId, CheckResult, CheckSpec, CheckStatus, DomainId, Effect, RunId, RunReport, RunSummary,
    Severity, SuiteId, Tag, Violation, Visibility,
};
use serde::Deserialize;

pub mod checks;
pub mod ops_inventory;

pub const DEFAULT_REGISTRY_PATH: &str = "ops/atlas-dev/registry.toml";

#[derive(Debug, Clone)]
pub struct RunRequest {
    pub repo_root: PathBuf,
    pub domain: Option<DomainId>,
    pub capabilities: Capabilities,
    pub artifacts_root: Option<PathBuf>,
    pub run_id: Option<RunId>,
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

pub struct AdapterSet<'a> {
    pub fs: &'a dyn Fs,
    pub process: &'a dyn ProcessRunner,
}

pub struct CheckContext<'a> {
    pub repo_root: &'a Path,
    pub artifacts_root: PathBuf,
    pub run_id: RunId,
    pub adapters: AdapterSet<'a>,
    pub registry: &'a Registry,
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
    let known_ids: BTreeSet<String> = registry
        .checks
        .iter()
        .map(|c| c.id.as_str().to_string())
        .collect();

    for check in &registry.checks {
        if !seen.insert(check.id.as_str().to_string()) {
            errors.push(format!("duplicate check id `{}`", check.id));
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
        if (in_domain && in_tag) || explicit {
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
        Ok(registry) => RegistryDoctorReport {
            errors: validate_registry(&registry),
        },
        Err(err) => RegistryDoctorReport { errors: vec![err] },
    }
}

fn effect_allowed(effect: Effect, caps: Capabilities) -> bool {
    match effect {
        Effect::FsRead => true,
        Effect::FsWrite => caps.fs_write,
        Effect::Subprocess => caps.subprocess,
        Effect::Git => caps.git,
        Effect::Network => caps.network,
    }
}

fn builtin_check_fn(check_id: &CheckId) -> Option<CheckFn> {
    checks::ops::builtin_ops_check_fn(check_id).or_else(|| match check_id.as_str() {
        "repo_import_boundary" => Some(check_repo_import_boundary),
        "docs_index_links" => Some(check_docs_index_links),
        "make_wrapper_commands" => Some(check_make_wrapper_commands),
        _ => None,
    })
}

fn sorted_violations(mut violations: Vec<Violation>) -> Vec<Violation> {
    violations.sort_by(|a, b| {
        a.code
            .cmp(&b.code)
            .then(a.message.cmp(&b.message))
            .then(a.path.cmp(&b.path))
            .then(a.line.cmp(&b.line))
    });
    violations
}

fn check_repo_import_boundary(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let target = Path::new("crates/bijux-atlas-cli/src/atlas_command_dispatch.rs");
    if ctx.adapters.fs.exists(ctx.repo_root, target) {
        Ok(Vec::new())
    } else {
        Ok(vec![Violation {
            code: "REPO_IMPORT_BOUNDARY_SOURCE_MISSING".to_string(),
            message: "missing expected atlas dispatch source file".to_string(),
            hint: Some("restore crate source tree".to_string()),
            path: Some(target.display().to_string()),
            line: None,
            severity: Severity::Error,
        }])
    }
}

fn check_docs_index_links(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let target = Path::new("docs/INDEX.md");
    if ctx.adapters.fs.exists(ctx.repo_root, target) {
        Ok(Vec::new())
    } else {
        Ok(vec![Violation {
            code: "DOCS_INDEX_MISSING".to_string(),
            message: "missing docs/INDEX.md".to_string(),
            hint: Some("restore docs index".to_string()),
            path: Some(target.display().to_string()),
            line: None,
            severity: Severity::Error,
        }])
    }
}

fn check_make_wrapper_commands(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let target = Path::new("makefiles/CONTRACT.md");
    if ctx.adapters.fs.exists(ctx.repo_root, target) {
        Ok(Vec::new())
    } else {
        Ok(vec![Violation {
            code: "MAKE_CONTRACT_MISSING".to_string(),
            message: "missing makefiles/CONTRACT.md".to_string(),
            hint: Some("restore make contract doc".to_string()),
            path: Some(target.display().to_string()),
            line: None,
            severity: Severity::Error,
        }])
    }
}

pub fn render_text_summary(report: &RunReport) -> String {
    format!(
        "summary: passed={} failed={} skipped={} errors={} total={} duration_ms={}",
        report.summary.passed,
        report.summary.failed,
        report.summary.skipped,
        report.summary.errors,
        report.summary.total,
        report.timings_ms.values().sum::<u64>(),
    )
}

pub fn render_text_with_durations(report: &RunReport, top_n: usize) -> String {
    let mut lines = vec![render_text_summary(report)];
    if top_n > 0 {
        let mut rows = report
            .results
            .iter()
            .map(|row| (row.id.as_str().to_string(), row.duration_ms))
            .collect::<Vec<_>>();
        rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        for (id, ms) in rows.into_iter().take(top_n) {
            lines.push(format!("duration: {id} {ms}ms"));
        }
    }
    lines.join("\n")
}

pub fn render_json(report: &RunReport) -> Result<String, String> {
    serde_json::to_string_pretty(report).map_err(|err| err.to_string())
}

pub fn render_jsonl(report: &RunReport) -> Result<String, String> {
    let mut lines = Vec::new();
    for row in &report.results {
        lines.push(serde_json::to_string(row).map_err(|err| err.to_string())?);
    }
    Ok(lines.join("\n"))
}

pub fn run_checks(
    process: &dyn ProcessRunner,
    fs: &dyn Fs,
    request: &RunRequest,
    selectors: &Selectors,
    options: &RunOptions,
) -> Result<RunReport, String> {
    let registry = load_registry(&request.repo_root)?;
    let effective_selectors = Selectors {
        domain: selectors.domain.or(request.domain),
        include_internal: selectors.include_internal,
        include_slow: selectors.include_slow,
        id_glob: selectors.id_glob.clone(),
        tag: selectors.tag.clone(),
        suite: selectors.suite.clone(),
    };
    let selected = select_checks(&registry, &effective_selectors)?;

    let run_id = request
        .run_id
        .clone()
        .unwrap_or_else(|| RunId::from_seed("registry_run"));
    let artifacts_root = request
        .artifacts_root
        .clone()
        .unwrap_or_else(|| request.repo_root.join("artifacts").join("atlas-dev"));
    let ctx = CheckContext {
        repo_root: &request.repo_root,
        artifacts_root: artifacts_root.join(run_id.as_str()),
        run_id,
        adapters: AdapterSet { fs, process },
        registry: &registry,
    };

    let mut timings = BTreeMap::new();
    let mut results = Vec::new();
    let mut failures = 0usize;

    for check in selected {
        let denied = check
            .effects_required
            .iter()
            .find(|effect| !effect_allowed(**effect, request.capabilities));

        if let Some(effect) = denied {
            timings.insert(check.id.clone(), 0);
            results.push(CheckResult {
                id: check.id,
                status: CheckStatus::Skip,
                skip_reason: Some(format!("effect denied: {effect:?}")),
                violations: Vec::new(),
                duration_ms: 0,
                evidence: Vec::new(),
            });
            continue;
        }

        let start = Instant::now();
        let check_fn = builtin_check_fn(&check.id);
        let mut result = CheckResult {
            id: check.id,
            status: CheckStatus::Pass,
            skip_reason: None,
            violations: Vec::new(),
            duration_ms: 0,
            evidence: Vec::new(),
        };

        match check_fn {
            Some(func) => match func(&ctx) {
                Ok(violations) => {
                    result.violations = sorted_violations(violations);
                    result.status = if result.violations.is_empty() {
                        CheckStatus::Pass
                    } else {
                        CheckStatus::Fail
                    };
                }
                Err(err) => {
                    result.status = CheckStatus::Error;
                    result.violations = vec![Violation {
                        code: "CHECK_EXECUTION_ERROR".to_string(),
                        message: match err {
                            CheckError::Failed(msg) => msg,
                        },
                        hint: Some("inspect check runner logs".to_string()),
                        path: None,
                        line: None,
                        severity: Severity::Error,
                    }];
                }
            },
            None => {
                result.status = CheckStatus::Error;
                result.violations = vec![Violation {
                    code: "CHECK_IMPLEMENTATION_MISSING".to_string(),
                    message: "missing check function implementation".to_string(),
                    hint: Some("add builtin_check_fn mapping for this check".to_string()),
                    path: None,
                    line: None,
                    severity: Severity::Error,
                }];
            }
        }

        result.duration_ms = start.elapsed().as_millis() as u64;
        timings.insert(result.id.clone(), result.duration_ms);

        if matches!(result.status, CheckStatus::Fail | CheckStatus::Error) {
            failures += 1;
        }

        results.push(result);

        if options.fail_fast && failures > 0 {
            break;
        }
        if let Some(max) = options.max_failures {
            if failures >= max {
                break;
            }
        }
    }

    results.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));

    let summary = RunSummary {
        passed: results
            .iter()
            .filter(|row| row.status == CheckStatus::Pass)
            .count() as u64,
        failed: results
            .iter()
            .filter(|row| row.status == CheckStatus::Fail)
            .count() as u64,
        skipped: results
            .iter()
            .filter(|row| row.status == CheckStatus::Skip)
            .count() as u64,
        errors: results
            .iter()
            .filter(|row| row.status == CheckStatus::Error)
            .count() as u64,
        total: results.len() as u64,
    };

    Ok(RunReport {
        run_id: ctx.run_id,
        repo_root: request.repo_root.display().to_string(),
        results,
        summary,
        timings_ms: timings,
    })
}

pub fn exit_code_for_report(report: &RunReport) -> i32 {
    if report.summary.errors > 0 {
        3
    } else if report.summary.failed > 0 {
        2
    } else {
        0
    }
}


#[cfg(test)]
mod lib_tests;
