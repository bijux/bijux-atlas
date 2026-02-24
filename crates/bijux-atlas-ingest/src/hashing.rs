use std::fs;
use std::path::Path;

use bijux_atlas_core::sha256_hex;

use crate::IngestError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputHashes {
    pub gff3_sha256: String,
    pub fasta_sha256: String,
    pub fai_sha256: String,
}

pub fn hash_file(path: &Path) -> Result<String, IngestError> {
    let bytes = fs::read(path).map_err(|e| IngestError(e.to_string()))?;
    Ok(sha256_hex(&bytes))
}

pub fn compute_input_hashes(gff3: &Path, fasta: &Path, fai: &Path) -> Result<InputHashes, IngestError> {
    Ok(InputHashes {
        gff3_sha256: hash_file(gff3)?,
        fasta_sha256: hash_file(fasta)?,
        fai_sha256: hash_file(fai)?,
    })
}
