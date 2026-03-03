# Ops Generated Lifecycle

- Owner: `bijux-atlas-operations`
- Purpose: define lifecycle and retention policy for generated ops artifacts.
- Consumers: `checks_ops_evidence_bundle_discipline`

## Lifecycle Classes

- transient_runtime
- domain_derived
- curated_evidence

## Retention Policy

- transient_runtime artifacts are disposable and regenerated per run.
- curated_evidence artifacts are committed and schema-validated.

## Regeneration Triggers

- contract or schema edits
- inventory surface changes

## Deterministic Ordering

- generated lists are sorted lexicographically.

## Generator Contract Versioning

- generated artifacts include `schema_version` and `generated_by`.
