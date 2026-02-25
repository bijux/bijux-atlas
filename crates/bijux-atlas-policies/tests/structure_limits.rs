// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use bijux_atlas_policies::{
    canonical_config_json, load_policy_from_workspace,
    validate_policy_change_requires_version_bump, validate_policy_config,
    validate_schema_version_transition, CacheBudget, ConcurrencyBulkheads, DocumentedDefault,
    EndpointClassBudget, PolicyConfig, PolicyMode, PolicyModeProfile, PolicyModes,
    PolicySchemaVersion, PublishGates, QueryBudgetPolicy, RateLimitPolicy, ResponseBudgetPolicy,
    StoreResiliencePolicy, TelemetryPolicy,
};

fn workspace_root() -> PathBuf {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--locked")
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to run cargo metadata for workspace root");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid cargo metadata JSON");
    PathBuf::from(
        value
            .get("workspace_root")
            .and_then(serde_json::Value::as_str)
            .expect("workspace_root missing from metadata"),
    )
}

fn valid_policy() -> PolicyConfig {
    PolicyConfig {
        schema_version: PolicySchemaVersion::V1,
        mode: PolicyMode::Strict,
        allow_override: false,
        network_in_unit_tests: false,
        modes: PolicyModes {
            strict: PolicyModeProfile {
                allow_override: false,
                max_page_size: 100,
                max_region_span: 10_000_000,
                max_response_bytes: 1_048_576,
            },
            compat: PolicyModeProfile {
                allow_override: true,
                max_page_size: 200,
                max_region_span: 25_000_000,
                max_response_bytes: 2_097_152,
            },
            dev: PolicyModeProfile {
                allow_override: false,
                max_page_size: 500,
                max_region_span: 50_000_000,
                max_response_bytes: 4_194_304,
            },
        },
        query_budget: QueryBudgetPolicy {
            cheap: EndpointClassBudget {
                max_limit: 100,
                max_region_span: 1_000_000,
                max_region_estimated_rows: 10_000,
                max_prefix_cost_units: 25_000,
            },
            medium: EndpointClassBudget {
                max_limit: 100,
                max_region_span: 5_000_000,
                max_region_estimated_rows: 50_000,
                max_prefix_cost_units: 80_000,
            },
            heavy: EndpointClassBudget {
                max_limit: 200,
                max_region_span: 10_000_000,
                max_region_estimated_rows: 100_000,
                max_prefix_cost_units: 120_000,
            },
            max_limit: 100,
            max_transcript_limit: 100,
            heavy_projection_limit: 200,
            max_prefix_length: 128,
            max_sequence_bases: 20_000,
            sequence_api_key_required_bases: 5_000,
        },
        response_budget: ResponseBudgetPolicy {
            cheap_max_bytes: 262_144,
            medium_max_bytes: 524_288,
            heavy_max_bytes: 1_048_576,
            max_serialization_bytes: 524_288,
        },
        cache_budget: CacheBudget {
            max_disk_bytes: 8 * 1024 * 1024 * 1024,
            max_dataset_count: 8,
            pinned_datasets_max: 32,
            shard_count_policy_max: 512,
            max_open_shards_per_pod: 16,
        },
        store_resilience: StoreResiliencePolicy {
            retry_budget: 20,
            retry_attempts: 4,
            retry_base_backoff_ms: 120,
            breaker_failure_threshold: 5,
            breaker_open_ms: 20_000,
        },
        rate_limit: RateLimitPolicy {
            per_ip_rps: 100,
            per_api_key_rps: 500,
            sequence_per_ip_rps: 15,
        },
        concurrency_bulkheads: ConcurrencyBulkheads {
            cheap: 128,
            medium: 64,
            heavy: 16,
        },
        telemetry: TelemetryPolicy {
            metrics_enabled: true,
            tracing_enabled: true,
            slow_query_log_enabled: true,
            request_id_required: true,
            required_metric_labels: vec![
                "subsystem".to_string(),
                "version".to_string(),
                "dataset".to_string(),
            ],
            trace_sampling_per_10k: 100,
        },
        publish_gates: PublishGates {
            required_indexes: vec!["idx_gene_summary_gene_id".to_string()],
            min_gene_count: 1,
            max_missing_parents: 1000,
        },
        documented_defaults: vec![DocumentedDefault {
            field: "query_budget.max_limit".to_string(),
            reason: "default page-size guard".to_string(),
        }],
    }
}

