# Model Docs Index

`bijux-atlas-model` is the SSOT for atlas domain types.

Primary domain objects:
- `DatasetId` and selector/normalization types (`Release`, `Species`, `Assembly`).
- Gene-domain value types (`GeneId`, `SeqId`, `GeneSummary`).
- Artifact contracts (`ArtifactManifest`, `Catalog`, anomaly report).

Docs:
- [Architecture](architecture.md)
- [Public API](public-api.md)
- Ingest contract reference: [`../../bijux-atlas-ingest/docs/ingest-contract.md`](../../bijux-atlas-ingest/docs/ingest-contract.md)
- [Schema stability rules](SCHEMA_STABILITY.md)
- [Contract compatibility](CONTRACT_COMPATIBILITY.md)
- [Schema evolution notes](SCHEMA_EVOLUTION_NOTES.md)
- [Purity policy](purity.md)
- [Effects policy](effects.md)
- [Strict ordering rules](ORDERING_RULES.md)
- [Optional fields policy](OPTIONAL_FIELDS.md)
- [What is NOT in model](NOT_IN_MODEL.md)
- [Compatibility with bijux-dna](COMPAT_BIJUX_DNA.md)

- [How to test](testing.md)
- [How to extend](#how-to-extend)

## API stability

Public API is defined only by `docs/public-api.md`; all other symbols are internal and may change without notice.

## Invariants

Core invariants for this crate must remain true across releases and tests.

## Failure modes

Failure modes are documented and mapped to stable error handling behavior.

## How to extend

Additions must preserve crate boundaries, update `docs/public-api.md`, and add targeted tests.

- Central docs index: docs/index.md
- Crate README: ../README.md
