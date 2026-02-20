# Ops Schemas

Generated from `ops/_schemas`. Do not edit manually.

## `ops/_schemas/datasets/corruption-drill-report.schema.json`

Required keys:
- `schema_version`
- `timestamp_utc`
- `status`
- `target_manifest`
- `quarantine_marker`

## `ops/_schemas/datasets/manifest-lock.schema.json`

Required keys:
- `schema_version`
- `entries`

## `ops/_schemas/datasets/promotion-report.schema.json`

Required keys:
- `schema_version`
- `run_id`
- `timestamp_utc`
- `source_catalog`
- `environments`
- `promoted_count`

## `ops/_schemas/e2e-realdata-scenarios.schema.json`

Required keys:
- `schema_version`
- `scenarios`

## `ops/_schemas/e2e-scenarios.schema.json`

Required keys:
- `schema_version`
- `scenarios`

## `ops/_schemas/e2e-suites.schema.json`

Required keys:
- `schema_version`
- `suites`

## `ops/_schemas/k8s/install-matrix.schema.json`

Required keys:
- `schema_version`
- `profiles`

## `ops/_schemas/load/perf-baseline.schema.json`

Required keys:
- `schema_version`
- `name`
- `metadata`
- `rows`

## `ops/_schemas/load/pinned-queries-lock.schema.json`

Required keys:
- `file_sha256`
- `query_hashes`

## `ops/_schemas/load/suite-manifest.schema.json`

Required keys:
- `schema_version`
- `query_set`
- `scenarios_dir`
- `suites`

## `ops/_schemas/meta/artifact-allowlist.schema.json`

Required keys:
- `entries`

## `ops/_schemas/meta/budgets.schema.json`

Required keys:
- `schema_version`
- `smoke`
- `root_local`
- `k6_latency`
- `cold_start`
- `cache`
- `metric_cardinality`

## `ops/_schemas/meta/layer-contract.schema.json`

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

## `ops/_schemas/meta/namespaces.schema.json`

Required keys:
- `schema_version`
- `namespaces`

## `ops/_schemas/meta/ownership.schema.json`

Required keys:
- `schema_version`
- `areas`

## `ops/_schemas/meta/pins.schema.json`

Required keys:
- `schema_version`
- `contract_version`
- `tools`
- `images`
- `helm`
- `datasets`
- `policy`

## `ops/_schemas/meta/ports.schema.json`

Required keys:
- `schema_version`
- `ports`

## `ops/_schemas/obs/budgets.schema.json`

Required keys:
- `schema_version`
- `cardinality`
- `required_metric_labels`
- `endpoint_class_metric_requirements`
- `span_attribute_requirements`
- `lag`

## `ops/_schemas/obs/drill-manifest.schema.json`

Required keys:
- `schema_version`
- `drills`

## `ops/_schemas/obs/suites.schema.json`

Required keys:
- `schema_version`
- `owner`
- `suites`

## `ops/_schemas/report/schema.json`

Required keys:
- `run_id`
- `namespace`
- `metadata`
- `artifacts`

## `ops/_schemas/report/stack-contract.schema.json`

Required keys:
- `schema_version`
- `stack_version_hash`
- `status`
- `run_id`
- `generated_at_utc`
- `artifacts`

## `ops/_schemas/report/unified.schema.json`

Required keys:
- `schema_version`
- `run_id`
- `generated_at`
- `lanes`
- `summary`
- `budget_status`

## `ops/_schemas/stack/profile-manifest.schema.json`

Required keys:
- `schema_version`
- `profiles`

## `ops/_schemas/stack/version-manifest.schema.json`

Required keys:
- `kind_node_image`
- `minio`
- `minio_mc`
- `prometheus`
- `otel_collector`
- `redis`
- `toxiproxy`