#[test]
fn policy_fields_are_table_validated() {
    let mut cases: Vec<(&str, PolicyConfig)> = Vec::new();

    let mut bad = valid_policy();
    bad.query_budget.max_limit = 0;
    cases.push(("query_budget.max_limit", bad));

    let mut bad = valid_policy();
    bad.cache_budget.max_disk_bytes = 0;
    cases.push(("cache_budget.max_disk_bytes", bad));

    let mut bad = valid_policy();
    bad.telemetry.metrics_enabled = false;
    cases.push(("telemetry.metrics_enabled", bad));

    let mut bad = valid_policy();
    bad.documented_defaults = vec![DocumentedDefault {
        field: "not_a_real.path".to_string(),
        reason: "invalid".to_string(),
    }];
    cases.push(("documented_defaults.field_unknown", bad));

    for (name, cfg) in cases {
        let result = validate_policy_config(&cfg);
        assert!(result.is_err(), "expected validation error for {name}");
    }
}

#[test]
fn schema_version_bump_rules_are_enforced() {
    assert!(validate_schema_version_transition("1", "1").is_ok());
    assert!(validate_schema_version_transition("1", "2").is_ok());

    assert!(validate_schema_version_transition("2", "1").is_err());
    assert!(validate_schema_version_transition("1", "3").is_err());
    assert!(validate_schema_version_transition("x", "1").is_err());
    assert!(validate_schema_version_transition("1", "x").is_err());
}

#[test]
fn canonical_config_print_is_stable() {
    let cfg = valid_policy();
    let a = canonical_config_json(&cfg).expect("canonical a");
    let b = canonical_config_json(&cfg).expect("canonical b");
    assert_eq!(a, b);
}

#[test]
fn workspace_policy_loads_from_ssot_paths() {
    let root = workspace_root();
    let cfg = load_policy_from_workspace(&root).expect("load policy config");
    assert_eq!(cfg.schema_version, PolicySchemaVersion::V1);
}

#[test]
fn policy_change_requires_version_bump() {
    let old = valid_policy();
    let mut changed = valid_policy();
    changed.query_budget.max_limit += 1;
    assert!(validate_policy_change_requires_version_bump(&old, &changed).is_err());
    let bumped = changed.clone();
    assert!(validate_policy_change_requires_version_bump(&changed, &bumped).is_ok());
}

#[test]
fn policy_compatibility_matrix_is_valid() {
    let matrix = [("1", "1", true), ("1", "2", true), ("2", "1", false)];
    for (from, to, ok) in matrix {
        assert_eq!(validate_schema_version_transition(from, to).is_ok(), ok);
    }
}

#[test]
fn policies_crate_dependency_minimalism_no_tokio_axum() {
    let cargo = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("read Cargo.toml");
    for forbidden in ["tokio", "axum"] {
        assert!(
            !cargo.contains(forbidden),
            "forbidden dependency in policies crate: {forbidden}"
        );
    }
}

#[test]
fn runtime_policy_crate_must_not_include_governance_policy_symbols() {
    let src_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut stack = vec![src_root];
    let forbidden = [
        "RepoPolicy",
        "OpsPolicy",
        "DevAtlasPolicySet",
        "ops/inventory/policies",
        "dev-atlas-policy",
    ];

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
            for item in forbidden {
                assert!(
                    !text.contains(item),
                    "runtime policies crate must not include governance symbol `{item}` in {}",
                    path.display()
                );
            }
        }
    }
}
