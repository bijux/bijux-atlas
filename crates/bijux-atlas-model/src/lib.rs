#![forbid(unsafe_code)]

mod dataset;
mod diff;
mod gene;
mod manifest;
mod policy;

pub use dataset::{
    normalize_assembly, normalize_release, normalize_species, Assembly, DatasetId, DatasetSelector,
    Release, Species, ValidationError,
};
pub use diff::{
    DiffPage, DiffRecord, DiffScope, DiffStatus, GeneSignatureInput, ReleaseGeneIndex,
    ReleaseGeneIndexEntry,
};
pub use gene::{
    BiotypePolicy, DuplicateGeneIdPolicy, GeneId, GeneNamePolicy, GeneSummary, SeqId,
    SeqidNormalizationPolicy, TranscriptTypePolicy,
};
pub use manifest::{
    artifact_paths, ArtifactChecksums, ArtifactManifest, ArtifactPaths, Catalog, CatalogEntry,
    IngestAnomalyReport, ManifestStats, OptionalFieldPolicy, QcSeverity, ShardCatalog, ShardEntry,
    LATEST_ALIAS_POLICY, NO_IMPLICIT_DEFAULT_DATASET_POLICY,
};
pub use policy::{GeneIdentifierPolicy, StrictnessMode};

pub const CRATE_NAME: &str = "bijux-atlas-model";
