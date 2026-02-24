use super::*;
use bijux_dev_atlas_adapters::{DeniedProcessRunner, RealFs};

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
            "ops_artifacts_not_tracked".to_string(),
            "ops_generated_readonly_markers".to_string(),
            "ops_manifest_integrity".to_string(),
            "ops_no_legacy_tooling_refs".to_string(),
            "ops_schema_presence".to_string(),
            "ops_surface_inventory".to_string(),
            "ops_surface_manifest".to_string(),
            "ops_tree_contract".to_string(),
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
        artifacts_root: None,
        run_id: None,
    };
    let report = run_checks(
        &DeniedProcessRunner,
        &RealFs,
        &req,
        &Selectors::default(),
        &RunOptions::default(),
    )
    .expect("report");
    assert!(report.summary.total >= 1);
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
    assert_eq!(selected.len(), 8);
    assert!(selected
        .iter()
        .any(|row| row.id.as_str() == "ops_surface_manifest"));
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

#[test]
fn effect_denied_results_in_skip() {
    let req = RunRequest {
        repo_root: root(),
        domain: Some(DomainId::Ops),
        capabilities: Capabilities::deny_all(),
        artifacts_root: None,
        run_id: None,
    };
    let report = run_checks(
        &DeniedProcessRunner,
        &RealFs,
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
    };
    let report = run_checks(
        &DeniedProcessRunner,
        &RealFs,
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
    };
    let a = run_checks(
        &DeniedProcessRunner,
        &RealFs,
        &req,
        &Selectors::default(),
        &RunOptions::default(),
    )
    .expect("report a");
    let b = run_checks(
        &DeniedProcessRunner,
        &RealFs,
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
    let a_text = render_json(&a).expect("json a");
    let b_text = render_json(&b).expect("json b");
    assert_eq!(a_text, b_text);
}
