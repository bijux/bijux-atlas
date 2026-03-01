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
- [Schema stability rules](schema-stability.md)
- [Contract compatibility](contract-compatibility.md)
- [Schema evolution notes](schema-evolution-notes.md)
- [Purity policy](purity.md)
- [Effects policy](effects.md)
- [Strict ordering rules](ordering-rules.md)
- [Optional fields policy](optional-fields.md)
- [What is NOT in model](not-in-model.md)
- [Compatibility with bijux-dna](compat-bijux-dna.md)

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
