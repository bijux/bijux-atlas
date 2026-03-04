# Redirects Contract

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the single redirect mechanism and the source of truth for it.

## Mechanism

This repository uses MkDocs-native redirects via the `mkdocs-redirects` plugin in `mkdocs.yml`.
The source of truth is `docs/redirects.json`.
Redirects in that file are page redirects only: markdown source paths to markdown target paths.
Legacy aliases that no longer exist on disk are governed by `docs/_internal/governance/redirect-legacy-policy.json`.
Redirect targets into `docs/_internal/` are only allowed when they match
`docs/_internal/governance/redirect-internal-target-policy.json`.

## Generation

Run `bijux-dev-atlas docs redirects sync --allow-write` to synchronize the `redirect_maps` block in
`mkdocs.yml` from `docs/redirects.json`. The generator only materializes markdown-to-markdown redirects because
that is the surface `mkdocs-redirects` can render during a normal docs build.

## Validation

- `docs/redirects.json` must contain repository-relative `docs/...` paths.
- `docs/redirects.json` must contain markdown-to-markdown redirects only.
- `mkdocs.yml` must keep the generated markdown redirect block in sync with `docs/redirects.json`.
- Redirects may point at removed historical paths only when the source is explicitly declared legacy.
- Redirects may not point into `docs/_internal/` unless an internal target is explicitly allowlisted.
- `mkdocs build` is expected to emit redirect pages because the redirect plugin is part of the
  canonical docs build path.
