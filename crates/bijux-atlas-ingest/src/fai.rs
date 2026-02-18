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

pub fn write_fai_from_fasta(fasta_path: &Path, fai_path: &Path) -> Result<(), IngestError> {
    let lengths = read_fasta_contig_lengths(fasta_path)?;
    let mut out = String::new();
    for (name, len) in lengths {
        out.push_str(&format!("{name}\t{len}\n"));
    }
    fs::write(fai_path, out).map_err(|e| IngestError(e.to_string()))
}

pub fn read_fasta_contig_lengths(path: &Path) -> Result<BTreeMap<String, u64>, IngestError> {
    let file = fs::File::open(path).map_err(|e| IngestError(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut out = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in reader.lines() {
        let line = line.map_err(|e| IngestError(e.to_string()))?;
        if let Some(rest) = line.strip_prefix('>') {
            let name = rest.split_whitespace().next().unwrap_or("").trim();
            if name.is_empty() {
                return Err(IngestError("FASTA header missing contig name".to_string()));
            }
            current = Some(name.to_string());
            out.entry(name.to_string()).or_insert(0);
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        let Some(contig) = current.as_ref() else {
            return Err(IngestError(
                "FASTA sequence line seen before header".to_string(),
            ));
        };
        let delta = line
            .as_bytes()
            .iter()
            .filter(|b| !b.is_ascii_whitespace())
            .count() as u64;
        *out.entry(contig.clone()).or_insert(0) += delta;
    }
    Ok(out)
}

#[derive(Debug, Clone)]
pub struct ContigStats {
    pub length: u64,
    pub gc_fraction: Option<f64>,
    pub n_fraction: Option<f64>,
}

pub fn read_fasta_contig_stats(
    path: &Path,
    compute_fractions: bool,
    max_total_bases: u64,
) -> Result<BTreeMap<String, ContigStats>, IngestError> {
    let file = fs::File::open(path).map_err(|e| IngestError(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut out: BTreeMap<String, (u64, u64, u64)> = BTreeMap::new(); // len, gc, n
    let mut total_bases: u64 = 0;
    let mut current: Option<String> = None;
    for line in reader.lines() {
        let line = line.map_err(|e| IngestError(e.to_string()))?;
        if let Some(rest) = line.strip_prefix('>') {
            let name = rest.split_whitespace().next().unwrap_or("").trim();
            if name.is_empty() {
                return Err(IngestError("FASTA header missing contig name".to_string()));
            }
            current = Some(name.to_string());
            out.entry(name.to_string()).or_insert((0, 0, 0));
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        let Some(contig) = current.as_ref() else {
            return Err(IngestError(
                "FASTA sequence line seen before header".to_string(),
            ));
        };
        let e = out.entry(contig.clone()).or_insert((0, 0, 0));
        for b in line.as_bytes() {
            if b.is_ascii_whitespace() {
                continue;
            }
            total_bases = total_bases.saturating_add(1);
            if max_total_bases > 0 && total_bases > max_total_bases {
                return Err(IngestError(format!(
                    "FASTA scanning memory guardrail exceeded: {total_bases}>{max_total_bases}"
                )));
            }
            e.0 += 1;
            if compute_fractions {
                match b.to_ascii_uppercase() {
                    b'G' | b'C' => e.1 += 1,
                    b'N' => e.2 += 1,
                    _ => {}
                }
            }
        }
    }
    let stats = out
        .into_iter()
        .map(|(k, (len, gc, n))| {
            let (gc_fraction, n_fraction) = if compute_fractions && len > 0 {
                (Some(gc as f64 / len as f64), Some(n as f64 / len as f64))
            } else {
                (None, None)
            };
            (
                k,
                ContigStats {
                    length: len,
                    gc_fraction,
                    n_fraction,
                },
            )
        })
        .collect();
    Ok(stats)
}
