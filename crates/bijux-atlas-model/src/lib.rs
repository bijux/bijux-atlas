// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod dataset;
pub mod diff;
pub mod gene;
pub mod policy;
pub mod serde_helpers {
    pub use crate::dataset::serde_helpers::*;
}

pub use dataset::{
    artifact_paths, canonical_identity_hash, normalize_assembly, normalize_release,
    normalize_species, parse_assembly, parse_dataset_key, parse_release, parse_species,
    parse_species_normalized, ArtifactChecksums, ArtifactManifest, ArtifactPaths, Assembly,
    Catalog, CatalogEntry, DatasetId, DatasetIdentity, DatasetLifecycleState,
    DatasetLifecycleTransition, DatasetSelector, IngestAnomalyClass, IngestAnomalyReport,
    IngestRejection, LatestAliasRecord, ManifestInputHashes, ManifestStats, ModelVersion,
    OptionalFieldPolicy, QcSeverity, Release, ShardCatalog, ShardEntry, ShardId, ShardingPlan,
    Species, ValidationError, ASSEMBLY_MAX_LEN, LATEST_ALIAS_POLICY,
    NO_IMPLICIT_DEFAULT_DATASET_POLICY, RELEASE_MAX_LEN, SPECIES_MAX_LEN,
};
pub use diff::{
    DiffPage, DiffRecord, DiffScope, DiffStatus, GeneSignatureInput, ReleaseGeneIndex,
    ReleaseGeneIndexEntry,
};
pub use gene::{
    BiotypePolicy, DuplicateGeneIdPolicy, DuplicateTranscriptIdPolicy, FeatureIdUniquenessPolicy,
    GeneId, GeneNamePolicy, GeneOrderKey, GeneSummary, ParseError, Region, SeqId,
    SeqidNormalizationPolicy, SeqidNormalizationTrace, Strand, TranscriptId, TranscriptIdPolicy,
    TranscriptOrderKey, TranscriptTypePolicy, UnknownFeaturePolicy, ID_MAX_LEN, NAME_MAX_LEN,
    SEQID_MAX_LEN,
};
pub use policy::{GeneIdentifierPolicy, StrictnessMode};

pub const CRATE_NAME: &str = "bijux-atlas-model";
