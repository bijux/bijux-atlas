// SPDX-License-Identifier: Apache-2.0

use super::identity::DatasetIdentity;
use super::keys::{DatasetId, ValidationError};
use super::serde_helpers as dataset_serde;
use super::version::ModelVersion;
use crate::domain::query::gene::SeqId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ArtifactChecksums {
    pub gff3_sha256: String,
    pub fasta_sha256: String,
    pub fai_sha256: String,
    pub sqlite_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ManifestInputHashes {
    pub gff3_sha256: String,
    pub fasta_sha256: String,
    pub fai_sha256: String,
    pub policy_sha256: String,
}

impl ManifestInputHashes {
    #[must_use]
    pub fn new(
        gff3_sha256: String,
        fasta_sha256: String,
        fai_sha256: String,
        policy_sha256: String,
    ) -> Self {
        Self {
            gff3_sha256,
            fasta_sha256,
            fai_sha256,
            policy_sha256,
        }
    }
}

impl ArtifactChecksums {
    #[must_use]
    pub fn new(
        gff3_sha256: String,
        fasta_sha256: String,
        fai_sha256: String,
        sqlite_sha256: String,
    ) -> Self {
        Self {
            gff3_sha256,
            fasta_sha256,
            fai_sha256,
            sqlite_sha256,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ManifestStats {
    pub gene_count: u64,
    pub transcript_count: u64,
    pub contig_count: u64,
}

impl ManifestStats {
    #[must_use]
    pub fn new(gene_count: u64, transcript_count: u64, contig_count: u64) -> Self {
        Self {
            gene_count,
            transcript_count,
            contig_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ArtifactManifest {
    #[serde(default)]
    pub model_version: ModelVersion,
    #[serde(default)]
    pub artifact_version: String,
    #[serde(default)]
    pub schema_version: String,
    #[serde(default = "default_identity_schema_version")]
    pub identity_schema_version: String,
    pub manifest_version: String,
    pub db_schema_version: String,
    pub dataset: DatasetId,
    pub checksums: ArtifactChecksums,
    #[serde(default)]
    pub input_hashes: ManifestInputHashes,
    pub stats: ManifestStats,
    #[serde(default)]
    pub identity: DatasetIdentity,
    #[serde(default)]
    pub dataset_signature_sha256: String,
    #[serde(default)]
    pub db_hash: String,
    #[serde(default)]
    pub artifact_hash: String,
    #[serde(default)]
    pub schema_evolution_note: String,
    #[serde(default)]
    pub ingest_toolchain: String,
    #[serde(default)]
    pub ingest_build_hash: String,
    #[serde(default)]
    pub toolchain_hash: String,
    #[serde(default)]
    #[serde(with = "dataset_serde::timestamp_string")]
    pub created_at: String,
    #[serde(default)]
    pub qc_report_path: String,
    #[serde(default)]
    pub canonical_feature_summary_path: String,
    #[serde(default)]
    pub source_gff3_filename: String,
    #[serde(default)]
    pub source_fasta_filename: String,
    #[serde(default)]
    pub source_fai_filename: String,
    #[serde(default)]
    pub source_facts_path: String,
    #[serde(default)]
    pub normalized_input_identity_sha256: String,
    #[serde(default)]
    pub software_version: String,
    #[serde(default)]
    pub config_version: String,
    #[serde(default)]
    pub build_policy_version: String,
    #[serde(default)]
    pub build_metadata_path: String,
    #[serde(default)]
    pub anomaly_summary_path: String,
    #[serde(default)]
    pub dataset_stats_path: String,
    #[serde(default)]
    pub artifact_inventory_path: String,
    #[serde(default)]
    pub evidence_bundle_path: String,
    #[serde(default)]
    pub evidence_bundle_sha256: String,
    #[serde(default = "default_sharding_plan")]
    pub sharding_plan: ShardingPlan,
    #[serde(default)]
    pub canonical_model_schema_version: u64,
    #[serde(default)]
    pub canonical_query_semantic_sha256: String,
    #[serde(default)]
    pub canonical_lineage_sha256: String,
    #[serde(default, skip_serializing_if = "dataset_serde::map_is_empty")]
    pub canonical_feature_counts: BTreeMap<String, u64>,
    #[serde(default, skip_serializing_if = "dataset_serde::map_is_empty")]
    pub contig_normalization_aliases: BTreeMap<String, String>,
    #[serde(
        default = "default_derived_column_origins",
        skip_serializing_if = "dataset_serde::map_is_empty"
    )]
    pub derived_column_origins: BTreeMap<String, String>,
}

impl ArtifactManifest {
    #[must_use]
    pub fn new(
        manifest_version: String,
        db_schema_version: String,
        dataset: DatasetId,
        checksums: ArtifactChecksums,
        stats: ManifestStats,
    ) -> Self {
        let mut manifest = Self {
            model_version: ModelVersion::V1,
            manifest_version,
            db_schema_version,
            dataset,
            checksums,
            artifact_version: "v1".to_string(),
            schema_version: "1".to_string(),
            identity_schema_version: default_identity_schema_version(),
            input_hashes: ManifestInputHashes {
                gff3_sha256: "unknown".to_string(),
                fasta_sha256: "unknown".to_string(),
                fai_sha256: "unknown".to_string(),
                policy_sha256: "unknown".to_string(),
            },
            stats,
            identity: DatasetIdentity::default(),
            dataset_signature_sha256: String::new(),
            db_hash: String::new(),
            artifact_hash: String::new(),
            schema_evolution_note:
                "v1 schema: additive-only evolution; existing fields remain stable".to_string(),
            ingest_toolchain: String::new(),
            ingest_build_hash: String::new(),
            toolchain_hash: "unknown".to_string(),
            created_at: String::new(),
            qc_report_path: String::new(),
            canonical_feature_summary_path: String::new(),
            source_gff3_filename: String::new(),
            source_fasta_filename: String::new(),
            source_fai_filename: String::new(),
            source_facts_path: String::new(),
            normalized_input_identity_sha256: String::new(),
            software_version: String::new(),
            config_version: String::new(),
            build_policy_version: String::new(),
            build_metadata_path: String::new(),
            anomaly_summary_path: String::new(),
            dataset_stats_path: String::new(),
            artifact_inventory_path: String::new(),
            evidence_bundle_path: String::new(),
            evidence_bundle_sha256: String::new(),
            sharding_plan: ShardingPlan::None,
            canonical_model_schema_version: 0,
            canonical_query_semantic_sha256: String::new(),
            canonical_lineage_sha256: String::new(),
            canonical_feature_counts: BTreeMap::new(),
            contig_normalization_aliases: BTreeMap::new(),
            derived_column_origins: default_derived_column_origins(),
        };
        manifest.identity = manifest.expected_identity().unwrap_or_default();
        manifest
    }

    pub fn validate_strict(&self) -> Result<(), ValidationError> {
        if self.artifact_version.trim().is_empty() {
            return Err(ValidationError(
                "artifact_version must not be empty".to_string(),
            ));
        }
        if self.schema_version.trim().is_empty() {
            return Err(ValidationError(
                "schema_version must not be empty".to_string(),
            ));
        }
        if self.identity_schema_version.trim().is_empty() {
            return Err(ValidationError(
                "identity_schema_version must not be empty".to_string(),
            ));
        }
        if self.manifest_version.trim().is_empty() {
            return Err(ValidationError(
                "manifest_version must not be empty".to_string(),
            ));
        }
        if self.db_schema_version.trim().is_empty() {
            return Err(ValidationError(
                "db_schema_version must not be empty".to_string(),
            ));
        }
        if self.manifest_version != self.schema_version {
            return Err(ValidationError(
                "manifest_version and schema_version must match".to_string(),
            ));
        }
        if self.schema_version != self.db_schema_version {
            return Err(ValidationError(
                "schema_version and db_schema_version must match".to_string(),
            ));
        }
        if self.identity_schema_version != "1" {
            return Err(ValidationError(
                "identity_schema_version must currently be 1".to_string(),
            ));
        }
        if self.input_hashes.gff3_sha256.trim().is_empty()
            || self.input_hashes.fasta_sha256.trim().is_empty()
            || self.input_hashes.fai_sha256.trim().is_empty()
            || self.input_hashes.policy_sha256.trim().is_empty()
        {
            return Err(ValidationError(
                "manifest input_hashes are required: gff3/fasta/fai/policy".to_string(),
            ));
        }
        if self.toolchain_hash.trim().is_empty() {
            return Err(ValidationError(
                "manifest toolchain_hash must not be empty".to_string(),
            ));
        }
        if self.source_facts_path.trim().is_empty()
            || self.normalized_input_identity_sha256.trim().is_empty()
        {
            return Err(ValidationError(
                "manifest source_facts_path and normalized_input_identity_sha256 are required"
                    .to_string(),
            ));
        }
        if self.software_version.trim().is_empty()
            || self.config_version.trim().is_empty()
            || self.build_policy_version.trim().is_empty()
        {
            return Err(ValidationError(
                "manifest software/config/build policy versions must not be empty".to_string(),
            ));
        }
        if self.build_metadata_path.trim().is_empty()
            || self.anomaly_summary_path.trim().is_empty()
            || self.dataset_stats_path.trim().is_empty()
            || self.artifact_inventory_path.trim().is_empty()
            || self.evidence_bundle_path.trim().is_empty()
            || self.evidence_bundle_sha256.trim().is_empty()
        {
            return Err(ValidationError(
                "manifest evidence artifact paths and bundle hash must be populated".to_string(),
            ));
        }
        if self.db_hash.trim().is_empty() {
            return Err(ValidationError(
                "manifest db_hash must not be empty".to_string(),
            ));
        }
        if self.artifact_hash.trim().is_empty() {
            return Err(ValidationError(
                "manifest artifact_hash must not be empty".to_string(),
            ));
        }
        if self.canonical_model_schema_version > 0 {
            if self.canonical_query_semantic_sha256.trim().is_empty()
                || self.canonical_lineage_sha256.trim().is_empty()
            {
                return Err(ValidationError(
                    "canonical hashes are required when canonical_model_schema_version is set"
                        .to_string(),
                ));
            }
        }
        if self.db_hash != self.checksums.sqlite_sha256 {
            return Err(ValidationError(
                "manifest db_hash must equal checksums.sqlite_sha256".to_string(),
            ));
        }
        if self.stats.gene_count == 0 {
            return Err(ValidationError("gene_count must be > 0".to_string()));
        }
        if self.derived_column_origins.is_empty() {
            return Err(ValidationError(
                "derived_column_origins must not be empty".to_string(),
            ));
        }
        self.identity.validate()?;
        if self.identity.release_id != self.dataset.canonical_string() {
            return Err(ValidationError(
                "identity release_id must match dataset canonical string".to_string(),
            ));
        }
        let expected_identity = self.expected_identity()?;
        if self.identity != expected_identity {
            return Err(ValidationError(
                "identity fields are contradictory to manifest source/build/artifact components"
                    .to_string(),
            ));
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        self.validate_strict()
    }

    fn source_identity_component(&self) -> serde_json::Value {
        serde_json::json!({
            "gff3_sha256": self.checksums.gff3_sha256.clone(),
            "fasta_sha256": self.checksums.fasta_sha256.clone(),
            "fai_sha256": self.checksums.fai_sha256.clone()
        })
    }

    fn build_identity_component(&self) -> serde_json::Value {
        serde_json::json!({
            "manifest_version": self.manifest_version.clone(),
            "schema_version": self.schema_version.clone(),
            "db_schema_version": self.db_schema_version.clone()
        })
    }

    fn artifact_identity_component(&self) -> serde_json::Value {
        serde_json::json!({
            "sqlite_sha256": self.checksums.sqlite_sha256.clone()
        })
    }

    fn expected_identity(&self) -> Result<DatasetIdentity, ValidationError> {
        DatasetIdentity::from_components(
            &self.dataset,
            &self.source_identity_component(),
            &self.build_identity_component(),
            &self.artifact_identity_component(),
        )
    }
}

fn default_derived_column_origins() -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    out.insert(
        "gene_summary.gene_id".to_string(),
        "GFF3 gene feature: ID attribute (or configured canonical gene ID policy)".to_string(),
    );
    out.insert(
        "gene_summary.name".to_string(),
        "GFF3 attributes using GeneNamePolicy priority keys".to_string(),
    );
    out.insert(
        "gene_summary.biotype".to_string(),
        "GFF3 attributes using BiotypePolicy priority keys (or policy-defined unknown)".to_string(),
    );
    out.insert(
        "gene_summary.seqid".to_string(),
        "GFF3 seqid normalized by dataset-local SeqidNormalizationPolicy".to_string(),
    );
    out.insert(
        "gene_summary.start/end".to_string(),
        "GFF3 feature coordinates validated against FASTA FAI contig lengths".to_string(),
    );
    out.insert(
        "gene_summary.transcript_count".to_string(),
        "Count of transcript/mRNA records whose Parent resolves to gene_id under strictness policy"
            .to_string(),
    );
    out.insert(
        "gene_summary.sequence_length".to_string(),
        "Computed as end-start+1 after contig-bound validation".to_string(),
    );
    out.insert(
        "transcript_summary.*".to_string(),
        "Derived from transcript/mRNA + exon/CDS relationships from GFF3".to_string(),
    );
    out
}

fn default_identity_schema_version() -> String {
    "1".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ShardingPlan {
    #[default]
    None,
    Contig,
    RegionGrid,
}

fn default_sharding_plan() -> ShardingPlan {
    ShardingPlan::None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactPaths {
    pub dataset_root: PathBuf,
    pub inputs_dir: PathBuf,
    pub derived_dir: PathBuf,
    pub gff3: PathBuf,
    pub fasta: PathBuf,
    pub fai: PathBuf,
    pub sqlite: PathBuf,
    pub manifest: PathBuf,
    pub anomaly_report: PathBuf,
    pub anomaly_summary: PathBuf,
    pub qc_report: PathBuf,
    pub source_facts: PathBuf,
    pub build_metadata: PathBuf,
    pub dataset_stats: PathBuf,
    pub artifact_inventory: PathBuf,
    pub evidence_bundle: PathBuf,
    pub release_gene_index: PathBuf,
}

#[must_use]
pub fn artifact_paths(root: &Path, dataset: &DatasetId) -> ArtifactPaths {
    let dataset_root = root
        .join(format!("release={}", dataset.release))
        .join(format!("species={}", dataset.species))
        .join(format!("assembly={}", dataset.assembly));
    let inputs = dataset_root.join("inputs");
    let derived = dataset_root.join("derived");
    ArtifactPaths {
        dataset_root,
        inputs_dir: inputs.clone(),
        derived_dir: derived.clone(),
        gff3: inputs.join("genes.gff3.bgz"),
        fasta: inputs.join("genome.fa.bgz"),
        fai: inputs.join("genome.fa.bgz.fai"),
        sqlite: derived.join("gene_summary.sqlite"),
        manifest: derived.join("manifest.json"),
        anomaly_report: derived.join("anomaly_report.json"),
        anomaly_summary: derived.join("anomaly_summary.json"),
        qc_report: derived.join("qc_report.json"),
        source_facts: derived.join("source_facts.json"),
        build_metadata: derived.join("build_metadata.json"),
        dataset_stats: derived.join("dataset_stats.json"),
        artifact_inventory: derived.join("artifact_inventory.json"),
        evidence_bundle: derived.join("evidence_bundle.lock.json"),
        release_gene_index: derived.join("release_gene_index.json"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct Catalog {
    #[serde(default)]
    pub model_version: ModelVersion,
    pub datasets: Vec<CatalogEntry>,
}

impl Catalog {
    #[must_use]
    pub fn new(datasets: Vec<CatalogEntry>) -> Self {
        Self {
            model_version: ModelVersion::V1,
            datasets,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct CatalogEntry {
    pub dataset: DatasetId,
    pub manifest_path: String,
    pub sqlite_path: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diff_artifacts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ShardCatalog {
    #[serde(default)]
    pub model_version: ModelVersion,
    pub dataset: DatasetId,
    pub mode: String,
    pub shards: Vec<ShardEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ShardEntry {
    pub shard_id: ShardId,
    pub seqids: Vec<SeqId>,
    pub sqlite_path: String,
    pub sqlite_sha256: String,
}

impl ShardEntry {
    #[must_use]
    pub fn new(
        shard_id: ShardId,
        seqids: Vec<SeqId>,
        sqlite_path: String,
        sqlite_sha256: String,
    ) -> Self {
        Self {
            shard_id,
            seqids,
            sqlite_path,
            sqlite_sha256,
        }
    }
}

impl ShardCatalog {
    #[must_use]
    pub fn new(dataset: DatasetId, mode: String, shards: Vec<ShardEntry>) -> Self {
        Self {
            model_version: ModelVersion::V1,
            dataset,
            mode,
            shards,
        }
    }
}

impl ShardCatalog {
    pub fn validate_sorted(&self) -> Result<(), ValidationError> {
        if self.mode.trim().is_empty() {
            return Err(ValidationError("shard mode must not be empty".to_string()));
        }
        let mut previous: Option<&ShardEntry> = None;
        for item in &self.shards {
            if item.sqlite_path.trim().is_empty() || item.sqlite_sha256.trim().is_empty() {
                return Err(ValidationError(
                    "shard catalog entries must have non-empty id/path/checksum".to_string(),
                ));
            }
            if item.seqids.is_empty() {
                return Err(ValidationError(
                    "shard catalog entries must include at least one seqid".to_string(),
                ));
            }
            if let Some(prev) = previous {
                if prev >= item {
                    return Err(ValidationError(
                        "shard catalog entries must be strictly sorted and unique".to_string(),
                    ));
                }
            }
            previous = Some(item);
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        self.validate_sorted()
    }
}

impl CatalogEntry {
    #[must_use]
    pub fn new(dataset: DatasetId, manifest_path: String, sqlite_path: String) -> Self {
        Self {
            dataset,
            manifest_path,
            sqlite_path,
            diff_artifacts: Vec::new(),
        }
    }
}

impl Catalog {
    pub fn validate_sorted(&self) -> Result<(), ValidationError> {
        let mut previous: Option<&CatalogEntry> = None;
        for item in &self.datasets {
            if item.manifest_path.trim().is_empty() || item.sqlite_path.trim().is_empty() {
                return Err(ValidationError(
                    "catalog paths must not be empty".to_string(),
                ));
            }
            if let Some(prev) = previous {
                if prev >= item {
                    return Err(ValidationError(
                        "catalog datasets must be strictly sorted and unique".to_string(),
                    ));
                }
            }
            previous = Some(item);
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        self.validate_sorted()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct IngestAnomalyReport {
    #[serde(default)]
    pub model_version: ModelVersion,
    pub missing_parents: Vec<String>,
    #[serde(default)]
    pub missing_transcript_parents: Vec<String>,
    #[serde(default)]
    pub multiple_parent_transcripts: Vec<String>,
    pub unknown_contigs: Vec<String>,
    pub overlapping_ids: Vec<String>,
    pub duplicate_gene_ids: Vec<String>,
    #[serde(default)]
    pub overlapping_gene_ids_across_contigs: Vec<String>,
    #[serde(default)]
    pub orphan_transcripts: Vec<String>,
    #[serde(default)]
    pub parent_cycles: Vec<String>,
    #[serde(default)]
    pub attribute_fallbacks: Vec<String>,
    #[serde(default)]
    pub unknown_feature_types: Vec<String>,
    #[serde(default)]
    pub missing_required_fields: Vec<String>,
    #[serde(default)]
    pub rejections: Vec<IngestRejection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct IngestRejection {
    pub line: usize,
    pub code: String,
    pub sample: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IngestAnomalyClass {
    MissingParents,
    MissingTranscriptParents,
    MultipleParentTranscripts,
    UnknownContigs,
    OverlappingIds,
    DuplicateGeneIds,
    OverlappingGeneIdsAcrossContigs,
    OrphanTranscripts,
    ParentCycles,
    AttributeFallbacks,
    UnknownFeatureTypes,
    MissingRequiredFields,
    Rejections,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, Hash)]
#[serde(transparent)]
#[non_exhaustive]
pub struct ShardId(String);

impl ShardId {
    pub fn parse(input: &str) -> Result<Self, ValidationError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ValidationError("shard_id must not be empty".to_string()));
        }
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        {
            return Err(ValidationError(
                "shard_id must contain only [a-z0-9_-]".to_string(),
            ));
        }
        Ok(Self(trimmed.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl IngestRejection {
    #[must_use]
    pub fn new(line: usize, code: String, sample: String) -> Self {
        Self { line, code, sample }
    }
}

impl IngestAnomalyReport {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        for rejection in &self.rejections {
            if rejection.line == 0 {
                return Err(ValidationError(
                    "ingest rejection line must be > 0".to_string(),
                ));
            }
            if rejection.code.trim().is_empty() {
                return Err(ValidationError(
                    "ingest rejection code must not be empty".to_string(),
                ));
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn anomaly_class_counts(&self) -> BTreeMap<IngestAnomalyClass, u64> {
        let mut counts = BTreeMap::new();
        counts.insert(IngestAnomalyClass::MissingParents, self.missing_parents.len() as u64);
        counts.insert(
            IngestAnomalyClass::MissingTranscriptParents,
            self.missing_transcript_parents.len() as u64,
        );
        counts.insert(
            IngestAnomalyClass::MultipleParentTranscripts,
            self.multiple_parent_transcripts.len() as u64,
        );
        counts.insert(IngestAnomalyClass::UnknownContigs, self.unknown_contigs.len() as u64);
        counts.insert(IngestAnomalyClass::OverlappingIds, self.overlapping_ids.len() as u64);
        counts.insert(
            IngestAnomalyClass::DuplicateGeneIds,
            self.duplicate_gene_ids.len() as u64,
        );
        counts.insert(
            IngestAnomalyClass::OverlappingGeneIdsAcrossContigs,
            self.overlapping_gene_ids_across_contigs.len() as u64,
        );
        counts.insert(
            IngestAnomalyClass::OrphanTranscripts,
            self.orphan_transcripts.len() as u64,
        );
        counts.insert(IngestAnomalyClass::ParentCycles, self.parent_cycles.len() as u64);
        counts.insert(
            IngestAnomalyClass::AttributeFallbacks,
            self.attribute_fallbacks.len() as u64,
        );
        counts.insert(
            IngestAnomalyClass::UnknownFeatureTypes,
            self.unknown_feature_types.len() as u64,
        );
        counts.insert(
            IngestAnomalyClass::MissingRequiredFields,
            self.missing_required_fields.len() as u64,
        );
        counts.insert(IngestAnomalyClass::Rejections, self.rejections.len() as u64);
        counts
    }

    #[must_use]
    pub fn severity_for_class(class: IngestAnomalyClass) -> QcSeverity {
        match class {
            IngestAnomalyClass::UnknownFeatureTypes
            | IngestAnomalyClass::MissingRequiredFields
            | IngestAnomalyClass::Rejections
            | IngestAnomalyClass::ParentCycles
            | IngestAnomalyClass::UnknownContigs => QcSeverity::Error,
            _ => QcSeverity::Warn,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
#[non_exhaustive]
pub enum QcSeverity {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub enum OptionalFieldPolicy {
    NullWhenMissing,
    OmitWhenMissing,
}

impl OptionalFieldPolicy {
    pub fn apply_to_json_map(
        self,
        map: &mut serde_json::Map<String, serde_json::Value>,
        key: &str,
        value: Option<serde_json::Value>,
    ) {
        match (self, value) {
            (_, Some(v)) => {
                map.insert(key.to_string(), v);
            }
            (Self::NullWhenMissing, None) => {
                map.insert(key.to_string(), serde_json::Value::Null);
            }
            (Self::OmitWhenMissing, None) => {}
        }
    }
}

pub const LATEST_ALIAS_POLICY: &str =
    "latest alias is allowed only as an explicit endpoint and must resolve deterministically";
pub const NO_IMPLICIT_DEFAULT_DATASET_POLICY: &str =
    "API must not use implicit default dataset; ops/release/species/assembly are required";
