use crate::IngestError;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

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

        let (attrs, duplicate_attr_keys) = parse_attributes(cols[8]);
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

fn parse_attributes(raw: &str) -> (BTreeMap<String, String>, BTreeSet<String>) {
    let mut out = BTreeMap::new();
    let mut dups = BTreeSet::new();
    for token in raw.split(';') {
        let t = token.trim();
        if t.is_empty() {
            continue;
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
    (out, dups)
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
}
