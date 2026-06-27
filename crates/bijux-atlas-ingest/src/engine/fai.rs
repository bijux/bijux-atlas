// SPDX-License-Identifier: Apache-2.0

use super::IngestError;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

const MAX_FASTA_CONTIG_NAME_LEN: usize = 255;

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
        let contig = validate_fasta_contig_name(cols[0].trim(), 0)?;
        let len: u64 = cols[1]
            .parse()
            .map_err(|_| IngestError(format!("invalid FAI contig length: {}", cols[1])))?;
        if let Some(previous) = out.insert(contig.clone(), len) {
            if previous != len {
                return Err(IngestError(format!(
                    "conflicting FAI duplicate contig length for {contig}: {previous} vs {len}"
                )));
            }
            return Err(IngestError(format!(
                "duplicate FAI contig declaration for {contig}"
            )));
        }
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
            let name = validate_fasta_contig_name(
                rest.split_whitespace().next().unwrap_or("").trim(),
                out.len() + 1,
            )?;
            if out.contains_key(&name) {
                return Err(IngestError(format!(
                    "duplicate FASTA contig header detected: {name}"
                )));
            }
            current = Some(name.clone());
            out.insert(name, 0);
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

#[derive(Debug, Clone, Serialize)]
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
            let name = validate_fasta_contig_name(
                rest.split_whitespace().next().unwrap_or("").trim(),
                out.len() + 1,
            )?;
            if out.contains_key(&name) {
                return Err(IngestError(format!(
                    "duplicate FASTA contig header detected: {name}"
                )));
            }
            current = Some(name.clone());
            out.insert(name, (0, 0, 0));
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

fn validate_fasta_contig_name(name: &str, header_index: usize) -> Result<String, IngestError> {
    if name.trim().is_empty() {
        return Err(IngestError("FASTA header missing contig name".to_string()));
    }
    let value = name.trim();
    if value.len() > MAX_FASTA_CONTIG_NAME_LEN {
        return Err(IngestError(format!(
            "FASTA header contig name exceeds {} bytes: {}",
            MAX_FASTA_CONTIG_NAME_LEN, value
        )));
    }
    if value.chars().any(char::is_control)
        || value.chars().any(|c| {
            matches!(
                c,
                '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FEFF}' | '\u{00AD}'
            )
        })
    {
        return Err(IngestError(format!(
            "FASTA header contig name contains forbidden control characters at header {}",
            header_index
        )));
    }
    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::{read_fasta_contig_lengths, read_fasta_contig_stats};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn fasta_parser_rejects_duplicate_headers() {
        let tmp = tempdir().expect("tmp");
        let fasta = tmp.path().join("dup.fa");
        fs::write(&fasta, ">chr1\nACGT\n>chr1\nTGCA\n").expect("write fasta");
        let err = read_fasta_contig_lengths(&fasta).expect_err("duplicate header must fail");
        assert!(err.0.contains("duplicate FASTA contig header"));
    }

    #[test]
    fn fasta_parser_rejects_forbidden_control_header_characters() {
        let tmp = tempdir().expect("tmp");
        let fasta = tmp.path().join("hidden.fa");
        fs::write(&fasta, ">\u{200B}chr1\nACGT\n").expect("write fasta");
        let err = read_fasta_contig_stats(&fasta, false, 10_000).expect_err("hidden char");
        assert!(err.0.contains("forbidden control characters"));
    }
}
