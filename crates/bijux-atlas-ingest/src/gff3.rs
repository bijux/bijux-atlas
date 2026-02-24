// SPDX-License-Identifier: Apache-2.0

use crate::IngestError;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use unicode_normalization::UnicodeNormalization;

const MAX_GFF3_LINE_BYTES: usize = 1_000_000;
const MAX_ATTR_TOKENS: usize = 4_096;

#[derive(Debug, Clone)]
#[allow(dead_code)] // ATLAS-EXC-0001
pub struct Gff3Record {
    pub line: usize,
    pub seqid: String,
    pub feature_type: String,
    pub strand: String,
    pub phase: String,
    pub start: u64,
    pub end: u64,
    pub attrs: BTreeMap<String, String>,
    pub duplicate_attr_keys: BTreeSet<String>,
    pub raw_line: String,
}

pub fn parse_gff3_records(path: &Path) -> Result<Vec<Gff3Record>, IngestError> {
    let file = fs::File::open(path).map_err(|e| IngestError(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut out = Vec::new();

    for (line_idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| IngestError(e.to_string()))?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.len() > MAX_GFF3_LINE_BYTES {
            return Err(IngestError(format!(
                "gff3 line exceeds max byte length {MAX_GFF3_LINE_BYTES}"
            )));
        }

        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() != 9 {
            return Err(IngestError(format!(
                "invalid GFF3 row (expected 9 columns): {line}"
            )));
        }

        let start: u64 = cols[3]
            .parse()
            .map_err(|_| IngestError(format!("invalid start coordinate: {}", cols[3])))?;
        let end: u64 = cols[4]
            .parse()
            .map_err(|_| IngestError(format!("invalid end coordinate: {}", cols[4])))?;
        if start == 0 || end < start {
            return Err(IngestError(format!(
                "invalid coordinate span: {start}-{end}"
            )));
        }

        let seqid = cols[0].trim().to_string();
        if seqid.is_empty() {
            return Err(IngestError(format!(
                "GFF3_MISSING_REQUIRED_FIELD line={} field=seqid sample={line}",
                line_idx + 1
            )));
        }
        let feature_type = cols[2].trim().to_string();
        if feature_type.is_empty() {
            return Err(IngestError(format!(
                "GFF3_MISSING_REQUIRED_FIELD line={} field=feature_type sample={line}",
                line_idx + 1
            )));
        }
        let strand = cols[6].trim().to_string();
        if !matches!(strand.as_str(), "+" | "-" | ".") {
            return Err(IngestError(format!(
                "GFF3_INVALID_STRAND line={} strand={} sample={line}",
                line_idx + 1,
                strand
            )));
        }
        let phase = cols[7].trim().to_string();
        if feature_type == "CDS" && !matches!(phase.as_str(), "0" | "1" | "2" | ".") {
            return Err(IngestError(format!(
                "GFF3_INVALID_PHASE line={} phase={} sample={line}",
                line_idx + 1,
                phase
            )));
        }

        let (attrs, duplicate_attr_keys) = parse_attributes(cols[8], line_idx + 1)?;
        out.push(Gff3Record {
            line: line_idx + 1,
            seqid,
            feature_type,
            strand,
            phase,
            start,
            end,
            attrs,
            duplicate_attr_keys,
            raw_line: line,
        });
    }

    Ok(out)
}

fn decode_attr_value(raw: &str) -> String {
    let trimmed = raw.trim().trim_matches('"');
    percent_decode(trimmed)
}

fn is_id_key(key: &str) -> bool {
    matches!(
        key,
        "ID" | "Parent" | "gene_id" | "transcript_id" | "transcriptId" | "protein_id" | "exon_id"
    )
}

fn is_name_key(key: &str) -> bool {
    matches!(key, "Name" | "gene_name" | "description")
}

fn has_forbidden_hidden_characters(value: &str) -> bool {
    value.chars().any(|c| {
        c.is_control()
            || matches!(
                c,
                '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FEFF}' | '\u{00AD}'
            )
    })
}

fn normalize_attribute_value(key: &str, value: &str, line: usize) -> Result<String, IngestError> {
    let mut normalized = value.nfc().collect::<String>();
    normalized = normalized.trim().to_string();
    if has_forbidden_hidden_characters(&normalized) {
        return Err(IngestError(format!(
            "GFF3_FORBIDDEN_HIDDEN_CHAR line={} key={}",
            line, key
        )));
    }
    if is_id_key(key) && normalized.chars().any(char::is_whitespace) {
        return Err(IngestError(format!(
            "GFF3_INVALID_ID_WHITESPACE line={} key={}",
            line, key
        )));
    }
    if is_name_key(key) {
        normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
    }
    Ok(normalized)
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let h1 = bytes[i + 1] as char;
            let h2 = bytes[i + 2] as char;
            if let (Some(a), Some(b)) = (h1.to_digit(16), h2.to_digit(16)) {
                out.push(((a << 4) + b) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn parse_attributes(
    raw: &str,
    line: usize,
) -> Result<(BTreeMap<String, String>, BTreeSet<String>), IngestError> {
    let mut out = BTreeMap::new();
    let mut dups = BTreeSet::new();
    let mut token_count = 0usize;
    for token in raw.split(';') {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }
        token_count += 1;
        if token_count > MAX_ATTR_TOKENS {
            return Err(IngestError(format!(
                "gff3 attribute token count exceeds max {MAX_ATTR_TOKENS}"
            )));
        }
        if let Some((k, v)) = t.split_once('=') {
            let key = k.trim().to_string();
            let value = normalize_attribute_value(&key, &decode_attr_value(v), line)?;
            if out.contains_key(&key) {
                dups.insert(key.clone());
            }
            out.insert(key, value);
        }
    }
    Ok((out, dups))
}

#[cfg(test)]
mod tests {
    use super::parse_gff3_records;
    use proptest::prelude::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn parses_attributes_with_quoting_percent_and_duplicates() {
        let tmp = tempdir().expect("tempdir");
        let gff = tmp.path().join("x.gff3");
        fs::write(
            &gff,
            "chr1\tsrc\tgene\t1\t10\t.\t+\t.\tID=g1;Name=%22Gene%201%22;Name=Gene_1;description=Tumor%20protein\n",
        )
        .expect("write gff3");
        let rows = parse_gff3_records(&gff).expect("parse gff3");
        assert_eq!(rows.len(), 1);
        let rec = &rows[0];
        assert_eq!(rec.attrs.get("ID").map(String::as_str), Some("g1"));
        assert_eq!(rec.attrs.get("Name").map(String::as_str), Some("Gene_1"));
        assert_eq!(
            rec.attrs.get("description").map(String::as_str),
            Some("Tumor protein")
        );
        assert!(rec.duplicate_attr_keys.contains("Name"));
    }

    #[test]
    fn attribute_weirdness_corpus_parses_deterministically() {
        let corpus = [
            "ID=g1;Name=\"A%20B\";gene_id=ENSG000001",
            "ID=g2;Name=G%5F2;description=alpha%3Bbeta",
            "ID=g3;Name= spaced ;biotype=protein_coding;;",
            "ID=g4;Name=\"quoted=value\";Parent=g1,g2",
            "ID=g5;Name=x;Name=y;Name=z",
        ];
        for (i, attrs) in corpus.iter().enumerate() {
            let tmp = tempdir().expect("tempdir");
            let gff = tmp.path().join("x.gff3");
            let row = format!("chr1\tsrc\tgene\t1\t10\t.\t+\t.\t{attrs}\n");
            fs::write(&gff, row).expect("write gff3");
            let rows = parse_gff3_records(&gff).expect("parse corpus entry");
            assert_eq!(rows.len(), 1, "entry {i} should parse");
            assert!(rows[0].attrs.contains_key("ID"));
        }
    }

    #[test]
    fn streaming_parser_handles_large_input_rows() {
        let tmp = tempdir().expect("tempdir");
        let gff = tmp.path().join("large.gff3");
        let mut buf = String::new();
        for i in 0..25_000_u64 {
            let line = format!(
                "chr1\tsrc\tgene\t{}\t{}\t.\t+\t.\tID=g{i};Name=Gene_{i};biotype=protein_coding\n",
                1 + i * 10,
                10 + i * 10
            );
            buf.push_str(&line);
        }
        fs::write(&gff, buf).expect("write large gff3");
        let rows = parse_gff3_records(&gff).expect("parse large gff3");
        assert_eq!(rows.len(), 25_000);
    }

    #[test]
    fn parser_rejects_attribute_token_explosion() {
        let tmp = tempdir().expect("tempdir");
        let gff = tmp.path().join("x.gff3");
        let attrs = (0..(super::MAX_ATTR_TOKENS + 1))
            .map(|i| format!("k{i}=v{i}"))
            .collect::<Vec<_>>()
            .join(";");
        let row = format!("chr1\tsrc\tgene\t1\t10\t.\t+\t.\t{attrs}\n");
        fs::write(&gff, row).expect("write gff3");
        let err = parse_gff3_records(&gff).expect_err("token explosion must fail");
        assert!(err.0.contains("attribute token count exceeds"));
    }

    #[test]
    fn parser_rejects_missing_required_core_fields() {
        let tmp = tempdir().expect("tempdir");
        let gff = tmp.path().join("x.gff3");
        fs::write(&gff, "\tsrc\tgene\t1\t10\t.\t+\t.\tID=g1\n").expect("write gff3");
        let err = parse_gff3_records(&gff).expect_err("missing seqid must fail");
        assert!(err.0.contains("GFF3_MISSING_REQUIRED_FIELD"));
    }

    #[test]
    fn fuzzish_attribute_order_spacing_is_stable() {
        let tmp = tempdir().expect("tempdir");
        let gff = tmp.path().join("fuzz.gff3");
        let mut lines = Vec::new();
        for i in 0..128 {
            let mut attrs = [
                format!("ID=g{i}"),
                "Name= Gene%20One ".to_string(),
                "gene_name=GENE_ONE".to_string(),
                "biotype=protein_coding".to_string(),
            ];
            let len = attrs.len();
            attrs.rotate_left(i % len);
            let row = format!(
                "chr1\tsrc\tgene\t{}\t{}\t.\t+\t.\t{}\n",
                1 + i as u64,
                10 + i as u64,
                attrs.join(" ; ")
            );
            lines.push(row);
        }
        fs::write(&gff, lines.concat()).expect("write fuzz");
        let rows = parse_gff3_records(&gff).expect("parse fuzz");
        assert_eq!(rows.len(), 128);
        for rec in rows {
            assert_eq!(
                rec.attrs.get("biotype").map(String::as_str),
                Some("protein_coding")
            );
            assert!(rec.attrs.contains_key("ID"));
        }
    }

    #[test]
    fn parser_rejects_hidden_characters_in_identifiers() {
        let tmp = tempdir().expect("tempdir");
        let gff = tmp.path().join("hidden.gff3");
        fs::write(
            &gff,
            "chr1\tsrc\tgene\t1\t10\t.\t+\t.\tID=g\u{200B}1;Name=Gene 1\n",
        )
        .expect("write gff3");
        let err = parse_gff3_records(&gff).expect_err("hidden char must fail");
        assert!(err.0.contains("GFF3_FORBIDDEN_HIDDEN_CHAR"));
    }

    proptest! {
        #[test]
        fn id_values_reject_whitespace_and_hidden_chars(seed in "[A-Za-z0-9_\\-]{1,32}") {
            let tmp = tempdir().expect("tempdir");
            let gff = tmp.path().join("id.gff3");
            let bad = format!(" {seed}\u{200B}");
            let row = format!("chr1\tsrc\tgene\t1\t10\t.\t+\t.\tID={bad};Name=Gene\n");
            fs::write(&gff, row).expect("write gff3");
            let err = parse_gff3_records(&gff).expect_err("id normalization guard must fail");
            prop_assert!(
                err.0.contains("GFF3_FORBIDDEN_HIDDEN_CHAR")
                    || err.0.contains("GFF3_INVALID_ID_WHITESPACE")
            );
        }
    }
}
