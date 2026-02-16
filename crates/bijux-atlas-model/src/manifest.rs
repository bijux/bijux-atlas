use crate::dataset::{DatasetId, ValidationError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ArtifactChecksums {
    pub gff3_sha256: String,
    pub fasta_sha256: String,
    pub fai_sha256: String,
    pub sqlite_sha256: String,
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
    pub manifest_version: String,
    pub db_schema_version: String,
    pub dataset: DatasetId,
    pub checksums: ArtifactChecksums,
    pub stats: ManifestStats,
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
            stats,
        }
    }

    pub fn validate_strict(&self) -> Result<(), ValidationError> {
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
        if self.stats.gene_count == 0 {
            return Err(ValidationError("gene_count must be > 0".to_string()));
        }
        Ok(())
    }
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
    pub unknown_contigs: Vec<String>,
    pub overlapping_ids: Vec<String>,
    pub duplicate_gene_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub enum OptionalFieldPolicy {
    NullWhenMissing,
    OmitWhenMissing,
}

pub const LATEST_ALIAS_POLICY: &str =
    "latest alias is allowed only as an explicit endpoint and must resolve deterministically";
pub const NO_IMPLICIT_DEFAULT_DATASET_POLICY: &str =
    "API must not use implicit default dataset; release/species/assembly are required";
