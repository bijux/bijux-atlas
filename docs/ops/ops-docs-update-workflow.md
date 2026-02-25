# Ops Docs Update Workflow

- Owner: bijux-atlas-operations
- Stability: stable

## Workflow

1. Update canonical source files under `ops/` (inventory, contracts, schemas, or generated artifacts).
2. Update impacted docs in `docs/ops/` and domain `README.md`/`INDEX.md` files.
3. Verify command examples against `ops/inventory/surfaces.json`.
4. Regenerate curated docs drift evidence: `ops/_generated.example/docs-drift-report.json`.
5. Confirm `docs/ops/INDEX.md` links every report doc.
6. Run docs governance checks before merging.

## Guardrails

- `docs/ops/` may not contain orphan files.
- `TODO`/`TBD` markers are forbidden in release docs.
- Removed or retired legacy observability aliases must not appear in docs.
- `ops/INDEX.md` is the root navigation page for top-level ops docs.
