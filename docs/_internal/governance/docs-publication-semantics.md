# Docs Publication Semantics

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define which documentation surfaces are published, which remain contributor-only, and how they are verified.

## Publication Boundary

The canonical published site is the output of `mkdocs build --strict` under `artifacts/docs/site`.
Committed markdown under `docs/` is the input surface. Reader-facing pages are publishable. Governance pages under
`docs/_internal/` remain contributor-only even when they are committed.

## Redirect Ownership

Redirects are MkDocs-native and are synchronized from `docs/redirects.json` into the generated `redirect_maps`
block in `mkdocs.yml`. Redirect policy lives with docs governance, not in ad hoc build scripts.

## Generated Artifacts

Committed generated markdown is allowed when it improves reviewability. Generated markdown must declare its
generator and must tell contributors not to edit it by hand. Machine-readable generated artifacts remain
control-plane-owned support outputs and are not part of the published reader nav.

## Canonical Command

Regenerate committed docs artifacts through `bijux-dev-atlas docs ...` commands, then validate with
`mkdocs build --strict`.
