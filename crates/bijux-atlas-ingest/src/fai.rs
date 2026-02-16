use crate::IngestError;
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn read_fai_contig_lengths(path: &Path) -> Result<BTreeMap<String, u64>, IngestError> {
    let file = fs::File::open(path).map_err(|e| IngestError(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut out = BTreeMap::new();
    for line in reader.lines() {
        let line = line.map_err(|e| IngestError(e.to_string()))?;
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 2 {
            return Err(IngestError(format!("invalid FAI line: {line}")));
        }
        let len: u64 = cols[1]
            .parse()
            .map_err(|_| IngestError(format!("invalid FAI contig length: {}", cols[1])))?;
        out.insert(cols[0].to_string(), len);
    }
    Ok(out)
}
