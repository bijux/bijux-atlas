# Migration Map Split Index

This directory is the canonical replacement for the monolithic `packages/atlasctl/MIGRATION_MAP.md`.

## Domain Files

- `ops-lint.md`: `ops/_lint/*` and lint-policy migrations.
- `ops-runtime.md`: `ops/*` command/report/runtime script migrations.
- `scripts-areas.md`: `scripts/areas/*` migrations.
- `atlasctl-layout-checks.md`: `packages/atlasctl/src/atlasctl/layout_checks/*` migrations.

## Policy

- New entries must be added to one split domain file in this directory.
- `packages/atlasctl/MIGRATION_MAP.md` is transitional and should only be touched for compatibility while the split migration completes.
- Keep table columns identical to the monolith to preserve parser/tool compatibility.
