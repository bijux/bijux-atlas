// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
//! Atlas model compatibility surface.
//!
//! ```compile_fail
//! use bijux_atlas::model::DiffStatus;
//!
//! fn exhaustive_match(s: DiffStatus) -> &'static str {
//!     match s {
//!         DiffStatus::Added => "a",
//!         DiffStatus::Removed => "r",
//!         DiffStatus::Changed => "c",
//!     }
//! }
//! ```

pub use crate::domain::dataset::serde_helpers;

pub use crate::domain::dataset::{
    keys::{
        normalize_assembly, normalize_release, normalize_species, parse_assembly,
        parse_dataset_key, parse_release, parse_species, parse_species_normalized, Assembly,
        DatasetId, DatasetSelector, Release, Species, ValidationError, ASSEMBLY_MAX_LEN,
        RELEASE_MAX_LEN, SPECIES_MAX_LEN,
    },
    manifest::{
        artifact_paths, ArtifactChecksums, ArtifactManifest, ArtifactPaths, Catalog, CatalogEntry,
        IngestAnomalyReport, IngestRejection, ManifestInputHashes, ManifestStats,
        OptionalFieldPolicy, QcSeverity, ShardCatalog, ShardEntry, ShardId, ShardingPlan,
        LATEST_ALIAS_POLICY, NO_IMPLICIT_DEFAULT_DATASET_POLICY,
    },
    version::ModelVersion,
};
pub use crate::domain::policy::model::{GeneIdentifierPolicy, StrictnessMode};
pub use crate::domain::query::diff::{
    DiffPage, DiffRecord, DiffScope, DiffStatus, GeneSignatureInput, ReleaseGeneIndex,
    ReleaseGeneIndexEntry,
};
pub use crate::domain::query::gene::{
    BiotypePolicy, DuplicateGeneIdPolicy, DuplicateTranscriptIdPolicy, FeatureIdUniquenessPolicy,
    GeneId, GeneNamePolicy, GeneOrderKey, GeneSummary, ParseError, Region, SeqId,
    SeqidNormalizationPolicy, Strand, TranscriptId, TranscriptIdPolicy, TranscriptOrderKey,
    TranscriptTypePolicy, UnknownFeaturePolicy, ID_MAX_LEN, NAME_MAX_LEN, SEQID_MAX_LEN,
};

pub const CRATE_NAME: &str = "bijux-atlas";
