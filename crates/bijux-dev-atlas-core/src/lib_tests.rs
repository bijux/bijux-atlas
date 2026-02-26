// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct TestFs;

impl Fs for TestFs {
    fn read_text(
        &self,
        repo_root: &Path,
        path: &Path,
    ) -> Result<String, crate::ports::AdapterError> {
        let target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };
        fs::read_to_string(target).map_err(|err| crate::ports::AdapterError::Io {
            op: "read_to_string",
            path: repo_root.join(path),
            detail: err.to_string(),
        })
    }

    fn exists(&self, repo_root: &Path, path: &Path) -> bool {
        let target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };
        target.exists()
    }

    fn canonicalize(
        &self,
        repo_root: &Path,
        path: &Path,
    ) -> Result<PathBuf, crate::ports::AdapterError> {
        let target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };
        target
            .canonicalize()
            .map_err(|err| crate::ports::AdapterError::Io {
                op: "canonicalize",
                path: target,
                detail: err.to_string(),
            })
    }
}

struct DeniedProcessRunner;

impl ProcessRunner for DeniedProcessRunner {
    fn run(
        &self,
        program: &str,
        _args: &[String],
        _repo_root: &Path,
    ) -> Result<i32, crate::ports::AdapterError> {
        Err(crate::ports::AdapterError::EffectDenied {
            effect: "subprocess",
            detail: format!("attempted to execute `{program}`"),
        })
    }
}

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
    assert_eq!(
        ids,
        vec![
            "checks_ops_artifacts_gitignore_policy".to_string(),
            "checks_ops_artifacts_not_tracked".to_string(),
            "checks_ops_control_plane_doc_contract".to_string(),
            "checks_ops_generated_readonly_markers".to_string(),
            "checks_ops_makefile_routes_dev_atlas".to_string(),
            "checks_ops_manifest_integrity".to_string(),
            "checks_ops_no_scripts_areas_or_xtask_refs".to_string(),
            "checks_ops_retired_artifact_path_references_absent".to_string(),
            "checks_ops_runtime_output_roots_under_ops_absent".to_string(),
            "checks_ops_schema_presence".to_string(),
            "checks_ops_ssot_manifests_schema_versions".to_string(),
            "checks_ops_surface_inventory".to_string(),
            "checks_ops_surface_manifest".to_string(),
            "checks_ops_tree_contract".to_string(),
            "checks_ops_workflow_routes_dev_atlas".to_string(),
        ]
    );
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
            include_slow: true,
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
        &CheckId::parse("checks_ops_surface_manifest").expect("id"),
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
    assert!(rendered.contains("checks_ops_surface_manifest\tops surface manifest consistency"));
}

#[test]
fn doctor_reports_ok_for_valid_registry() {
    let report = registry_doctor(&root());
    assert!(
        report.errors.is_empty(),
        "registry_doctor errors: {:?}",
        report.errors
    );
}

#[test]
fn validate_flags_unsorted_registry_ordering() {
    let mut registry = load_registry(&root()).expect("registry");
    registry.checks.swap(0, 1);
    let errors = registry_ordering_errors(&registry);
    assert!(errors
        .iter()
        .any(|err| err == "registry checks must be sorted by id"));
}

#[test]
fn doctor_detects_missing_implementation_for_registered_check() {
    let mut registry = load_registry(&root()).expect("registry");
    registry.checks.push(CheckSpec {
        id: CheckId::parse("checks_ops_sample_unimplemented").expect("id"),
        domain: DomainId::Ops,
        title: "sample unimplemented".to_string(),
        docs: "ops/CONTRACT.md".to_string(),
        tags: vec![Tag::parse("lint").expect("tag")],
        suites: vec![SuiteId::parse("deep").expect("suite")],
        effects_required: vec![Effect::FsRead],
        budget_ms: 100,
        visibility: Visibility::Internal,
    });
    let mut errors = validate_registry(&registry);
    let registered_ids: std::collections::BTreeSet<String> = registry
        .checks
        .iter()
        .map(|check| check.id.as_str().to_string())
        .collect();
    let implemented_ids = crate::check_runner::builtin_check_ids();
    for missing in registered_ids.difference(&implemented_ids) {
        errors.push(format!(
            "registered check missing implementation `{missing}`"
        ));
    }
    assert!(errors.iter().any(|err| {
        err == "registered check missing implementation `checks_ops_sample_unimplemented`"
    }));
}

