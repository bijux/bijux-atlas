// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};

use super::*;

fn known_violation_id(raw: &str) -> crate::model::ViolationId {
    match crate::model::ViolationId::parse(raw) {
        Ok(id) => id,
        Err(_) => panic!("static violation id literal must be valid"),
    }
}

fn known_artifact_path(raw: String) -> crate::model::ArtifactPath {
    match crate::model::ArtifactPath::parse(&raw) {
        Ok(path) => path,
        Err(_) => panic!("static artifact path must be valid"),
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
        "checks_repo_import_boundary" => Some(check_repo_import_boundary),
        "checks_docs_index_links" => Some(check_docs_index_links),
        "checks_make_wrapper_commands" => Some(check_make_wrapper_commands),
        _ => None,
    })
}

pub(crate) fn builtin_check_ids() -> BTreeSet<String> {
    let mut ids = checks::ops::builtin_ops_check_ids();
    ids.insert("checks_repo_import_boundary".to_string());
    ids.insert("checks_docs_index_links".to_string());
    ids.insert("checks_make_wrapper_commands".to_string());
    ids
}

pub trait Check {
    fn id(&self) -> &CheckId;
    fn description(&self) -> &str;
    fn tags(&self) -> &[Tag];
    fn inputs(&self) -> &[Effect];
    fn run(&self, ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError>;
}

#[derive(Clone)]
pub struct BuiltinCheck {
    spec: CheckSpec,
    run_fn: CheckFn,
}

impl BuiltinCheck {
    fn from_spec(spec: CheckSpec, run_fn: CheckFn) -> Self {
        Self { spec, run_fn }
    }
}

impl Check for BuiltinCheck {
    fn id(&self) -> &CheckId {
        &self.spec.id
    }

    fn description(&self) -> &str {
        &self.spec.title
    }

    fn tags(&self) -> &[Tag] {
        &self.spec.tags
    }

    fn inputs(&self) -> &[Effect] {
        &self.spec.effects_required
    }

    fn run(&self, ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
        (self.run_fn)(ctx)
    }
}

pub struct CheckRunner<'a> {
    fs: &'a dyn Fs,
    process: &'a dyn ProcessRunner,
    request: &'a RunRequest,
    selectors: &'a Selectors,
    options: &'a RunOptions,
}

impl<'a> CheckRunner<'a> {
    pub fn new(
        process: &'a dyn ProcessRunner,
        fs: &'a dyn Fs,
        request: &'a RunRequest,
        selectors: &'a Selectors,
        options: &'a RunOptions,
    ) -> Self {
        Self {
            fs,
            process,
            request,
            selectors,
            options,
        }
    }

