# Docs Publication Model

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define how documentation artifacts are produced and published.

## Publication Path

The canonical documentation publication path is:

1. Source markdown and committed generated pages live under `docs/`.
2. Deterministic generators refresh committed generated surfaces before validation.
3. `mkdocs build --strict` renders the reader site into `artifacts/docs/site`.
4. The rendered site is the only publishable site artifact.

## Source Classes

- Reader pages are the primary published content.
- Governance pages define contributor-only policy and remain authoritative for docs controls.
- Generated markdown pages are committed when reviewability matters and must declare their
  generator.
- Machine-readable `_generated` artifacts remain generator-owned and are not part of the MkDocs
  redirect surface.

## Validation

- Redirect maps are synchronized from `docs/redirects.json`.
- Navigation rules are validated before publish.
- Generated health diagnostics are refreshed before publish.

## Publish Boundary

Do not publish raw generator inputs, local scratch files, or non-markdown `_generated` artifacts as
part of the reader site.
