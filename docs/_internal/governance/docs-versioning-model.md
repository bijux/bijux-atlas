# Docs Versioning Model

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define how documentation versions map to branches and tags.

## Version Sources

- `main` is the live development branch for the default published docs site.
- Release tags may publish frozen snapshots when the release pipeline produces tagged site
  artifacts.

## Stability Rules

- Stable pages must remain compatible across normal documentation edits.
- Breaking structural changes require redirects or explicit deprecation handling.
- Generated diagnostics may change more often, but their generator contract stays stable.

## Verification Markers

Stable pages must record the source they were last verified against using the canonical
`main@<full_sha>` or `vX.Y.Z@<full_sha>` format.

## Reader Expectation

Readers should assume the default site reflects `main` unless a versioned release artifact is
explicitly selected.
