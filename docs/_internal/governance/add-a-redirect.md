# How To Add A Redirect

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the supported redirect workflow.

## Procedure

1. Add the old `docs/...` path and new `docs/...` path to `docs/redirects.json`.
2. Run `bijux-dev-atlas docs redirects sync --allow-write`.
3. Run `bijux-dev-atlas docs nav check`.
4. Run `mkdocs build --strict`.

## Constraints

- Redirects are only rendered through MkDocs for markdown-to-markdown paths.
- Non-markdown aliases may exist in `docs/redirects.json`, but they stay generator-owned and do not
  belong in `redirect_maps`.
- The destination page must already exist before the redirect is added.
