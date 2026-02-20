# Migration Map: Ops Lint

Scope: `ops/_lint/*` and `ops/_lint/policy/*`.

## Table Contract

Use the same columns as `packages/atlasctl/MIGRATION_MAP.md`:

| Legacy Script | New Module Path | New CLI Command | Output Schema | Tests |
|---|---|---|---|---|

## Notes

- Prefer `atlasctl.lint.*` targets.
- Keep `migration.todo` rows explicit when not yet migrated.
