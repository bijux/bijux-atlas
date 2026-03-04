# Docs Artifact Contract

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define which documentation artifacts are committed, which are build outputs, and how they are maintained.

## Committed Artifacts

The repository commits:

- authored reader pages under `docs/`
- contributor governance pages under `docs/_internal/governance/`
- committed generated markdown under `docs/_internal/generated/` when reviewability matters

Committed generated markdown must carry the standard generated header and must be refreshed with the control-plane.

## Build-Only Artifacts

The repository does not commit the rendered site under `artifacts/docs/site`. That directory is a build output only.
Machine-readable support artifacts under `artifacts/` are generated evidence, not source documentation.

## Ownership

`docs/_internal/governance/generated-content-policy.md` is the policy authority for immutable generated docs.
`docs/_internal/governance/docs-publication-semantics.md` is the publication authority for the rendered site.

## Enforcement

The contract lane rejects committed generated markdown that omits the generated header or implies manual editing is
acceptable. Regeneration must happen through the canonical `bijux-dev-atlas docs ...` command flow.
