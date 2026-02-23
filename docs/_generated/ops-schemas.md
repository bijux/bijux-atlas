# Ops Schemas

Generated from `ops/schema`. Do not edit manually.

## `ops/schema/datasets/corruption-drill-report.schema.json`

Required keys:
- `schema_version`
- `timestamp_utc`
- `status`
- `target_manifest`
- `quarantine_marker`

## `ops/schema/datasets/manifest-lock.schema.json`

Required keys:
- `schema_version`
- `entries`

## `ops/schema/datasets/promotion-report.schema.json`

Required keys:
- `schema_version`
- `run_id`
- `timestamp_utc`
- `source_catalog`
- `environments`
- `promoted_count`

## `ops/schema/e2e-realdata-scenarios.schema.json`

Required keys:
- `schema_version`
- `scenarios`

## `ops/schema/e2e-scenarios.schema.json`

Required keys:
- `schema_version`
- `scenarios`

## `ops/schema/e2e-suites.schema.json`

Required keys:
- `schema_version`
- `suites`

## `ops/schema/k8s/install-matrix.schema.json`

Required keys:
- `schema_version`
- `profiles`

## `ops/schema/k8s/suite-report.schema.json`

Required keys:
- `schema_version`
- `run_id`
- `suite_id`
- `total`
- `failed`
- `passed`
- `results`

## `ops/schema/load/perf-baseline.schema.json`

Required keys:
- `schema_version`
- `name`
- `metadata`
- `rows`

## `ops/schema/load/pinned-queries-lock.schema.json`

Required keys:
- `file_sha256`
- `query_hashes`

## `ops/schema/load/suite-manifest.schema.json`

Required keys:
- `schema_version`
- `query_set`
- `scenarios_dir`
- `suites`

## `ops/schema/meta/artifact-allowlist.schema.json`

Required keys:
- `entries`

## `ops/schema/meta/budgets.schema.json`

Required keys:
- `schema_version`
- `smoke`
- `root_local`
- `k6_latency`
- `cold_start`
- `cache`
- `metric_cardinality`

## `ops/schema/meta/layer-contract.schema.json`

Required keys:
- `contract_version`
- `compatibility`
- `layer_dependencies`
- `ssot`
- `namespaces`
- `services`
- `ports`
- `labels`
- `release_metadata`

## `ops/schema/meta/namespaces.schema.json`

Required keys:
- `schema_version`
- `namespaces`

## `ops/schema/meta/ownership.schema.json`

Required keys:
- `schema_version`
- `areas`

## `ops/schema/meta/pins.schema.json`

Required keys:
- `schema_version`
- `contract_version`
- `tools`
- `images`
- `helm`
- `datasets`
- `policy`

## `ops/schema/meta/ports.schema.json`

Required keys:
- `schema_version`
- `ports`

## `ops/schema/obs/budgets.schema.json`

Required keys:
- `schema_version`
- `cardinality`
- `required_metric_labels`
- `endpoint_class_metric_requirements`
- `span_attribute_requirements`
- `lag`

## `ops/schema/obs/drill-manifest.schema.json`

Required keys:
- `schema_version`
- `drills`

## `ops/schema/obs/suites.schema.json`

Required keys:
- `schema_version`
- `owner`
- `suites`

## `ops/schema/report/lane.schema.json`

Required keys:
- `schema_version`
- `report_version`
- `lane`
- `run_id`
- `status`
- `started_at`
- `ended_at`
- `duration_seconds`
- `log`
- `artifact_paths`

## `ops/schema/report/schema.json`

Required keys:
- `run_id`
- `namespace`
- `metadata`
- `artifacts`

## `ops/schema/report/stack-contract.schema.json`

Required keys:
- `schema_version`
- `stack_version_hash`
- `status`
- `run_id`
- `generated_at_utc`
- `artifacts`

## `ops/schema/report/unified.schema.json`

Required keys:
- `schema_version`
- `report_version`
- `run_id`
- `generated_at`
- `lanes`
- `summary`
- `budget_status`

## `ops/schema/stack/profile-manifest.schema.json`

Required keys:
- `schema_version`
- `profiles`

## `ops/schema/stack/version-manifest.schema.json`

Required keys:
- `kind_node_image`
- `minio`
- `minio_mc`
- `prometheus`
- `otel_collector`
- `redis`
- `toxiproxy`

