# Artifact model

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define artifact structure, publication semantics, and schema boundaries.

## Artifact definition

- Artifact is the immutable output of ingest and QC.
- Artifact identity is content-addressed and release-mapped through registry metadata.
- Artifact schemas are contract-governed and versioned.

## Structure

- Dataset payload segments
- Manifest metadata
- Integrity hashes and compatibility metadata

## Terminology used here

- Artifact: [Glossary](../glossary.md)
- Release: [Glossary](../glossary.md)

## Next steps

- [Runtime data model](runtime-data-model.md)
- [Reference schemas](../reference/schemas.md)
- [Reference contracts artifacts](../reference/contracts/artifacts/index.md)
