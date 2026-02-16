# PUBLIC API: bijux-atlas-model

Stability contract for public types:

- `DatasetId`: stable shape; normalization behavior is stable and tested.
- `ArtifactManifest`, `ArtifactChecksums`, `ManifestStats`: stable v1 artifact contract.
- `Catalog`, `CatalogEntry`: stable deterministic catalog contract.
- Policy types (`GeneIdentifierPolicy`, `GeneNamePolicy`, `BiotypePolicy`, `TranscriptTypePolicy`, `SeqidNormalizationPolicy`, `DuplicateGeneIdPolicy`, `StrictnessMode`): semantically stable but marked `#[non_exhaustive]` where extension is expected.
- `IngestAnomalyReport`: stable top-level report keys; values are deterministic and sorted.

Rules:

- New policy variants may be added without breaking change where `#[non_exhaustive]` is used.
- New required fields in manifest/catalog are breaking and require schema version bump.
- Unknown fields in strict serde types are rejected by contract.
