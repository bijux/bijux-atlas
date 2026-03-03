# Release Evidence Contracts

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@5fcfe93aaeed218cb75ecb5c143ee3129fbe4bcf`
- Last changed: `2026-03-03`
- Reason to exist: document the `REL-EVID-*` release evidence contract set.

## Contract Set

- `REL-EVID-001`: `release/evidence/manifest.json` exists and satisfies `release/evidence/manifest.schema.json`.
- `REL-EVID-002`: `release/evidence/identity.json` exists and satisfies `release/evidence/identity.schema.json`.
- `REL-EVID-003`: the packaged chart checksum is present in the manifest.
- `REL-EVID-004`: prod profiles record non-empty image artifact coverage.
- `REL-EVID-005`: required reports and simulation summaries are listed when present.
- `REL-EVID-006`: docs site summary is included.
- `REL-EVID-007`: the evidence tarball is reproducible for identical inputs because the bundler uses stable ordering and normalized metadata.

## Validation Surface

- `ops evidence collect` generates the governed files.
- `ops evidence verify <tarball>` validates the generated bundle.
- `ops evidence diff <tarballA> <tarballB>` provides a structured difference between two bundles.
