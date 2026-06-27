// SPDX-License-Identifier: Apache-2.0

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::CacheError;

pub(crate) fn read_bytes(path: &Path) -> Result<Vec<u8>, CacheError> {
    std::fs::read(path).map_err(|e| CacheError(e.to_string()))
}

pub(crate) fn read_to_string(path: &Path) -> Result<String, CacheError> {
    std::fs::read_to_string(path).map_err(|e| CacheError(e.to_string()))
}

pub(crate) fn read_fasta_window(
    fasta_path: &Path,
    offset: u64,
    len: usize,
) -> Result<Vec<u8>, CacheError> {
    let mut file = File::open(fasta_path).map_err(|e| CacheError(e.to_string()))?;
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| CacheError(e.to_string()))?;
    let mut buf = vec![0_u8; len];
    file.read_exact(&mut buf)
        .map_err(|e| CacheError(e.to_string()))?;
    Ok(buf)
}
