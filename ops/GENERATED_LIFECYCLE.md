# Generated Artifact Lifecycle

## Scope

This contract defines generated artifact classes, retention, and regeneration policy for all `ops/` generated surfaces.

## Lifecycle Classes

- `transient_runtime`: local or CI runtime outputs under `ops/_generated/`; never committed.
- `domain_derived`: deterministic domain outputs under `ops/**/generated/`; committed only when used as contract artifacts.
- `curated_evidence`: committed verification artifacts under `ops/_generated.example/` allowlist.

## Derived-Only Rule

- Every `ops/**/generated/` directory is derived-only.
- Files in derived-only directories must include `generated_by` and `schema_version` when JSON.
- Authored source-of-truth files are forbidden in derived-only directories.

## Retention Policy

- `ops/_generated/`: no retention in git; markers only.
- `ops/**/generated/`: retain only current deterministic contract outputs.
- `ops/_generated.example/`: retain only allowlisted evidence needed for verification and release audits.

## Regeneration Triggers

- Update authored inventory/schema/domain inputs.
- Update generator implementation or output schema contracts.
- Any evidence drift report transition to `fail`.

## Deterministic Ordering

- Generated lists must be stable and lexicographically sorted where applicable.
- Hash indexes must be regenerated in deterministic order.

## Generator Contract Versioning

- Generator-emitted artifacts must carry `schema_version`.
- Changes to generated artifact structure require schema updates and compatibility review.
