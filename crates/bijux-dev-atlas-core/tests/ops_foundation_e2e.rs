// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

use bijux_dev_atlas_core::{
    run_checks, Capabilities, Fs, ProcessRunner, RunOptions, RunRequest, Selectors,
};
use bijux_dev_atlas_model::SuiteId;

struct TestFs;
impl Fs for TestFs {
    fn read_text(
        &self,
        repo_root: &Path,
        path: &Path,
    ) -> Result<String, bijux_dev_atlas_core::ports::AdapterError> {
        let target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };
        fs::read_to_string(target).map_err(|err| bijux_dev_atlas_core::ports::AdapterError::Io {
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
    ) -> Result<PathBuf, bijux_dev_atlas_core::ports::AdapterError> {
        let target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };
        target
            .canonicalize()
            .map_err(|err| bijux_dev_atlas_core::ports::AdapterError::Io {
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
    ) -> Result<i32, bijux_dev_atlas_core::ports::AdapterError> {
        Err(bijux_dev_atlas_core::ports::AdapterError::EffectDenied {
            effect: "subprocess",
            detail: format!("attempted to execute `{program}`"),
        })
    }
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(path, content).expect("write");
}

#[test]
fn ops_foundation_suite_passes_on_minimal_fixture() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    write(
        &root.join("ops/inventory/registry.toml"),
        r#"schema_version = 1

[tags]
vocabulary = ["ops", "fast"]

[[checks]]
id = "checks_ops_tree_contract"
domain = "ops"
title = "ops contract files are present"
docs = "ops/CONTRACT.md"
tags = ["ops", "fast"]
suites = ["ops_fast"]
effects_required = ["fs_read"]
budget_ms = 1000
visibility = "public"

[[checks]]
id = "checks_ops_schema_presence"
domain = "ops"
title = "ops schema baseline is present"
docs = "ops/schema/README.md"
tags = ["ops", "fast"]
suites = ["ops_fast"]
effects_required = ["fs_read"]
budget_ms = 1000
visibility = "public"

[[checks]]
id = "checks_ops_manifest_integrity"
domain = "ops"
title = "ops inventory manifests are valid json with required keys"
docs = "ops/inventory/README.md"
tags = ["ops", "fast"]
suites = ["ops_fast"]
effects_required = ["fs_read"]
budget_ms = 1000
visibility = "public"

[[checks]]
id = "checks_ops_surface_inventory"
domain = "ops"
title = "ops index inventories top-level surfaces"
docs = "ops/INDEX.md"
tags = ["ops", "fast"]
suites = ["ops_fast"]
effects_required = ["fs_read"]
budget_ms = 1000
visibility = "public"

[[checks]]
id = "checks_ops_surface_manifest"
domain = "ops"
title = "ops surface manifest consistency"
docs = "ops/CONTRACT.md"
tags = ["ops", "fast"]
suites = ["ops_fast"]
effects_required = ["fs_read"]
budget_ms = 1000
visibility = "public"

[[checks]]
id = "checks_ops_generated_readonly_markers"
domain = "ops"
title = "ops generated files keep readonly generator markers"
docs = "ops/_generated.example/MIRROR_POLICY.md"
tags = ["ops", "fast"]
suites = ["ops_fast"]
effects_required = ["fs_read"]
budget_ms = 1000
visibility = "public"

[[checks]]
id = "checks_ops_artifacts_not_tracked"
domain = "ops"
title = "ops evidence paths stay empty in repository"
docs = "ops/CONTRACT.md"
tags = ["ops", "fast"]
suites = ["ops_fast"]
effects_required = ["fs_read"]
budget_ms = 1000
visibility = "public"

[[suites]]
id = "ops_fast"
checks = []
domains = ["ops"]
tags_any = ["fast"]
"#,
    );

    write(&root.join("ops/CONTRACT.md"), "# Contract\n");
    write(&root.join("ops/ERRORS.md"), "# Errors\n");
    write(
        &root.join("ops/INDEX.md"),
        "# Ops\n- `ops/inventory/`\n- `ops/schema/`\n- `ops/env/`\n- `ops/stack/`\n- `ops/k8s/`\n- `ops/observe/`\n- `ops/load/`\n- `ops/datasets/`\n- `ops/e2e/`\n- `ops/report/`\n- `ops/_generated/`\n- `ops/_generated.example/`\nSchema policy: [Versioning Policy](ops/schema/VERSIONING_POLICY.md)\n",
    );
    write(&root.join("ops/README.md"), "# Ops\n");
    write(&root.join("ops/inventory/OWNER.md"), "# Owner\n");
    write(
        &root.join("ops/inventory/REQUIRED_FILES.md"),
        "# Required Files\n",
    );
    write(
        &root.join("ops/schema/README.md"),
        "# Schema\n- `ops/schema/VERSIONING_POLICY.md`\n- `ops/schema/BUDGET_POLICY.md`\n- `ops/schema/SCHEMA_BUDGET_EXCEPTIONS.md`\n- `ops/schema/SCHEMA_REFERENCE_ALLOWLIST.md`\n",
    );
    write(
        &root.join("ops/schema/VERSIONING_POLICY.md"),
        "# Versioning Policy\n",
    );
    write(
        &root.join("ops/schema/BUDGET_POLICY.md"),
        "# Budget Policy\n",
    );
    write(
        &root.join("ops/schema/SCHEMA_BUDGET_EXCEPTIONS.md"),
        "# Schema Budget Exceptions\n",
    );
    write(
        &root.join("ops/schema/SCHEMA_REFERENCE_ALLOWLIST.md"),
        "# Schema Reference Allowlist\n\
- `ops/schema/configs/public-surface.schema.json`: minimal fixture does not include configs consumers\n\
- `ops/schema/datasets/dataset-index.schema.json`: minimal fixture does not include dataset inventory consumers\n\
- `ops/schema/datasets/dataset-lineage.schema.json`: minimal fixture does not include lineage consumers\n\
- `ops/schema/datasets/fixture-inventory.schema.json`: minimal fixture does not include fixture inventory consumers\n\
- `ops/schema/datasets/manifest.schema.json`: minimal fixture keeps schema presence checks isolated\n\
- `ops/schema/datasets/promotion-rules.schema.json`: minimal fixture omits promotion workflows\n\
- `ops/schema/datasets/qc-metadata.schema.json`: minimal fixture omits qc workflows\n\
- `ops/schema/datasets/rollback-policy.schema.json`: minimal fixture omits rollback workflows\n\
- `ops/schema/e2e/coverage-matrix.schema.json`: minimal fixture omits e2e coverage generator\n\
- `ops/schema/e2e/expectations.schema.json`: minimal fixture omits e2e expectations consumers\n\
- `ops/schema/env/overlay.schema.json`: minimal fixture keeps env overlays inline for contract smoke coverage\n\
- `ops/schema/inventory/gates.schema.json`: minimal fixture omits gate config consumers\n\
- `ops/schema/inventory/pin-freeze.schema.json`: minimal fixture omits pin freeze consumers\n\
- `ops/schema/inventory/pins.schema.json`: minimal fixture omits pin registry consumers\n\
- `ops/schema/inventory/toolchain.schema.json`: minimal fixture omits toolchain registry consumers\n\
- `ops/schema/load/deterministic-seed-policy.schema.json`: minimal fixture omits load seed policies\n\
- `ops/schema/load/k6-suite.schema.json`: minimal fixture omits k6 suite configs\n\
- `ops/schema/load/perf-baseline.schema.json`: minimal fixture omits perf baseline consumers\n\
- `ops/schema/load/thresholds.schema.json`: minimal fixture omits threshold configs\n\
- `ops/schema/meta/inventory-index.schema.json`: minimal fixture omits generated inventory index consumers\n\
- `ops/schema/meta/namespaces.schema.json`: minimal fixture omits namespace registry consumers\n\
- `ops/schema/meta/ops-index.schema.json`: minimal fixture omits ops index generator\n\
- `ops/schema/meta/ownership.schema.json`: minimal fixture omits ownership validators\n\
- `ops/schema/meta/pins.schema.json`: minimal fixture omits pins metadata consumers\n\
- `ops/schema/meta/required-files-contract.schema.json`: minimal fixture validates presence only\n\
- `ops/schema/meta/scorecard.schema.json`: minimal fixture omits scorecard generator\n\
- `ops/schema/report/evidence-levels.schema.json`: minimal fixture omits report evidence generators\n\
- `ops/schema/report/readiness-score.schema.json`: minimal fixture omits readiness score generator\n\
- `ops/schema/report/unified.schema.json`: minimal fixture omits unified report generator\n\
- `ops/schema/stack/artifact-metadata.schema.json`: minimal fixture omits stack artifact generator\n\
- `ops/schema/stack/dependency-graph.schema.json`: minimal fixture omits stack graph generator\n\
- `ops/schema/stack/profile-manifest.schema.json`: minimal fixture omits stack profile generator\n",
    );
    write(&root.join("ops/schema/OWNER.md"), "# Owner\n");
    write(
        &root.join("ops/schema/REQUIRED_FILES.md"),
        "# Required Files\n",
    );
    write(&root.join("ops/schema/meta/ownership.schema.json"), "{}\n");
    write(
        &root.join("ops/schema/meta/required-files-contract.schema.json"),
        "{\"required\":[\"schema_version\",\"required_files\",\"required_dirs\",\"forbidden_patterns\",\"notes\"],\"properties\":{\"schema_version\":{},\"required_files\":{},\"required_dirs\":{},\"forbidden_patterns\":{},\"notes\":{}}}\n",
    );
    write(
        &root.join("ops/schema/inventory/pins.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/inventory/toolchain.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/datasets/manifest.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/datasets/dataset-index.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/datasets/dataset-lineage.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/datasets/fixture-inventory.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/datasets/promotion-rules.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/datasets/qc-metadata.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/datasets/rollback-policy.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/e2e/coverage-matrix.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/e2e/expectations.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/inventory/gates.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/inventory/pin-freeze.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/load/perf-baseline.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/load/deterministic-seed-policy.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/load/k6-suite.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/load/thresholds.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/configs/public-surface.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/env/overlay.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/report/unified.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/meta/namespaces.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/meta/pins.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/meta/inventory-index.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/meta/ops-index.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/meta/scorecard.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/report/readiness-score.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/report/evidence-levels.schema.json"),
        "{\"required\":[\"schema_version\"],\"properties\":{\"schema_version\":{}}}\n",
    );
    write(
        &root.join("ops/schema/stack/profile-manifest.schema.json"),
        "{}\n",
    );
    write(
        &root.join("ops/schema/stack/artifact-metadata.schema.json"),
        "{}\n",
    );
    write(
        &root.join("ops/schema/stack/dependency-graph.schema.json"),
        "{}\n",
    );
    write(
        &root.join("ops/schema/generated/schema-index.json"),
        "{\"schema_version\":1,\"source\":\"ops/schema\",\"files\":[\"ops/schema/configs/public-surface.schema.json\",\"ops/schema/datasets/dataset-index.schema.json\",\"ops/schema/datasets/dataset-lineage.schema.json\",\"ops/schema/datasets/fixture-inventory.schema.json\",\"ops/schema/datasets/manifest.schema.json\",\"ops/schema/datasets/promotion-rules.schema.json\",\"ops/schema/datasets/qc-metadata.schema.json\",\"ops/schema/datasets/rollback-policy.schema.json\",\"ops/schema/e2e/coverage-matrix.schema.json\",\"ops/schema/e2e/expectations.schema.json\",\"ops/schema/env/overlay.schema.json\",\"ops/schema/inventory/gates.schema.json\",\"ops/schema/inventory/pin-freeze.schema.json\",\"ops/schema/inventory/pins.schema.json\",\"ops/schema/inventory/toolchain.schema.json\",\"ops/schema/load/deterministic-seed-policy.schema.json\",\"ops/schema/load/k6-suite.schema.json\",\"ops/schema/load/perf-baseline.schema.json\",\"ops/schema/load/thresholds.schema.json\",\"ops/schema/meta/inventory-index.schema.json\",\"ops/schema/meta/namespaces.schema.json\",\"ops/schema/meta/ops-index.schema.json\",\"ops/schema/meta/ownership.schema.json\",\"ops/schema/meta/pins.schema.json\",\"ops/schema/meta/required-files-contract.schema.json\",\"ops/schema/meta/scorecard.schema.json\",\"ops/schema/report/evidence-levels.schema.json\",\"ops/schema/report/readiness-score.schema.json\",\"ops/schema/report/unified.schema.json\",\"ops/schema/stack/artifact-metadata.schema.json\",\"ops/schema/stack/dependency-graph.schema.json\",\"ops/schema/stack/profile-manifest.schema.json\"]}\n",
    );
    write(
        &root.join("ops/schema/generated/schema-index.md"),
        "# Schema Index\n| Schema | Notes |\n| --- | --- |\n| `ops/schema/configs/public-surface.schema.json` | fixture |\n| `ops/schema/datasets/dataset-index.schema.json` | fixture |\n| `ops/schema/datasets/dataset-lineage.schema.json` | fixture |\n| `ops/schema/datasets/fixture-inventory.schema.json` | fixture |\n| `ops/schema/datasets/manifest.schema.json` | fixture |\n| `ops/schema/datasets/promotion-rules.schema.json` | fixture |\n| `ops/schema/datasets/qc-metadata.schema.json` | fixture |\n| `ops/schema/datasets/rollback-policy.schema.json` | fixture |\n| `ops/schema/e2e/coverage-matrix.schema.json` | fixture |\n| `ops/schema/e2e/expectations.schema.json` | fixture |\n| `ops/schema/env/overlay.schema.json` | fixture |\n| `ops/schema/inventory/gates.schema.json` | fixture |\n| `ops/schema/inventory/pin-freeze.schema.json` | fixture |\n| `ops/schema/inventory/pins.schema.json` | fixture |\n| `ops/schema/inventory/toolchain.schema.json` | fixture |\n| `ops/schema/load/deterministic-seed-policy.schema.json` | fixture |\n| `ops/schema/load/k6-suite.schema.json` | fixture |\n| `ops/schema/load/perf-baseline.schema.json` | fixture |\n| `ops/schema/load/thresholds.schema.json` | fixture |\n| `ops/schema/meta/inventory-index.schema.json` | fixture |\n| `ops/schema/meta/namespaces.schema.json` | fixture |\n| `ops/schema/meta/ops-index.schema.json` | fixture |\n| `ops/schema/meta/ownership.schema.json` | fixture |\n| `ops/schema/meta/pins.schema.json` | fixture |\n| `ops/schema/meta/required-files-contract.schema.json` | fixture |\n| `ops/schema/meta/scorecard.schema.json` | fixture |\n| `ops/schema/report/evidence-levels.schema.json` | fixture |\n| `ops/schema/report/readiness-score.schema.json` | fixture |\n| `ops/schema/report/unified.schema.json` | fixture |\n| `ops/schema/stack/artifact-metadata.schema.json` | fixture |\n| `ops/schema/stack/dependency-graph.schema.json` | fixture |\n| `ops/schema/stack/profile-manifest.schema.json` | fixture |\n",
    );
    write(
        &root.join("ops/schema/generated/compatibility-lock.json"),
        "{\"schema_version\":1,\"targets\":[{\"schema_path\":\"ops/schema/inventory/pins.schema.json\",\"required_fields\":[]}]}\n",
    );
    write(&root.join("ops/inventory/README.md"), "# Inventory\n");
    write(&root.join("ops/env/README.md"), "# Env\n");
    write(&root.join("ops/env/OWNER.md"), "# Owner\n");
    write(
        &root.join("ops/env/REQUIRED_FILES.md"),
        "# Required Files\n",
    );
    write(
        &root.join("ops/env/base/overlay.json"),
        "{\"schema_version\":1,\"environment\":\"base\",\"values\":{\"namespace\":\"atlas-e2e\",\"cluster_profile\":\"kind\",\"allow_write\":false,\"allow_subprocess\":false,\"network_mode\":\"restricted\"}}\n",
    );
    write(
        &root.join("ops/env/dev/overlay.json"),
        "{\"schema_version\":1,\"environment\":\"dev\",\"values\":{\"allow_write\":true,\"allow_subprocess\":true,\"network_mode\":\"local\"}}\n",
    );
    write(
        &root.join("ops/env/ci/overlay.json"),
        "{\"schema_version\":1,\"environment\":\"ci\",\"values\":{\"allow_write\":false,\"allow_subprocess\":false,\"network_mode\":\"restricted\"}}\n",
    );
    write(
        &root.join("ops/env/prod/overlay.json"),
        "{\"schema_version\":1,\"environment\":\"prod\",\"values\":{\"allow_write\":false,\"allow_subprocess\":true,\"network_mode\":\"restricted\"}}\n",
    );
    write(
        &root.join("ops/inventory/surfaces.json"),
        "{\"schema_version\":1,\"entrypoints\":[]}\n",
    );
    write(
        &root.join("ops/inventory/contracts.json"),
        "{\"schema_version\":1}\n",
    );
    write(
        &root.join("ops/inventory/drills.json"),
        "{\"schema_version\":1}\n",
    );
    write(
        &root.join("ops/inventory/gates.json"),
        "{\"schema_version\":1,\"gates\":[]}\n",
    );
    write(
        &root.join("ops/inventory/generated-committed-mirror.json"),
        "{\"schema_version\":1,\"allow_runtime_compat\":[],\"mirrors\":[{\"committed\":\"ops/_generated.example/.gitkeep\",\"source\":\"ops/_generated/.gitkeep\"},{\"committed\":\"ops/_generated.example/README.md\",\"source\":\"ops/_generated/README.md\"},{\"committed\":\"ops/_generated.example/OWNER.md\",\"source\":\"ops/_generated/OWNER.md\"},{\"committed\":\"ops/_generated.example/REQUIRED_FILES.md\",\"source\":\"ops/_generated/REQUIRED_FILES.md\"}]}\n",
    );
    write(&root.join("ops/_generated.example/.gitkeep"), "\n");
    write(&root.join("ops/_generated/README.md"), "# Generated\n");
    write(&root.join("ops/_generated/OWNER.md"), "# Owner\n");
    write(
        &root.join("ops/_generated/REQUIRED_FILES.md"),
        "# Required Files\n",
    );
    write(
        &root.join("ops/_generated.example/README.md"),
        "# Generated Example\n",
    );
    write(&root.join("ops/_generated.example/OWNER.md"), "# Owner\n");
    write(
        &root.join("ops/_generated.example/REQUIRED_FILES.md"),
        "# Required Files\n",
    );
    write(
        &root.join("ops/stack/generated/stack-index.json"),
        "{\"schema_version\":1}\n",
    );
    write(
        &root.join("ops/stack/generated/dependency-graph.json"),
        "{\"schema_version\":1}\n",
    );
    write(
        &root.join("ops/stack/generated/artifact-metadata.json"),
        "{\"schema_version\":1}\n",
    );
    write(&root.join("ops/stack/README.md"), "# Stack\n");
    write(&root.join("ops/stack/OWNER.md"), "# Owner\n");
    write(
        &root.join("ops/stack/REQUIRED_FILES.md"),
        "# Required Files\n",
    );
    write(&root.join("ops/k8s/README.md"), "# K8s\n");
    write(&root.join("ops/k8s/OWNER.md"), "# Owner\n");
    write(
        &root.join("ops/k8s/REQUIRED_FILES.md"),
        "# Required Files\n",
    );
    write(&root.join("ops/observe/README.md"), "# Observe\n");
    write(&root.join("ops/observe/OWNER.md"), "# Owner\n");
    write(
        &root.join("ops/observe/REQUIRED_FILES.md"),
        "# Required Files\n",
    );
    write(&root.join("ops/load/README.md"), "# Load\n");
    write(&root.join("ops/load/OWNER.md"), "# Owner\n");
    write(
        &root.join("ops/load/REQUIRED_FILES.md"),
        "# Required Files\n",
    );
    write(&root.join("ops/e2e/README.md"), "# E2E\n");
    write(&root.join("ops/e2e/OWNER.md"), "# Owner\n");
    write(
        &root.join("ops/e2e/REQUIRED_FILES.md"),
        "# Required Files\n",
    );
    write(&root.join("ops/datasets/README.md"), "# Datasets\n");
    write(&root.join("ops/datasets/OWNER.md"), "# Owner\n");
    write(
        &root.join("ops/datasets/REQUIRED_FILES.md"),
        "# Required Files\n",
    );
    write(&root.join("ops/report/README.md"), "# Report\n");
    write(&root.join("ops/report/OWNER.md"), "# Owner\n");
    write(
        &root.join("ops/report/REQUIRED_FILES.md"),
        "# Required Files\n",
    );
    write(
        &root.join("configs/ops/ops-surface-manifest.json"),
        "{\"schema_version\":1}\n",
    );

    let request = RunRequest {
        repo_root: root.to_path_buf(),
        domain: None,
        capabilities: Capabilities::deny_all(),
        artifacts_root: None,
        run_id: None,
        command: None,
    };
    let selectors = Selectors {
        suite: Some(SuiteId::parse("ops_fast").expect("suite")),
        include_internal: true,
        include_slow: true,
        ..Selectors::default()
    };
    let report = run_checks(
        &DeniedProcessRunner,
        &TestFs,
        &request,
        &selectors,
        &RunOptions::default(),
    )
    .expect("run");

    assert_eq!(report.summary.failed, 0);
    assert_eq!(report.summary.errors, 0);
    assert_eq!(report.summary.total, 7);
}
