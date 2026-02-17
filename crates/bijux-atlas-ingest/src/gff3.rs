use crate::IngestError;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

const MAX_GFF3_LINE_BYTES: usize = 1_000_000;
const MAX_ATTR_TOKENS: usize = 4_096;

#[derive(Debug, Clone)]
pub struct Gff3Record {
    pub seqid: String,
    pub feature_type: String,
    pub start: u64,
    pub end: u64,
    pub attrs: BTreeMap<String, String>,
    pub duplicate_attr_keys: BTreeSet<String>,
}

pub fn parse_gff3_records(path: &Path) -> Result<Vec<Gff3Record>, IngestError> {
    let file = fs::File::open(path).map_err(|e| IngestError(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut out = Vec::new();

    for line in reader.lines() {
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

        let (attrs, duplicate_attr_keys) = parse_attributes(cols[8])?;
        out.push(Gff3Record {
            seqid: cols[0].trim().to_string(),
            feature_type: cols[2].trim().to_string(),
            start,
            end,
            attrs,
            duplicate_attr_keys,
        });
    }

    Ok(out)
}

fn decode_attr_value(raw: &str) -> String {
    let trimmed = raw.trim().trim_matches('"');
    percent_decode(trimmed)
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
            let value = decode_attr_value(v);
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
}
