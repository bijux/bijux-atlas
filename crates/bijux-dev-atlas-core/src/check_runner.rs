use crate::*;

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
        if result
            .evidence
            .iter()
            .any(|ev| evidence_path_has_timestamp(&ev.path))
        {
            result.status = CheckStatus::Error;
            result.violations.push(Violation {
                code: "EVIDENCE_PATH_TIMESTAMP_FORBIDDEN".to_string(),
                message: "evidence paths must not include timestamps".to_string(),
                hint: Some("use stable run identifiers and deterministic file names".to_string()),
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
        command: request
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
            ("fs_write".to_string(), request.capabilities.fs_write),
            ("subprocess".to_string(), request.capabilities.subprocess),
            ("git".to_string(), request.capabilities.git),
            ("network".to_string(), request.capabilities.network),
        ]),
        results,
        durations_ms: timings.clone(),
        counts: summary.clone(),
        summary,
        timings_ms: timings,
    })
}
