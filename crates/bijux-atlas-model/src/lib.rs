#![forbid(unsafe_code)]
//! Atlas model SSOT.
//!
//! ```compile_fail
//! use bijux_atlas_model::DiffStatus;
//!
//! fn exhaustive_match(s: DiffStatus) -> &'static str {
//!     match s {
//!         DiffStatus::Added => "a",
//!         DiffStatus::Removed => "r",
//!         DiffStatus::Changed => "c",
//!     }
//! }
//! ```

mod dataset;
mod diff;
mod gene;
mod manifest;
mod policy;
pub mod serde_helpers;
mod version;

pub use dataset::{
    normalize_assembly, normalize_release, normalize_species, parse_assembly, parse_dataset_key,
    parse_release, parse_species, parse_species_normalized, Assembly, DatasetId, DatasetSelector,
    Release, Species, ValidationError, ASSEMBLY_MAX_LEN, RELEASE_MAX_LEN, SPECIES_MAX_LEN,
};
pub use diff::{
    DiffPage, DiffRecord, DiffScope, DiffStatus, GeneSignatureInput, ReleaseGeneIndex,
    ReleaseGeneIndexEntry,
};
pub use gene::{
    BiotypePolicy, DuplicateGeneIdPolicy, DuplicateTranscriptIdPolicy, FeatureIdUniquenessPolicy,
    GeneId, GeneNamePolicy, GeneOrderKey, GeneSummary, Region, SeqId, SeqidNormalizationPolicy,
    Strand, TranscriptId, TranscriptIdPolicy, TranscriptOrderKey, TranscriptTypePolicy,
    UnknownFeaturePolicy, ID_MAX_LEN, NAME_MAX_LEN, ParseError, SEQID_MAX_LEN,
};
pub use manifest::{
    artifact_paths, ArtifactChecksums, ArtifactManifest, ArtifactPaths, Catalog, CatalogEntry,
    IngestAnomalyReport, IngestRejection, ManifestInputHashes, ManifestStats, OptionalFieldPolicy,
    QcSeverity, ShardCatalog, ShardEntry, ShardId, ShardingPlan, LATEST_ALIAS_POLICY,
    NO_IMPLICIT_DEFAULT_DATASET_POLICY,
};
pub use policy::{GeneIdentifierPolicy, StrictnessMode};
pub use version::ModelVersion;

pub const CRATE_NAME: &str = "bijux-atlas-model";
