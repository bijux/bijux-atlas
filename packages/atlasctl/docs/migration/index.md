# Migration Map Split Index

This directory is the canonical replacement for the historical monolithic migration map.

## Domain Files

- `ops-lint.md`: `ops/_lint/*` and lint-policy migrations.
- `ops-runtime.md`: `ops/*` command/report/runtime script migrations.
- `scripts-areas.md`: `scripts/areas/*` migrations.
- `layout-checks.md`: `packages/atlasctl/src/atlasctl/layout_checks/*` migrations.

## Policy

- New entries must be added to one split domain file in this directory.
- `docs/migration/map.md` is transitional and should only be touched for compatibility while the split migration completes.
- Keep table columns identical to the monolith to preserve parser/tool compatibility.
- `legacy-removal-plan.md` tracks the pre-1.0 hard-reset removal milestones.
