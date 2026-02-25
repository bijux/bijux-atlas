# Docs Front Matter Contract

## Purpose
- Owner: `docs-governance`
- Stability: `stable`

Define required documentation metadata fields and where they are sourced.

## Stability
- v0.1.0 stable scope.
- Canonical schema source: `docs/metadata/schema.json`.
- Canonical inventory source: `docs/metadata/front-matter.index.json`.

## Guarantees
- Every canonical documentation page must resolve to one owner.
- Registry validation fails when ownership metadata is missing.
- Front matter metadata remains machine-readable for CI checks.
