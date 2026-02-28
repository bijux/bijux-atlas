# Docs Build Publish Model

- Owner: `docs-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: explain how docs are built, validated, and published without exposing tooling artifacts in reader navigation.

## Build model

- Source docs live under reader sections and governance.
- Generated artifacts are tooling outputs and not part of reader docs surface.
- Contract and docs checks enforce link and metadata quality before publish.

## Publish model

- Reader navigation includes only curated section indexes.
- Contributor-only governance pages remain outside reader start paths.
- `_generated` outputs are available for diagnostics but are not linked from user/operator docs.

## Verification

Run docs and contract checks before publishing.

## Next

- [Generated Content Policy](generated-content-policy.md)
- [Docs Dashboard](docs-dashboard.md)
