# Redirects Contract

- Owner: `docs-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@fc2319a8483a4d0c9d08e5227ec31d7cb6677c4a`
- Reason to exist: define the single redirect mechanism and the source of truth for it.

## Mechanism

This repository uses MkDocs-native redirects via the `mkdocs-redirects` plugin in `mkdocs.yml`.
The source of truth is `docs/redirects.json`.

## Generation

Run `scripts/docs/sync_redirects.py` to synchronize the `redirect_maps` block in `mkdocs.yml`
from `docs/redirects.json`. The generator only materializes markdown-to-markdown redirects because
that is the surface `mkdocs-redirects` can render during a normal docs build.

## Validation

- `docs/redirects.json` must contain repository-relative `docs/...` paths.
- `mkdocs.yml` must keep the generated markdown redirect block in sync with `docs/redirects.json`.
- Non-markdown aliases remain generator-owned artifact paths and are not part of `redirect_maps`.
- `mkdocs build` is expected to emit redirect pages because the redirect plugin is part of the
  canonical docs build path.
