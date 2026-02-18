use crate::dataset::{DatasetId, ValidationError};
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
    pub artifact_version: String,
    #[serde(default)]
    pub schema_version: String,
    pub manifest_version: String,
    pub db_schema_version: String,
    pub dataset: DatasetId,
    pub checksums: ArtifactChecksums,
    #[serde(default)]
    pub input_hashes: ManifestInputHashes,
    pub stats: ManifestStats,
    #[serde(default)]
    pub dataset_signature_sha256: String,
    #[serde(default)]
    pub schema_evolution_note: String,
    #[serde(default)]
    pub ingest_toolchain: String,
    #[serde(default)]
    pub ingest_build_hash: String,
    #[serde(default)]
    pub toolchain_hash: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default = "default_derived_column_origins")]
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
        Self {
            manifest_version,
            db_schema_version,
            dataset,
            checksums,
            artifact_version: "v1".to_string(),
            schema_version: "1".to_string(),
            input_hashes: ManifestInputHashes {
                gff3_sha256: "unknown".to_string(),
                fasta_sha256: "unknown".to_string(),
                fai_sha256: "unknown".to_string(),
                policy_sha256: "unknown".to_string(),
            },
            stats,
            dataset_signature_sha256: String::new(),
            schema_evolution_note:
                "v1 schema: additive-only evolution; existing fields remain stable".to_string(),
            ingest_toolchain: String::new(),
            ingest_build_hash: String::new(),
            toolchain_hash: "unknown".to_string(),
            created_at: String::new(),
            derived_column_origins: default_derived_column_origins(),
        }
    }

    pub fn validate_strict(&self) -> Result<(), ValidationError> {
        if self.artifact_version.trim().is_empty() {
            return Err(ValidationError(
                "artifact_version must not be empty".to_string(),
            ));
        }
        if self.schema_version.trim().is_empty() {
            return Err(ValidationError("schema_version must not be empty".to_string()));
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
        if self.stats.gene_count == 0 {
            return Err(ValidationError("gene_count must be > 0".to_string()));
        }
        if self.derived_column_origins.is_empty() {
            return Err(ValidationError(
                "derived_column_origins must not be empty".to_string(),
            ));
        }
        Ok(())
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
        release_gene_index: derived.join("release_gene_index.json"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct Catalog {
    pub datasets: Vec<CatalogEntry>,
}

impl Catalog {
    #[must_use]
    pub fn new(datasets: Vec<CatalogEntry>) -> Self {
        Self { datasets }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct CatalogEntry {
    pub dataset: DatasetId,
    pub manifest_path: String,
    pub sqlite_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ShardCatalog {
    pub dataset: DatasetId,
    pub mode: String,
    pub shards: Vec<ShardEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ShardEntry {
    pub shard_id: String,
    pub seqids: Vec<String>,
    pub sqlite_path: String,
    pub sqlite_sha256: String,
}

impl ShardEntry {
    #[must_use]
    pub fn new(
        shard_id: String,
        seqids: Vec<String>,
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
            if item.shard_id.trim().is_empty()
                || item.sqlite_path.trim().is_empty()
                || item.sqlite_sha256.trim().is_empty()
            {
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
}

impl CatalogEntry {
    #[must_use]
    pub fn new(dataset: DatasetId, manifest_path: String, sqlite_path: String) -> Self {
        Self {
            dataset,
            manifest_path,
            sqlite_path,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct IngestAnomalyReport {
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
    "API must not use implicit default dataset; release/species/assembly are required";