#[test]
fn glob_selector_filters_ids() {
    let registry = load_registry(&root()).expect("registry");
    let selected = select_checks(
        &registry,
        &Selectors {
            id_glob: Some("checks_ops_*".to_string()),
            ..Selectors::default()
        },
    )
    .expect("select");
    assert!(selected
        .iter()
        .all(|row| row.id.as_str().starts_with("checks_ops_")));
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
fn artifacts_root_defaults_to_repo_artifacts_atlas_dev() {
    let repo = root();
    let artifacts = ArtifactsRoot::default_for_repo(&repo);
    assert_eq!(
        artifacts.as_path(),
        repo.join("artifacts").join("atlas-dev").as_path()
    );
}

#[test]
fn runtime_context_resolves_artifacts_and_run_id_deterministically() {
    let req = RunRequest {
        repo_root: root(),
        domain: None,
        capabilities: Capabilities::deny_all(),
        artifacts_root: None,
        run_id: None,
        command: None,
    };
    let runtime = RuntimeContext::from_run_request(&req).expect("runtime");
    assert_eq!(runtime.run_id.as_str(), "registry_run");
    assert_eq!(
        runtime.check_artifacts_run_root(),
        root()
            .join("artifacts")
            .join("atlas-dev")
            .join("registry_run")
    );
}

#[test]
fn run_checks_produces_summary() {
    let req = RunRequest {
        repo_root: root(),
        domain: None,
        capabilities: Capabilities::deny_all(),
        artifacts_root: None,
        run_id: None,
        command: None,
    };
    let report = run_checks(
        &DeniedProcessRunner,
        &TestFs,
        &req,
        &Selectors::default(),
        &RunOptions::default(),
    )
    .expect("report");
    assert!(report.summary.total >= 1);
}

#[test]
fn runner_results_are_stably_ordered_and_repeatable() {
    let req = RunRequest {
        repo_root: root(),
        domain: None,
        capabilities: Capabilities::deny_all(),
        artifacts_root: None,
        run_id: None,
        command: None,
    };
    let selectors = Selectors::default();
    let options = RunOptions::default();
    let first = run_checks(&DeniedProcessRunner, &TestFs, &req, &selectors, &options)
        .expect("first report");
    let second = run_checks(&DeniedProcessRunner, &TestFs, &req, &selectors, &options)
        .expect("second report");
    let first_ids = first
        .results
        .iter()
        .map(|row| row.id.as_str().to_string())
        .collect::<Vec<_>>();
    let second_ids = second
        .results
        .iter()
        .map(|row| row.id.as_str().to_string())
        .collect::<Vec<_>>();
    let mut sorted = first_ids.clone();
    sorted.sort();
    assert_eq!(first_ids, sorted);
    assert_eq!(first_ids, second_ids);
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
    assert!(selected.len() >= 11);
    for required in [
        "checks_ops_surface_manifest",
        "checks_ops_artifacts_gitignore_policy",
        "checks_ops_retired_artifact_path_references_absent",
    ] {
        assert!(selected.iter().any(|row| row.id.as_str() == required));
    }
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
        .any(|row| row.id.as_str() == "checks_repo_import_boundary"));
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

#[test]
fn effect_denied_results_in_skip() {
    let req = RunRequest {
        repo_root: root(),
        domain: Some(DomainId::Ops),
        capabilities: Capabilities::deny_all(),
        artifacts_root: None,
        run_id: None,
        command: None,
    };
    let report = run_checks(
        &DeniedProcessRunner,
        &TestFs,
        &req,
        &Selectors {
            include_internal: true,
            include_slow: true,
            ..Selectors::default()
        },
        &RunOptions::default(),
    )
    .expect("report");
    assert!(report
        .results
        .iter()
        .any(|row| row.status == CheckStatus::Skip));
}

#[test]
fn fail_fast_stops_after_first_failure() {
    let req = RunRequest {
        repo_root: root(),
        domain: Some(DomainId::Ops),
        capabilities: Capabilities::from_cli_flags(false, false, true, false),
        artifacts_root: None,
        run_id: None,
        command: None,
    };
    let report = run_checks(
        &DeniedProcessRunner,
        &TestFs,
        &req,
        &Selectors {
            include_internal: true,
            include_slow: true,
            ..Selectors::default()
        },
        &RunOptions {
            fail_fast: true,
            max_failures: None,
        },
    )
    .expect("report");
    assert!(report.summary.failed + report.summary.errors >= 1);
    assert!(report.summary.total < 9);
}

#[test]
fn deterministic_json_output() {
    let req = RunRequest {
        repo_root: root(),
        domain: None,
        capabilities: Capabilities::from_cli_flags(false, true, false, false),
        artifacts_root: None,
        run_id: None,
        command: None,
    };
    let a = run_checks(
        &DeniedProcessRunner,
        &TestFs,
        &req,
        &Selectors::default(),
        &RunOptions::default(),
    )
    .expect("report a");
    let b = run_checks(
        &DeniedProcessRunner,
        &TestFs,
        &req,
        &Selectors::default(),
        &RunOptions::default(),
    )
    .expect("report b");
    let mut a = a;
    let mut b = b;
    for row in &mut a.results {
        row.duration_ms = 0;
    }
    for row in &mut b.results {
        row.duration_ms = 0;
    }
    for value in a.timings_ms.values_mut() {
        *value = 0;
    }
    for value in b.timings_ms.values_mut() {
        *value = 0;
    }
    for value in a.durations_ms.values_mut() {
        *value = 0;
    }
    for value in b.durations_ms.values_mut() {
        *value = 0;
    }
    let a_text = render_json(&a).expect("json a");
    let b_text = render_json(&b).expect("json b");
    assert_eq!(a_text, b_text);
}

#[test]
fn exit_code_mapping_is_distinct_for_fail_and_error() {
    let pass_report = RunReport {
        schema_version: bijux_dev_atlas_model::schema_version(),
        run_id: RunId::from_seed("pass"),
        repo_root: ".".to_string(),
        command: "check run".to_string(),
        selections: BTreeMap::new(),
        capabilities: BTreeMap::new(),
        results: Vec::new(),
        durations_ms: BTreeMap::new(),
        counts: RunSummary {
            schema_version: bijux_dev_atlas_model::schema_version(),
            passed: 1,
            failed: 0,
            skipped: 0,
            errors: 0,
            total: 1,
        },
        summary: RunSummary {
            schema_version: bijux_dev_atlas_model::schema_version(),
            passed: 1,
            failed: 0,
            skipped: 0,
            errors: 0,
            total: 1,
        },
        timings_ms: BTreeMap::new(),
    };
    assert_eq!(exit_code_for_report(&pass_report), 0);

    let fail_report = RunReport {
        summary: RunSummary {
            schema_version: bijux_dev_atlas_model::schema_version(),
            passed: 0,
            failed: 1,
            skipped: 0,
            errors: 0,
            total: 1,
        },
        ..pass_report.clone()
    };
    assert_eq!(exit_code_for_report(&fail_report), 2);

    let error_report = RunReport {
        summary: RunSummary {
            schema_version: bijux_dev_atlas_model::schema_version(),
            passed: 0,
            failed: 0,
            skipped: 0,
            errors: 1,
            total: 1,
        },
        ..pass_report
    };
    assert_eq!(exit_code_for_report(&error_report), 3);

    let skip_report = RunReport {
        summary: RunSummary {
            schema_version: bijux_dev_atlas_model::schema_version(),
            passed: 0,
            failed: 0,
            skipped: 2,
            errors: 0,
            total: 2,
        },
        ..error_report
    };
    assert_eq!(exit_code_for_report(&skip_report), 4);
}

#[test]
fn duration_output_is_deterministic_for_equal_durations() {
    let report = RunReport {
        schema_version: bijux_dev_atlas_model::schema_version(),
        run_id: RunId::from_seed("durations"),
        repo_root: ".".to_string(),
        command: "check run".to_string(),
        selections: BTreeMap::new(),
        capabilities: BTreeMap::new(),
        results: vec![
            CheckResult {
                schema_version: bijux_dev_atlas_model::schema_version(),
                id: CheckId::parse("checks_ops_surface_manifest").expect("id"),
                status: CheckStatus::Pass,
                skip_reason: None,
                violations: Vec::new(),
                duration_ms: 50,
                evidence: Vec::new(),
            },
            CheckResult {
                schema_version: bijux_dev_atlas_model::schema_version(),
                id: CheckId::parse("checks_ops_tree_contract").expect("id"),
                status: CheckStatus::Pass,
                skip_reason: None,
                violations: Vec::new(),
                duration_ms: 50,
                evidence: Vec::new(),
            },
        ],
        summary: RunSummary {
            schema_version: bijux_dev_atlas_model::schema_version(),
            passed: 2,
            failed: 0,
            skipped: 0,
            errors: 0,
            total: 2,
        },
        durations_ms: BTreeMap::new(),
        counts: RunSummary {
            schema_version: bijux_dev_atlas_model::schema_version(),
            passed: 2,
            failed: 0,
            skipped: 0,
            errors: 0,
            total: 2,
        },
        timings_ms: BTreeMap::new(),
    };
    let rendered = render_text_with_durations(&report, 2);
    let lines: Vec<&str> = rendered.lines().collect();
    assert!(lines.iter().any(|line| line.starts_with("CI_SUMMARY ")));
    assert!(lines
        .iter()
        .any(|line| line.contains("duration: checks_ops_surface_manifest 50ms")));
    assert!(lines
        .iter()
        .any(|line| line.contains("duration: checks_ops_tree_contract 50ms")));
    let first_duration = lines
        .iter()
        .find(|line| line.starts_with("duration:"))
        .expect("first duration");
    assert_eq!(
        *first_duration,
        "duration: checks_ops_surface_manifest 50ms"
    );
}

#[test]
fn ops_inventory_validation_is_clean_for_repo_ssot() {
    let errors = ops_inventory::validate_ops_inventory(&root());
    assert!(errors.is_empty(), "ops inventory errors: {errors:?}");
}

#[test]
fn ops_inventory_summary_reports_counts() {
    let summary = ops_inventory::ops_inventory_summary(&root()).expect("summary");
    assert!(
        summary
            .get("stack_profiles")
            .and_then(|value| value.as_u64())
            .unwrap_or(0)
            >= 1
    );
    assert!(
        summary
            .get("surface_actions")
            .and_then(|value| value.as_u64())
            .unwrap_or(0)
            >= 1
    );
}

#[test]
fn evidence_paths_must_not_include_timestamps() {
    assert!(evidence_path_has_timestamp(
        "artifacts/atlas-dev/run_20260224/report.json"
    ));
    assert!(!evidence_path_has_timestamp(
        "artifacts/atlas-dev/run_registry/report.json"
    ));
}

fn write_temp_registry(temp: &TempDir, body: &str) -> PathBuf {
    let path = temp.path().join("ops/inventory");
    fs::create_dir_all(&path).expect("mkdir");
    let registry = path.join("registry.toml");
    fs::write(&registry, body).expect("write registry");
    registry
}

#[test]
fn registry_parse_errors_are_stable_and_readable() {
    let temp = TempDir::new().expect("tempdir");
    write_temp_registry(
        &temp,
        r#"
[[checks]]
id = "checks_ops_example"
domain = "ops"
"#,
    );
    let err = load_registry(temp.path()).expect_err("must fail");
    assert!(err.contains("failed to parse"));
    assert!(err.contains("ops/inventory/registry.toml"));
}

#[test]
fn suite_unknown_check_errors_are_stable_and_readable() {
    let temp = TempDir::new().expect("tempdir");
    write_temp_registry(
        &temp,
        r#"
[tags]
vocabulary = ["ci", "lint"]

[[checks]]
id = "checks_ops_sample_example"
domain = "ops"
title = "example check"
docs = "ops/CONTRACT.md"
tags = ["ci"]
suites = ["ci_fast"]
effects_required = ["fs_read"]
budget_ms = 100
visibility = "public"

[[suites]]
id = "ci_fast"
checks = ["checks_ops_sample_missing_impl"]
domains = []
tags_any = []
"#,
    );
    let err = load_registry(temp.path()).expect_err("must fail");
    assert!(err.contains("suite ci_fast references unknown check checks_ops_sample_missing_impl"));
}
