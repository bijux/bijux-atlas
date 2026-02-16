# PUBLIC API: bijux-atlas-model

Stable exports:

- `CRATE_NAME`
- Dataset identity: `Release`, `Species`, `Assembly`, `DatasetId`, `DatasetSelector`
- Normalizers: `normalize_release`, `normalize_species`, `normalize_assembly`
- Gene domain: `GeneId`, `SeqId`, `GeneSummary`
- Policies: `StrictnessMode`, `GeneIdentifierPolicy`, `GeneNamePolicy`, `BiotypePolicy`, `TranscriptTypePolicy`, `SeqidNormalizationPolicy`, `DuplicateGeneIdPolicy`
- Artifact contract: `ArtifactChecksums`, `ManifestStats`, `ArtifactManifest`, `Catalog`, `CatalogEntry`, `IngestAnomalyReport`, `OptionalFieldPolicy`
- Artifact paths: `ArtifactPaths`, `artifact_paths`
- Policy constants: `LATEST_ALIAS_POLICY`, `NO_IMPLICIT_DEFAULT_DATASET_POLICY`
- Error: `ValidationError`

`src/lib.rs` must only export documented items unless this file is updated in the same change.
