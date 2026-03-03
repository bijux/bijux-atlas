# Ops SSOT Boundary

`ops/` is the operational source of truth for machine-governed operational inputs and outputs.

## Canonical Scope

- Inventory and mapping: `ops/inventory/**`
- Schema definitions: `ops/schema/**`
- Operational specifications and runbooks: `ops/**` domain folders
- Curated generated examples: `ops/_generated.example/**` (example-only, never authoritative)

## Boundary Rules

- User-facing narrative guidance stays in `docs/**`.
- Operational evidence is emitted to `artifacts/**`; evidence is not checked in under `ops/**`.
- Markdown under `ops/**` must be classified as `spec`, `runbook`, `policy`, `evidence`, or `stub` per `docs/_internal/policies/ops-docs-classes.json`.
- Any markdown outside `docs/**` must follow `docs/_internal/policies/allowed-non-docs-markdown.json`.
