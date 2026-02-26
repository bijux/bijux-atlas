# Ops Design Defense

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

## What Ops Is

`ops/` is the machine-verifiable contract surface for operations behavior.
It exists to provide stable artifacts that are validated by schemas, registries, and governance checks.

## What Ops Is Not

`ops/` is not the handbook or tutorial surface.
Human-facing workflows, runbooks, and architecture guidance live in `docs/operations/**`.

## Why This Structure Exists

- Prevent duplicate sources of truth across docs and ops.
- Make every ops file defendable by explicit necessity:
  - registry reference,
  - schema validation,
  - generator input/output contract,
  - REQUIRED_FILES contract,
  - explicit asset exception.
- Keep generated runtime output (`ops/_generated/`) minimal and uncommitted.
- Keep curated evidence (`ops/_generated.example/`) allowlisted, policy-governed, and reproducible.

## Enforcement Model

- `ops/inventory/contracts-map.json` is the authoritative contracts map.
- `ops/schema/generated/schema-index.json` is the schema index authority.
- `ops/_generated.example/ops-ledger.json` is the file necessity ledger.
- `checks_ops_*` governance checks enforce drift, ownership, naming, and evidence policy.

## Review Rule

A proposed ops file change is accepted only if it remains necessary under the ledger model and passes governance checks without introducing duplicate narrative content.
