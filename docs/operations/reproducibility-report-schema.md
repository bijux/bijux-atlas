# Reproducibility Report Schema

`ops/reproducibility/report.schema.json` defines the canonical shape for reproducibility outputs.

Required top-level fields:
- `schema_version`
- `kind`
- `status`

Optional sections used by current commands:
- `environment`
- `artifact_hashes`
- `scenarios`
- `release_manifest_artifact_count`
- `metrics`

Schema versioning policy:
- Increment `schema_version` only for breaking shape changes.
- Additive fields must remain optional.
- Consumers must reject unknown major schema versions.
