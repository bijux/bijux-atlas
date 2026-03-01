# PUBLIC API: bijux-atlas-model

Stability reference: [Stability Levels](../../../docs/_internal/governance/style/stability-levels.md)

Stable exports:

- `CRATE_NAME`
- Dataset identity: `Release`, `Species`, `Assembly`, `DatasetId`, `DatasetSelector`
- Versioning: `ModelVersion`
- Parsers/normalizers: `parse_release`, `parse_species`, `parse_species_normalized`, `parse_assembly`, `normalize_release`, `normalize_species`, `normalize_assembly`
- Limits: `RELEASE_MAX_LEN`, `SPECIES_MAX_LEN`, `ASSEMBLY_MAX_LEN`, `ID_MAX_LEN`, `SEQID_MAX_LEN`, `NAME_MAX_LEN`
- Gene domain: `GeneId`, `TranscriptId`, `SeqId`, `Strand`, `Region`, `GeneOrderKey`, `TranscriptOrderKey`, `GeneSummary`, `ParseError`
- Policies: `StrictnessMode`, `GeneIdentifierPolicy`, `GeneNamePolicy`, `BiotypePolicy`, `TranscriptTypePolicy`, `SeqidNormalizationPolicy`, `DuplicateGeneIdPolicy`
- Artifact contract: `ArtifactChecksums`, `ManifestStats`, `ArtifactManifest`, `Catalog`, `CatalogEntry`, `ShardId`, `IngestAnomalyReport`, `OptionalFieldPolicy`
- Artifact paths: `ArtifactPaths`, `artifact_paths`
- Policy constants: `LATEST_ALIAS_POLICY`, `NO_IMPLICIT_DEFAULT_DATASET_POLICY`
- Error: `ValidationError`

`src/lib.rs` must only export documented items unless this file is updated in the same change.
