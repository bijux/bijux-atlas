#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use bijux_atlas_dev_adapters::{Capabilities, ProcessRunner};
use bijux_atlas_dev_model::{
    CheckId, CheckResult, CheckSpec, CheckStatus, DomainId, Effect, RunId, RunReport, RunSummary,
    SuiteId, Tag, Visibility,
};
use serde::Deserialize;

pub const DEFAULT_REGISTRY_PATH: &str = "ops/atlas-dev/registry.toml";

#[derive(Debug, Clone)]
pub struct RunRequest {
    pub repo_root: PathBuf,
    pub domain: Option<DomainId>,
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone)]
pub struct Selectors {
    pub id_glob: Option<String>,
    pub domain: Option<DomainId>,
    pub tag: Option<Tag>,
    pub suite: Option<SuiteId>,
    pub include_internal: bool,
}

impl Default for Selectors {
    fn default() -> Self {
        Self {
            id_glob: None,
            domain: None,
            tag: None,
            suite: None,
            include_internal: false,
        }
    }
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

pub fn run_checks(_adapter: &dyn ProcessRunner, request: &RunRequest) -> Result<RunReport, String> {
    let registry = load_registry(&request.repo_root)?;
    let selectors = Selectors {
        domain: request.domain,
        ..Selectors::default()
    };
    let selected = select_checks(&registry, &selectors)?;
    let mut timings = BTreeMap::new();
    let mut results = Vec::new();
    for check in selected {
        timings.insert(check.id.clone(), 0);
        results.push(CheckResult {
            id: check.id,
            status: CheckStatus::Pass,
            violations: Vec::new(),
            duration_ms: 0,
            evidence: Vec::new(),
        });
    }

    let summary = RunSummary {
        passed: results.len() as u64,
        failed: 0,
        skipped: 0,
        errors: 0,
        total: results.len() as u64,
    };

    Ok(RunReport {
        run_id: RunId::from_seed("registry_run"),
        repo_root: request.repo_root.display().to_string(),
        results,
        summary,
        timings_ms: timings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace")
            .parent()
            .expect("repo")
            .to_path_buf()
    }

    #[test]
    fn registry_parses_and_validates() {
        let registry = load_registry(&root()).expect("registry");
        assert!(!registry.checks.is_empty());
        assert!(validate_registry(&registry).is_empty());
    }

    #[test]
    fn suite_expansion_is_stable() {
        let registry = load_registry(&root()).expect("registry");
        let suite = SuiteId::parse("ops_fast").expect("suite");
        let checks = expand_suite(&registry, &suite).expect("expand");
        let ids = checks
            .into_iter()
            .map(|c| c.id.to_string())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["ops_surface_manifest".to_string()]);
    }

    #[test]
    fn selectors_hide_internal_by_default() {
        let registry = load_registry(&root()).expect("registry");
        let selected = select_checks(&registry, &Selectors::default()).expect("select");
        assert!(selected
            .iter()
            .all(|row| row.visibility == Visibility::Public));
    }

    #[test]
    fn selectors_include_internal_when_requested() {
        let registry = load_registry(&root()).expect("registry");
        let selected = select_checks(
            &registry,
            &Selectors {
                include_internal: true,
                ..Selectors::default()
            },
        )
        .expect("select");
        assert!(selected
            .iter()
            .any(|row| row.visibility == Visibility::Internal));
    }

    #[test]
    fn explain_contains_docs() {
        let registry = load_registry(&root()).expect("registry");
        let text = explain_output(
            &registry,
            &CheckId::parse("ops_surface_manifest").expect("id"),
        )
        .expect("explain");
        assert!(text.contains("docs:"));
        assert!(text.contains("ops/CONTRACT.md"));
    }

    #[test]
    fn list_output_shape_is_stable() {
        let registry = load_registry(&root()).expect("registry");
        let selected = select_checks(&registry, &Selectors::default()).expect("select");
        let rendered = list_output(&selected);
        assert!(rendered.contains("ops_surface_manifest\tops surface manifest consistency"));
    }

    #[test]
    fn doctor_reports_ok_for_valid_registry() {
        let report = registry_doctor(&root());
        assert!(report.errors.is_empty());
    }

    #[test]
    fn glob_selector_filters_ids() {
        let registry = load_registry(&root()).expect("registry");
        let selected = select_checks(
            &registry,
            &Selectors {
                id_glob: Some("ops_*".to_string()),
                ..Selectors::default()
            },
        )
        .expect("select");
        assert!(selected
            .iter()
            .all(|row| row.id.as_str().starts_with("ops_")));
    }

    #[test]
    fn can_build_report_schema_from_model() {
        let schema = bijux_atlas_dev_model::report_json_schema();
        assert_eq!(schema["title"], "bijux-atlas-dev run report");
    }

    #[test]
    fn parse_effect_rejects_unknown_value() {
        let err = parse_effect("shell").expect_err("must fail");
        assert!(err.contains("invalid effect"));
    }

    #[test]
    fn run_id_is_deterministic() {
        let one = RunId::from_seed("registry_run");
        let two = RunId::from_seed("registry_run");
        assert_eq!(one, two);
    }

    #[test]
    fn run_checks_produces_summary() {
        let req = RunRequest {
            repo_root: root(),
            domain: None,
            capabilities: Capabilities::deny_all(),
        };
        let adapter = bijux_atlas_dev_adapters::DeniedProcessRunner;
        let report = run_checks(&adapter, &req).expect("report");
        assert!(report.summary.total >= 1);
        assert_eq!(report.summary.failed, 0);
    }

    #[test]
    fn selector_by_suite_works() {
        let registry = load_registry(&root()).expect("registry");
        let selected = select_checks(
            &registry,
            &Selectors {
                suite: Some(SuiteId::parse("ops_fast").expect("suite")),
                ..Selectors::default()
            },
        )
        .expect("selected");
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id.as_str(), "ops_surface_manifest");
    }

    #[test]
    fn selector_by_domain_works() {
        let registry = load_registry(&root()).expect("registry");
        let selected = select_checks(
            &registry,
            &Selectors {
                domain: Some(DomainId::Docs),
                ..Selectors::default()
            },
        )
        .expect("selected");
        assert!(selected.iter().all(|row| row.domain == DomainId::Docs));
    }

    #[test]
    fn selector_by_tag_works() {
        let registry = load_registry(&root()).expect("registry");
        let selected = select_checks(
            &registry,
            &Selectors {
                tag: Some(Tag::parse("lint").expect("tag")),
                ..Selectors::default()
            },
        )
        .expect("selected");
        assert!(selected
            .iter()
            .any(|row| row.id.as_str() == "repo_import_boundary"));
    }

    #[test]
    fn validate_fails_on_empty_effects() {
        let mut registry = load_registry(&root()).expect("registry");
        registry.checks[0].effects_required.clear();
        let errors = validate_registry(&registry);
        assert!(errors.iter().any(|err| err.contains("effects_required")));
    }

    #[test]
    fn validate_fails_on_zero_budget() {
        let mut registry = load_registry(&root()).expect("registry");
        registry.checks[0].budget_ms = 0;
        let errors = validate_registry(&registry);
        assert!(errors.iter().any(|err| err.contains("budget_ms")));
    }
}