    fn selected_checks(&self, registry: &Registry) -> Result<Vec<BuiltinCheck>, String> {
        let effective_selectors = Selectors {
            domain: self.selectors.domain.or(self.request.domain),
            include_internal: self.selectors.include_internal,
            include_slow: self.selectors.include_slow,
            id_glob: self.selectors.id_glob.clone(),
            tag: self.selectors.tag.clone(),
            suite: self.selectors.suite.clone(),
        };
        let selected = select_checks(registry, &effective_selectors)?;
        selected
            .into_iter()
            .map(|spec| match builtin_check_fn(&spec.id) {
                Some(run_fn) => Ok(BuiltinCheck::from_spec(spec, run_fn)),
                None => Err(format!(
                    "missing check implementation for {}",
                    spec.id.as_str()
                )),
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub fn run(&self) -> Result<RunReport, String> {
        let registry = load_registry(&self.request.repo_root)?;
        let checks = self.selected_checks(&registry)?;
        let runtime = RuntimeContext::from_run_request(self.request)?;
        let ctx = CheckContext {
            repo_root: &runtime.repo_root,
            artifacts_root: runtime.check_artifacts_run_root(),
            run_id: runtime.run_id.clone(),
            adapters: AdapterSet {
                fs: self.fs,
                process: self.process,
            },
            registry: &registry,
        };

        let mut timings = BTreeMap::new();
        let mut results = Vec::new();
        let mut failures = 0usize;

        for check in checks {
            let denied = check
                .inputs()
                .iter()
                .find(|effect| !effect_allowed(**effect, runtime.capabilities));

            if let Some(effect) = denied {
                let check_id = check.id().clone();
                timings.insert(check_id.clone(), 0);
                results.push(CheckResult {
                    schema_version: crate::model::schema_version(),
                    id: check_id,
                    status: CheckStatus::Skip,
                    skip_reason: Some(format!("effect denied: {effect:?}")),
                    violations: Vec::new(),
                    duration_ms: 0,
                    evidence: Vec::new(),
                });
                continue;
            }

            let start = Instant::now();
            let mut result = CheckResult {
                schema_version: crate::model::schema_version(),
                id: check.id().clone(),
                status: CheckStatus::Pass,
                skip_reason: None,
                violations: Vec::new(),
                duration_ms: 0,
                evidence: Vec::new(),
            };

            match check.run(&ctx) {
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
                        schema_version: crate::model::schema_version(),
                        code: known_violation_id("check_execution_error"),
                        message: match err {
                            CheckError::Failed(msg) => msg,
                        },
                        hint: Some("inspect check runner logs".to_string()),
                        path: None,
                        line: None,
                        severity: Severity::Error,
                    }];
                }
            }

            result.duration_ms = start.elapsed().as_millis() as u64;
            if result
                .evidence
                .iter()
                .any(|ev| evidence_path_has_timestamp(ev.path.as_str()))
            {
                result.status = CheckStatus::Error;
                result.violations.push(Violation {
                    schema_version: crate::model::schema_version(),
                    code: known_violation_id("evidence_path_timestamp_forbidden"),
                    message: "evidence paths must not include timestamps".to_string(),
                    hint: Some(
                        "use stable run identifiers and deterministic file names".to_string(),
                    ),
                    path: None,
                    line: None,
                    severity: Severity::Error,
                });
            }
            timings.insert(result.id.clone(), result.duration_ms);

            if matches!(result.status, CheckStatus::Fail | CheckStatus::Error) {
                failures += 1;
            }

            results.push(result);

            if self.options.fail_fast && failures > 0 {
                break;
            }
            if let Some(max) = self.options.max_failures {
                if failures >= max {
                    break;
                }
            }
        }

        results.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));

        let summary = RunSummary {
            schema_version: crate::model::schema_version(),
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

        let effective_selectors = Selectors {
            domain: self.selectors.domain.or(self.request.domain),
            include_internal: self.selectors.include_internal,
            include_slow: self.selectors.include_slow,
            id_glob: self.selectors.id_glob.clone(),
            tag: self.selectors.tag.clone(),
            suite: self.selectors.suite.clone(),
        };

        Ok(RunReport {
            schema_version: crate::model::schema_version(),
            run_id: ctx.run_id,
            repo_root: runtime.repo_root.display().to_string(),
            command: self
                .request
                .command
                .clone()
                .unwrap_or_else(|| "check run".to_string()),
            selections: BTreeMap::from([
                (
                    "suite".to_string(),
                    effective_selectors
                        .suite
                        .as_ref()
                        .map_or_else(String::new, |v| v.as_str().to_string()),
                ),
                (
                    "domain".to_string(),
                    effective_selectors
                        .domain
                        .map_or_else(String::new, |v| format!("{v:?}").to_lowercase()),
                ),
                (
                    "tag".to_string(),
                    effective_selectors
                        .tag
                        .as_ref()
                        .map_or_else(String::new, |v| v.as_str().to_string()),
                ),
                (
                    "id_glob".to_string(),
                    effective_selectors.id_glob.clone().unwrap_or_default(),
                ),
            ]),
            capabilities: BTreeMap::from([
                ("fs_write".to_string(), runtime.capabilities.fs_write),
                ("subprocess".to_string(), runtime.capabilities.subprocess),
                ("git".to_string(), runtime.capabilities.git),
                ("network".to_string(), runtime.capabilities.network),
            ]),
            results,
            durations_ms: timings.clone(),
            counts: summary.clone(),
            summary,
            timings_ms: timings,
        })
    }
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

pub(crate) fn evidence_path_has_timestamp(path: &str) -> bool {
    let bytes = path.as_bytes();
    let mut run = 0usize;
    for b in bytes {
        if b.is_ascii_digit() {
            run += 1;
            if run >= 8 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

fn check_repo_import_boundary(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let target = Path::new("crates/bijux-atlas-cli/src/atlas_command_dispatch.rs");
    if ctx.adapters.fs.exists(ctx.repo_root, target) {
        Ok(Vec::new())
    } else {
        Ok(vec![Violation {
            schema_version: crate::model::schema_version(),
            code: known_violation_id("repo_import_boundary_source_missing"),
            message: "missing expected atlas dispatch source file".to_string(),
            hint: Some("restore crate source tree".to_string()),
            path: Some(known_artifact_path(target.display().to_string())),
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
            schema_version: crate::model::schema_version(),
            code: known_violation_id("docs_index_missing"),
            message: "missing docs/INDEX.md".to_string(),
            hint: Some("restore docs index".to_string()),
            path: Some(known_artifact_path(target.display().to_string())),
            line: None,
            severity: Severity::Error,
        }])
    }
}

fn check_make_wrapper_commands(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let target = Path::new("make/makefiles/CONTRACT.md");
    if ctx.adapters.fs.exists(ctx.repo_root, target) {
        Ok(Vec::new())
    } else {
        Ok(vec![Violation {
            schema_version: crate::model::schema_version(),
            code: known_violation_id("make_contract_missing"),
            message: "missing make/makefiles/CONTRACT.md".to_string(),
            hint: Some("restore make contract doc".to_string()),
            path: Some(known_artifact_path(target.display().to_string())),
            line: None,
            severity: Severity::Error,
        }])
    }
}

pub fn run_checks(
    process: &dyn ProcessRunner,
    fs: &dyn Fs,
    request: &RunRequest,
    selectors: &Selectors,
    options: &RunOptions,
) -> Result<RunReport, String> {
    CheckRunner::new(process, fs, request, selectors, options).run()
}
