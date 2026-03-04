# Add A Release Artifact

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: describe the governed process for adding a new artifact to the release evidence set.

Related ops contracts: `OPS-ROOT-023`, `REL-EVID-005`.

## Prereqs

- A clear justification for why the artifact is required for release review
- An agreed file path that is stable across releases

## Install

1. Add the artifact generator or committed source.
2. Update `release/evidence/manifest.schema.json` if the manifest shape changes.
3. Bump the manifest schema version when the schema shape changes.
4. Update `release/signing/policy.yaml` if the artifact must be signed.
5. Regenerate evidence with `ops evidence collect`.
6. Regenerate signing artifacts with `release sign`.

## Verify

- Run `ops evidence verify`.
- Run `release verify`.
- If two bundles should be equivalent, run `release diff`.

## Rollback

- Remove the artifact from the manifest and signing policy if the new surface cannot be validated.
- Restore the previous manifest schema version only by reverting the schema change as one unit.
