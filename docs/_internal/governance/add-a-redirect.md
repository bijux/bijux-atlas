# How To Add A Redirect

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: define the supported redirect workflow.

## Procedure

1. Add the old `docs/...` path and new `docs/...` path to `docs/redirects.json`.
2. Run `python3 scripts/docs/sync_redirects.py`.
3. Run `python3 scripts/docs/check_navigation_policy.py`.
4. Run `mkdocs build --strict`.

## Constraints

- Redirects are only rendered through MkDocs for markdown-to-markdown paths.
- Non-markdown aliases may exist in `docs/redirects.json`, but they stay generator-owned and do not
  belong in `redirect_maps`.
- The destination page must already exist before the redirect is added.
