use crate::extract::ExtractResult;
use crate::IngestError;
use bijux_atlas_core::canonical;
use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{
    ArtifactChecksums, ArtifactManifest, DatasetId, ManifestStats, ValidationError,
};
use serde_json::json;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct BuiltManifest {
    pub manifest: ArtifactManifest,
    pub qc_report_path: PathBuf,
}

pub fn build_and_write_manifest_and_reports(
    output_root: &Path,
    dataset: &DatasetId,
    gff3_path: &Path,
    fasta_path: &Path,
    fai_path: &Path,
    sqlite_path: &Path,
    manifest_path: &Path,
    anomaly_path: &Path,
    extract: &ExtractResult,
) -> Result<BuiltManifest, IngestError> {
    let mut total_transcripts = 0_u64;
    let mut contigs = BTreeSet::new();
    for g in &extract.gene_rows {
        total_transcripts += g.transcript_count;
        contigs.insert(g.seqid.clone());
    }

    let manifest = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        dataset.clone(),
        ArtifactChecksums::new(
            sha256_hex(&fs::read(gff3_path).map_err(|e| IngestError(e.to_string()))?),
            sha256_hex(&fs::read(fasta_path).map_err(|e| IngestError(e.to_string()))?),
            sha256_hex(&fs::read(fai_path).map_err(|e| IngestError(e.to_string()))?),
            sha256_hex(&fs::read(sqlite_path).map_err(|e| IngestError(e.to_string()))?),
        ),
        ManifestStats::new(
            extract.gene_rows.len() as u64,
            total_transcripts,
            contigs.len() as u64,
        ),
    );

    manifest
        .validate_strict()
        .map_err(|e: ValidationError| IngestError(e.to_string()))?;

    let manifest_bytes =
        canonical::stable_json_bytes(&manifest).map_err(|e| IngestError(e.to_string()))?;
    fs::write(manifest_path, manifest_bytes).map_err(|e| IngestError(e.to_string()))?;

    let anomaly_bytes =
        canonical::stable_json_bytes(&extract.anomaly).map_err(|e| IngestError(e.to_string()))?;
    fs::write(anomaly_path, anomaly_bytes).map_err(|e| IngestError(e.to_string()))?;

    let qc_report = json!({
        "dataset": dataset,
        "gene_count": extract.gene_rows.len(),
        "transcript_count": total_transcripts,
        "biotype_distribution": extract.biotype_distribution,
        "contig_distribution": extract.contig_distribution,
        "anomalies": {
            "missing_parents": extract.anomaly.missing_parents,
            "unknown_contigs": extract.anomaly.unknown_contigs,
            "overlapping_ids": extract.anomaly.overlapping_ids,
            "duplicate_gene_ids": extract.anomaly.duplicate_gene_ids,
        }
    });
    let qc_bytes =
        canonical::stable_json_bytes(&qc_report).map_err(|e| IngestError(e.to_string()))?;
    let qc_report_path = output_root
        .join(format!("release={}", dataset.release))
        .join(format!("species={}", dataset.species))
        .join(format!("assembly={}", dataset.assembly))
        .join("derived")
        .join("qc_report.json");
    fs::write(&qc_report_path, qc_bytes).map_err(|e| IngestError(e.to_string()))?;

    Ok(BuiltManifest {
        manifest,
        qc_report_path,
    })
}
