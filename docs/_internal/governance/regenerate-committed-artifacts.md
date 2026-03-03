# Regenerate Committed Docs Artifacts

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@f8bf6639b45f6717d0de1903b56bc7dcf4615b06`
- Reason to exist: define the single canonical command flow for refreshing committed documentation artifacts.

## Canonical Command

Use `bijux-dev-atlas docs ...` commands to regenerate committed documentation artifacts.

## Standard Flow

1. Run the required docs generator command, such as `bijux-dev-atlas docs redirects sync --allow-write` or
   `bijux-dev-atlas docs health-dashboard --allow-write`.
2. If reference surfaces changed, run `bijux-dev-atlas docs reference generate --allow-subprocess --allow-write`.
3. Run `mkdocs build --strict`.
4. Review only the committed generated markdown and governance surfaces that changed.

Do not edit committed generated markdown directly unless the governing policy explicitly says a fixture update is
required.
