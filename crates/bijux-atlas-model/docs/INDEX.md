# Model Docs Index

`bijux-atlas-model` is the SSOT for atlas domain types.

Primary domain objects:
- `DatasetId` and selector/normalization types (`Release`, `Species`, `Assembly`).
- Gene-domain value types (`GeneId`, `SeqId`, `GeneSummary`).
- Artifact contracts (`ArtifactManifest`, `Catalog`, anomaly report).

Docs:
- [Architecture](ARCHITECTURE.md)
- [Public API](PUBLIC_API.md)
- [Schema stability rules](SCHEMA_STABILITY.md)
- [Schema evolution notes](SCHEMA_EVOLUTION_NOTES.md)
- [Purity policy](PURITY.md)
- [Effects policy](EFFECTS.md)
- [Strict ordering rules](ORDERING_RULES.md)
- [Migration notes](MIGRATION.md)
- [Optional fields policy](OPTIONAL_FIELDS.md)
- [What is NOT in model](NOT_IN_MODEL.md)
- [Compatibility with bijux-dna](COMPAT_BIJUX_DNA.md)

## API stability

Public API is defined only by `docs/PUBLIC_API.md`; all other symbols are internal and may change without notice.

## Invariants

Core invariants for this crate must remain true across releases and tests.

## Failure modes

Failure modes are documented and mapped to stable error handling behavior.

## How to extend

Additions must preserve crate boundaries, update `docs/PUBLIC_API.md`, and add targeted tests.

