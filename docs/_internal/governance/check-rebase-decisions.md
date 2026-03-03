# Check Rebase Decisions

- Owner: `team:atlas-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: record permanent decisions for replacing outdated layout checks.

## Decisions

- Replace ad hoc root-surface checks with authority from `configs/layout/root-markdown-allowlist.json`.
- Replace broad markdown placement bans with `docs/_internal/policies/allowed-non-docs-markdown.json`.
- Replace nested-markdown style bans with class-based enforcement from `docs/_internal/policies/ops-docs-classes.json`.
- Keep docs nav authority in `mkdocs.yml`; all nav targets must resolve in `docs/`.

## Deprecated Patterns

- Checks that fail based only on legacy file layout assumptions.
- Checks that enforce structure without a declared policy authority.

## Replacement Rule

A check that governs repository structure must cite one canonical policy file.
