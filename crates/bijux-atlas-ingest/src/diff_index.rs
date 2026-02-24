use crate::extract::GeneRecord;
use crate::IngestError;
use bijux_atlas_core::{canonical, sha256_hex};
use bijux_atlas_model::{DatasetId, GeneSignatureInput, ReleaseGeneIndex, ReleaseGeneIndexEntry};
use std::path::Path;

fn signature_for_gene(row: &GeneRecord) -> Result<String, IngestError> {
    let payload = GeneSignatureInput::new(
        bijux_atlas_model::GeneId::parse(&row.gene_id).map_err(|e| IngestError(e.to_string()))?,
        row.gene_name.clone(),
        row.biotype.clone(),
        bijux_atlas_model::SeqId::parse(&row.seqid).map_err(|e| IngestError(e.to_string()))?,
        row.start,
        row.end,
        row.transcript_count,
    );
    let bytes = canonical::stable_json_bytes(&payload).map_err(|e| IngestError(e.to_string()))?;
    Ok(sha256_hex(&bytes))
}

pub fn build_and_write_release_gene_index(
    dataset: &DatasetId,
    output_path: &Path,
    rows: &[GeneRecord],
) -> Result<(), IngestError> {
    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        entries.push(ReleaseGeneIndexEntry::new(
            bijux_atlas_model::GeneId::parse(&row.gene_id).map_err(|e| IngestError(e.to_string()))?,
            bijux_atlas_model::SeqId::parse(&row.seqid).map_err(|e| IngestError(e.to_string()))?,
            row.start,
            row.end,
            signature_for_gene(row)?,
        ));
    }
    entries.sort();
    let index = ReleaseGeneIndex::new("1".to_string(), dataset.clone(), entries);
    let bytes = canonical::stable_json_bytes(&index).map_err(|e| IngestError(e.to_string()))?;
    std::fs::write(output_path, bytes).map_err(|e| IngestError(e.to_string()))
}
