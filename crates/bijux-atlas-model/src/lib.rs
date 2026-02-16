#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

pub const CRATE_NAME: &str = "bijux-atlas-model";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError(pub String);

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for ValidationError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct DatasetId {
    pub release: String,
    pub species: String,
    pub assembly: String,
}

impl DatasetId {
    pub fn new(release: &str, species: &str, assembly: &str) -> Result<Self, ValidationError> {
        Ok(Self {
            release: normalize_release(release)?,
            species: normalize_species(species)?,
            assembly: normalize_assembly(assembly)?,
        })
    }
}

pub fn normalize_species(input: &str) -> Result<String, ValidationError> {
    let s = input.trim().to_ascii_lowercase().replace('-', "_");
    if s.is_empty() {
        return Err(ValidationError("species must not be empty".to_string()));
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(ValidationError(
            "species must match [a-z0-9_]+ in snake_case".to_string(),
        ));
    }
    if s.starts_with('_') || s.ends_with('_') || s.contains("__") {
        return Err(ValidationError(
            "species must not start/end with '_' or contain '__'".to_string(),
        ));
    }
    Ok(s)
}

pub fn normalize_assembly(input: &str) -> Result<String, ValidationError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ValidationError("assembly must not be empty".to_string()));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_')
    {
        return Err(ValidationError(
            "assembly must match [A-Za-z0-9._]+".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

pub fn normalize_release(input: &str) -> Result<String, ValidationError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(ValidationError("release must not be empty".to_string()));
    }
    if !s.chars().all(|c| c.is_ascii_digit()) {
        return Err(ValidationError(
            "release must be numeric string (e.g. 110)".to_string(),
        ));
    }
    Ok(s.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactChecksums {
    pub gff3_sha256: String,
    pub fasta_sha256: String,
    pub fai_sha256: String,
    pub sqlite_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestStats {
    pub gene_count: u64,
    pub transcript_count: u64,
    pub contig_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactManifest {
    pub manifest_version: String,
    pub db_schema_version: String,
    pub dataset: DatasetId,
    pub checksums: ArtifactChecksums,
    pub stats: ManifestStats,
}

impl ArtifactManifest {
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
pub struct Catalog {
    pub datasets: Vec<CatalogEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct CatalogEntry {
    pub dataset: DatasetId,
    pub manifest_path: String,
    pub sqlite_path: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum StrictnessMode {
    Strict,
    Lenient,
    ReportOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum GeneIdentifierPolicy {
    Gff3Id,
    PreferEnsemblStableId {
        attribute_keys: Vec<String>,
        fallback_to_gff3_id: bool,
    },
}

impl Default for GeneIdentifierPolicy {
    fn default() -> Self {
        Self::Gff3Id
    }
}

impl GeneIdentifierPolicy {
    pub fn resolve(
        &self,
        attrs: &BTreeMap<String, String>,
        gff3_id: &str,
        strict: bool,
    ) -> Result<String, ValidationError> {
        match self {
            Self::Gff3Id => Ok(gff3_id.to_string()),
            Self::PreferEnsemblStableId {
                attribute_keys,
                fallback_to_gff3_id,
            } => {
                for key in attribute_keys {
                    if let Some(value) = attrs.get(key) {
                        let v = value.trim();
                        if !v.is_empty() {
                            if strict && !v.starts_with("ENS") {
                                return Err(ValidationError(
                                    "strict mode requires ENS* stable ID when using Ensembl policy"
                                        .to_string(),
                                ));
                            }
                            return Ok(v.to_string());
                        }
                    }
                }
                if *fallback_to_gff3_id {
                    Ok(gff3_id.to_string())
                } else {
                    Err(ValidationError(
                        "no Ensembl stable ID found and fallback disabled".to_string(),
                    ))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct GeneNamePolicy {
    pub attribute_keys: Vec<String>,
}

impl Default for GeneNamePolicy {
    fn default() -> Self {
        Self {
            attribute_keys: vec![
                "gene_name".to_string(),
                "Name".to_string(),
                "gene".to_string(),
                "description".to_string(),
            ],
        }
    }
}

impl GeneNamePolicy {
    #[must_use]
    pub fn from_keys(attribute_keys: Vec<String>) -> Self {
        Self { attribute_keys }
    }

    #[must_use]
    pub fn resolve(&self, attrs: &BTreeMap<String, String>, fallback: &str) -> String {
        for key in &self.attribute_keys {
            if let Some(value) = attrs.get(key) {
                let v = value.split_whitespace().collect::<Vec<_>>().join(" ");
                if !v.is_empty() {
                    return v;
                }
            }
        }
        fallback.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct BiotypePolicy {
    pub attribute_keys: Vec<String>,
    pub unknown_value: String,
}

impl Default for BiotypePolicy {
    fn default() -> Self {
        Self {
            attribute_keys: vec![
                "gene_biotype".to_string(),
                "biotype".to_string(),
                "gene_type".to_string(),
            ],
            unknown_value: "unknown".to_string(),
        }
    }
}

impl BiotypePolicy {
    #[must_use]
    pub fn from_keys_and_unknown(attribute_keys: Vec<String>, unknown_value: String) -> Self {
        Self {
            attribute_keys,
            unknown_value,
        }
    }

    #[must_use]
    pub fn resolve(&self, attrs: &BTreeMap<String, String>) -> String {
        for key in &self.attribute_keys {
            if let Some(value) = attrs.get(key) {
                let v = value.split_whitespace().collect::<Vec<_>>().join(" ");
                if !v.is_empty() {
                    return v;
                }
            }
        }
        self.unknown_value.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct TranscriptTypePolicy {
    pub accepted_types: BTreeSet<String>,
}

impl Default for TranscriptTypePolicy {
    fn default() -> Self {
        Self {
            accepted_types: BTreeSet::from([
                "transcript".to_string(),
                "mRNA".to_string(),
                "mrna".to_string(),
            ]),
        }
    }
}

impl TranscriptTypePolicy {
    #[must_use]
    pub fn from_types(accepted_types: BTreeSet<String>) -> Self {
        Self { accepted_types }
    }

    #[must_use]
    pub fn accepts(&self, feature_type: &str) -> bool {
        self.accepted_types.contains(feature_type)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct SeqidNormalizationPolicy {
    pub aliases: BTreeMap<String, String>,
}

impl SeqidNormalizationPolicy {
    #[must_use]
    pub fn from_aliases(aliases: BTreeMap<String, String>) -> Self {
        Self { aliases }
    }

    #[must_use]
    pub fn normalize(&self, raw: &str) -> String {
        let trimmed = raw.trim();
        if let Some(mapped) = self.aliases.get(trimmed) {
            return mapped.clone();
        }
        trimmed.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum DuplicateGeneIdPolicy {
    Fail,
    DedupeKeepLexicographicallySmallest,
}

impl Default for DuplicateGeneIdPolicy {
    fn default() -> Self {
        Self::Fail
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct IngestAnomalyReport {
    pub missing_parents: Vec<String>,
    pub unknown_contigs: Vec<String>,
    pub overlapping_ids: Vec<String>,
    pub duplicate_gene_ids: Vec<String>,
}

pub const LATEST_ALIAS_POLICY: &str =
    "latest alias is allowed only as an explicit endpoint and must resolve deterministically";
pub const NO_IMPLICIT_DEFAULT_DATASET_POLICY: &str =
    "API must not use implicit default dataset; release/species/assembly are required";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dataset_id_normalizes() {
        let id = DatasetId::new("110", "Homo_Sapiens", "GRCh38").expect("valid dataset");
        assert_eq!(id.release, "110");
        assert_eq!(id.species, "homo_sapiens");
        assert_eq!(id.assembly, "GRCh38");
    }

    #[test]
    fn manifest_round_trip_and_strict_unknown_fields() {
        let manifest = ArtifactManifest {
            manifest_version: "1".to_string(),
            db_schema_version: "1".to_string(),
            dataset: DatasetId::new("110", "homo_sapiens", "GRCh38").expect("valid dataset"),
            checksums: ArtifactChecksums {
                gff3_sha256: "a".repeat(64),
                fasta_sha256: "b".repeat(64),
                fai_sha256: "c".repeat(64),
                sqlite_sha256: "d".repeat(64),
            },
            stats: ManifestStats {
                gene_count: 1,
                transcript_count: 2,
                contig_count: 3,
            },
        };
        manifest.validate_strict().expect("valid manifest");

        let json = serde_json::to_string(&manifest).expect("serialize manifest");
        let decoded: ArtifactManifest = serde_json::from_str(&json).expect("decode manifest");
        assert_eq!(decoded, manifest);

        let with_unknown = r#"{
            "manifest_version":"1",
            "db_schema_version":"1",
            "dataset":{"release":"110","species":"homo_sapiens","assembly":"GRCh38"},
            "checksums":{"gff3_sha256":"a","fasta_sha256":"b","fai_sha256":"c","sqlite_sha256":"d"},
            "stats":{"gene_count":1,"transcript_count":1,"contig_count":1},
            "unexpected":"field"
        }"#;
        assert!(serde_json::from_str::<ArtifactManifest>(with_unknown).is_err());
    }

    #[test]
    fn catalog_sorting_is_deterministic() {
        let catalog = Catalog {
            datasets: vec![
                CatalogEntry {
                    dataset: DatasetId::new("109", "homo_sapiens", "GRCh38").expect("valid"),
                    manifest_path: "a/manifest.json".to_string(),
                    sqlite_path: "a/gene_summary.sqlite".to_string(),
                },
                CatalogEntry {
                    dataset: DatasetId::new("110", "homo_sapiens", "GRCh38").expect("valid"),
                    manifest_path: "b/manifest.json".to_string(),
                    sqlite_path: "b/gene_summary.sqlite".to_string(),
                },
            ],
        };
        catalog.validate_sorted().expect("sorted catalog must pass");
    }

    #[test]
    fn normalization_rejects_invalid_inputs() {
        assert!(normalize_release("110a").is_err());
        assert!(normalize_species("homo sapiens").is_err());
        assert!(normalize_assembly("GRCh38!").is_err());
    }

    #[test]
    fn catalog_strict_unknown_fields_enforced() {
        let raw = r#"{
          "datasets": [],
          "unexpected": "field"
        }"#;
        assert!(serde_json::from_str::<Catalog>(raw).is_err());
    }

    #[test]
    fn gene_name_policy_variants() {
        let mut attrs = BTreeMap::new();
        attrs.insert("Name".to_string(), " BRCA1 ".to_string());
        assert_eq!(GeneNamePolicy::default().resolve(&attrs, "gene_x"), "BRCA1");

        let mut attrs2 = BTreeMap::new();
        attrs2.insert("description".to_string(), "Tumor protein p53".to_string());
        assert_eq!(
            GeneNamePolicy::default().resolve(&attrs2, "gene_y"),
            "Tumor protein p53"
        );

        let attrs3 = BTreeMap::new();
        assert_eq!(
            GeneNamePolicy::default().resolve(&attrs3, "gene_z"),
            "gene_z"
        );
    }

    #[test]
    fn biotype_policy_variants() {
        let mut attrs = BTreeMap::new();
        attrs.insert("gene_biotype".to_string(), "protein_coding".to_string());
        assert_eq!(BiotypePolicy::default().resolve(&attrs), "protein_coding");

        let attrs2 = BTreeMap::new();
        assert_eq!(BiotypePolicy::default().resolve(&attrs2), "unknown");
    }

    #[test]
    fn transcript_type_policy_variants() {
        let policy = TranscriptTypePolicy::default();
        assert!(policy.accepts("mRNA"));
        assert!(policy.accepts("transcript"));
        assert!(!policy.accepts("gene"));
    }
}
