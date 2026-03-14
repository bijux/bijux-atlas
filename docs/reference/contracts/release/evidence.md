# Release Evidence Contracts

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: document the `REL-EVID-*` release evidence contract set.

## Contract Set

- `REL-EVID-001`: `ops/release/evidence/manifest.json` exists and satisfies `ops/release/evidence/manifest.schema.json`.
- `REL-EVID-002`: `ops/release/evidence/identity.json` exists and satisfies `ops/release/evidence/identity.schema.json`.
- `REL-EVID-003`: the packaged chart checksum is present in the manifest.
- `REL-EVID-004`: prod profiles record non-empty image artifact coverage.
- `REL-EVID-005`: required reports and simulation summaries are listed when present.
- `REL-EVID-006`: docs site summary is included.
- `REL-EVID-007`: the evidence tarball is reproducible for identical inputs because the bundler uses stable ordering and normalized metadata.

## Validation Surface

- `ops evidence collect` generates the governed files.
- `ops evidence verify <tarball>` validates the generated bundle.
- `ops evidence diff <tarballA> <tarballB>` provides a structured difference between two bundles.
