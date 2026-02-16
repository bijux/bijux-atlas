use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use bijux_atlas_policies::{
    canonical_config_json, load_policy_from_workspace, validate_policy_config,
    validate_schema_version_transition, CacheBudget, ConcurrencyBulkheads, PolicyConfig,
    QueryBudget, RateLimitPolicy, TelemetryPolicy, MAX_DEPTH_HARD, MAX_LOC_HARD,
    MAX_MODULES_PER_DIR_HARD, MAX_RS_FILES_PER_DIR_HARD,
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

fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !dir.exists() {
        return out;
    }
    for entry in fs::read_dir(dir).expect("read_dir failed") {
        let entry = entry.expect("dir entry failed");
        let path = entry.path();
        if path.is_dir() {
            out.extend(collect_rs_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
    out
}

fn valid_policy() -> PolicyConfig {
    PolicyConfig {
        schema_version: "1".to_string(),
        allow_override: false,
        network_in_unit_tests: false,
        query_budget: QueryBudget {
            max_limit: 100,
            max_region_span: 10_000_000,
            max_region_estimated_rows: 50_000,
            max_prefix_cost_units: 80_000,
            heavy_projection_limit: 200,
            max_serialization_bytes: 524_288,
            max_prefix_length: 128,
        },
        cache_budget: CacheBudget {
            max_disk_bytes: 8 * 1024 * 1024 * 1024,
            max_dataset_count: 8,
            pinned_datasets_max: 32,
        },
        rate_limit: RateLimitPolicy {
            per_ip_rps: 100,
            per_api_key_rps: 500,
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
        },
        documented_defaults: Vec::new(),
    }
}

#[test]
fn max_loc_per_rust_file_is_enforced() {
    let root = workspace_root();
    let files = collect_rs_files(&root.join("crates"));
    let allowlist: [&str; 1] = ["crates/bijux-atlas-server/src/lib.rs"];

    let mut violators = Vec::new();
    for file in files {
        let lines = fs::read_to_string(&file)
            .expect("failed to read rust file")
            .lines()
            .count();
        if lines > MAX_LOC_HARD {
            let rel = file
                .strip_prefix(&root)
                .expect("path must be under workspace root")
                .to_string_lossy()
                .to_string();
            if !allowlist.contains(&rel.as_str()) {
                violators.push((lines, file));
            }
        }
    }

    assert!(
        violators.is_empty(),
        "max_loc policy violations (> {} lines): {:?}",
        MAX_LOC_HARD,
        violators
    );
}

#[test]
fn max_path_depth_for_rust_files_is_enforced() {
    let root = workspace_root();
    let files = collect_rs_files(&root.join("crates"));

    let mut violators = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("path must be under workspace root");
        let depth = rel.components().count();
        if depth > MAX_DEPTH_HARD {
            violators.push((depth, rel.to_path_buf()));
        }
    }

    assert!(
        violators.is_empty(),
        "max_depth policy violations (> {} components): {:?}",
        MAX_DEPTH_HARD,
        violators
    );
}

#[test]
fn max_rs_files_per_directory_is_enforced() {
    let root = workspace_root();
    let files = collect_rs_files(&root.join("crates"));

    let mut counts: BTreeMap<PathBuf, usize> = BTreeMap::new();
    for file in files {
        let dir = file
            .parent()
            .expect("rust file must have parent")
            .strip_prefix(&root)
            .expect("parent must be under root")
            .to_path_buf();
        *counts.entry(dir).or_insert(0) += 1;
    }

    let violators: Vec<_> = counts
        .into_iter()
        .filter(|(_, count)| *count > MAX_RS_FILES_PER_DIR_HARD)
        .collect();

    assert!(
        violators.is_empty(),
        "max_rs_files_per_dir policy violations (> {}): {:?}",
        MAX_RS_FILES_PER_DIR_HARD,
        violators
    );
}

#[test]
fn max_modules_per_directory_is_enforced() {
    let root = workspace_root();
    let files = collect_rs_files(&root.join("crates"));

    let mut counts: BTreeMap<PathBuf, usize> = BTreeMap::new();
    for file in files {
        let dir = file
            .parent()
            .expect("rust file must have parent")
            .strip_prefix(&root)
            .expect("parent must be under root")
            .to_path_buf();
        *counts.entry(dir).or_insert(0) += 1;
    }

    let violators: Vec<_> = counts
        .into_iter()
        .filter(|(_, count)| *count > MAX_MODULES_PER_DIR_HARD)
        .collect();

    assert!(
        violators.is_empty(),
        "max_modules_per_dir policy violations (> {}): {:?}",
        MAX_MODULES_PER_DIR_HARD,
        violators
    );
}

#[test]
fn policy_fields_are_table_validated() {
    let mut cases: Vec<(&str, PolicyConfig)> = Vec::new();

    let mut bad = valid_policy();
    bad.query_budget.max_limit = 0;
    cases.push(("query_budget.max_limit", bad));

    let mut bad = valid_policy();
    bad.query_budget.max_region_span = 0;
    cases.push(("query_budget.max_region_span", bad));

    let mut bad = valid_policy();
    bad.query_budget.max_prefix_length = 0;
    cases.push(("query_budget.max_prefix_length", bad));

    let mut bad = valid_policy();
    bad.query_budget.max_region_estimated_rows = 0;
    cases.push(("query_budget.max_region_estimated_rows", bad));

    let mut bad = valid_policy();
    bad.query_budget.max_prefix_cost_units = 0;
    cases.push(("query_budget.max_prefix_cost_units", bad));

    let mut bad = valid_policy();
    bad.query_budget.heavy_projection_limit = 0;
    cases.push(("query_budget.heavy_projection_limit", bad));

    let mut bad = valid_policy();
    bad.query_budget.max_serialization_bytes = 0;
    cases.push(("query_budget.max_serialization_bytes", bad));

    let mut bad = valid_policy();
    bad.cache_budget.max_disk_bytes = 0;
    cases.push(("cache_budget.max_disk_bytes", bad));

    let mut bad = valid_policy();
    bad.cache_budget.max_dataset_count = 0;
    cases.push(("cache_budget.max_dataset_count", bad));

    let mut bad = valid_policy();
    bad.rate_limit.per_ip_rps = 0;
    cases.push(("rate_limit.per_ip_rps", bad));

    let mut bad = valid_policy();
    bad.rate_limit.per_api_key_rps = 0;
    cases.push(("rate_limit.per_api_key_rps", bad));

    let mut bad = valid_policy();
    bad.concurrency_bulkheads.cheap = 0;
    cases.push(("concurrency_bulkheads.cheap", bad));

    let mut bad = valid_policy();
    bad.concurrency_bulkheads.medium = 0;
    cases.push(("concurrency_bulkheads.medium", bad));

    let mut bad = valid_policy();
    bad.concurrency_bulkheads.heavy = 0;
    cases.push(("concurrency_bulkheads.heavy", bad));

    let mut bad = valid_policy();
    bad.telemetry.metrics_enabled = false;
    cases.push(("telemetry.metrics_enabled", bad));

    let mut bad = valid_policy();
    bad.telemetry.tracing_enabled = false;
    cases.push(("telemetry.tracing_enabled", bad));

    let mut bad = valid_policy();
    bad.telemetry.request_id_required = false;
    cases.push(("telemetry.request_id_required", bad));

    let mut bad = valid_policy();
    bad.allow_override = true;
    cases.push(("allow_override", bad));

    let mut bad = valid_policy();
    bad.network_in_unit_tests = true;
    cases.push(("network_in_unit_tests", bad));

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
    assert_eq!(cfg.schema_version, "1");
}

#[test]
fn policies_crate_must_not_depend_on_server_api_or_query() {
    let cargo = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("read Cargo.toml");

    for forbidden in ["bijux-atlas-server", "bijux-atlas-api", "bijux-atlas-query"] {
        assert!(
            !cargo.contains(forbidden),
            "forbidden dependency in policies crate: {forbidden}"
        );
    }
}
