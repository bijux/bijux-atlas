> Redirect Notice: canonical handbook content lives under `docs/operations/` (see `docs/operations/ops-system/INDEX.md`).

# Pin Lifecycle

- Owner: bijux-atlas-operations
- Stability: stable

## States

- `draft`: candidate values prepared for review and validation.
- `promoted`: accepted values ready for release use.
- `frozen`: released values; immutable for that release line.

## Rules

- All pin updates are proposed in `draft` state first.
- Promotion requires passing inventory/schema validation and review.
- Frozen pins are immutable. Corrections must use a new promoted/frozen version set.
- Runtime commands must consume pins from canonical inventory sources only.

## Why

- Prevent silent drift between development and release environments.
- Keep release evidence reproducible.
- Preserve deterministic rollback behavior.
