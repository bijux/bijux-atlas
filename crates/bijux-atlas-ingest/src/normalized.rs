// SPDX-License-Identifier: Apache-2.0

use crate::extract::{ExonRecord, GeneRecord, TranscriptRecord};
use crate::IngestError;
use bijux_atlas_core::canonical;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub const NORMALIZED_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NormalizedRecord {
    schema_version: u64,
    kind: String,
    record_id: String,
    seqid: String,
    start: u64,
    end: u64,
    payload: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReplayCounts {
    pub genes: u64,
    pub transcripts: u64,
    pub exons: u64,
}

pub fn write_normalized_jsonl_zst(
    out_path: &Path,
    genes: &[GeneRecord],
    transcripts: &[TranscriptRecord],
    exons: &[ExonRecord],
) -> Result<(), IngestError> {
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(|e| IngestError(e.to_string()))?;
    }
    let mut items = Vec::<NormalizedRecord>::new();
    for g in genes {
        items.push(NormalizedRecord {
            schema_version: NORMALIZED_SCHEMA_VERSION,
            kind: "gene".to_string(),
            record_id: format!("gene:{}", g.gene_id),
            seqid: g.seqid.clone(),
            start: g.start,
            end: g.end,
            payload: serde_json::to_value(g).map_err(|e| IngestError(e.to_string()))?,
        });
    }
    for t in transcripts {
        items.push(NormalizedRecord {
            schema_version: NORMALIZED_SCHEMA_VERSION,
            kind: "transcript".to_string(),
            record_id: format!("transcript:{}", t.transcript_id),
            seqid: t.seqid.clone(),
            start: t.start,
            end: t.end,
            payload: serde_json::to_value(t).map_err(|e| IngestError(e.to_string()))?,
        });
    }
    for e in exons {
        items.push(NormalizedRecord {
            schema_version: NORMALIZED_SCHEMA_VERSION,
            kind: "exon".to_string(),
            record_id: format!(
                "exon:{}:{}:{}:{}",
                e.transcript_id, e.exon_id, e.start, e.end
            ),
            seqid: e.seqid.clone(),
            start: e.start,
            end: e.end,
            payload: serde_json::to_value(e).map_err(|e| IngestError(e.to_string()))?,
        });
    }
    items.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then(a.seqid.cmp(&b.seqid))
            .then(a.start.cmp(&b.start))
            .then(a.end.cmp(&b.end))
            .then(a.record_id.cmp(&b.record_id))
    });

    let file = fs::File::create(out_path).map_err(|e| IngestError(e.to_string()))?;
    let mut encoder =
        zstd::stream::write::Encoder::new(file, 3).map_err(|e| IngestError(e.to_string()))?;
    for item in items {
        let mut line =
            canonical::stable_json_bytes(&item).map_err(|e| IngestError(e.to_string()))?;
        line.push(b'\n');
        encoder
            .write_all(&line)
            .map_err(|e| IngestError(e.to_string()))?;
    }
    let _ = encoder.finish().map_err(|e| IngestError(e.to_string()))?;
    Ok(())
}

pub fn replay_counts_from_normalized(path: &Path) -> Result<ReplayCounts, IngestError> {
    let file = fs::File::open(path).map_err(|e| IngestError(e.to_string()))?;
    let decoder = zstd::stream::read::Decoder::new(file).map_err(|e| IngestError(e.to_string()))?;
    let reader = BufReader::new(decoder);
    let mut counts = ReplayCounts::default();
    for line in reader.lines() {
        let line = line.map_err(|e| IngestError(e.to_string()))?;
        if line.trim().is_empty() {
            continue;
        }
        let rec: NormalizedRecord =
            serde_json::from_str(&line).map_err(|e| IngestError(e.to_string()))?;
        if rec.schema_version != NORMALIZED_SCHEMA_VERSION {
            return Err(IngestError(format!(
                "normalized schema version mismatch: expected {}, got {}",
                NORMALIZED_SCHEMA_VERSION, rec.schema_version
            )));
        }
        match rec.kind.as_str() {
            "gene" => counts.genes += 1,
            "transcript" => counts.transcripts += 1,
            "exon" => counts.exons += 1,
            other => return Err(IngestError(format!("unknown normalized kind: {other}"))),
        }
    }
    Ok(counts)
}

pub fn diff_normalized_record_ids(
    base_path: &Path,
    target_path: &Path,
) -> Result<(Vec<String>, Vec<String>), IngestError> {
    let base = read_record_ids(base_path)?;
    let target = read_record_ids(target_path)?;
    let removed = base.difference(&target).cloned().collect::<Vec<_>>();
    let added = target.difference(&base).cloned().collect::<Vec<_>>();
    Ok((removed, added))
}

fn read_record_ids(path: &Path) -> Result<BTreeSet<String>, IngestError> {
    let file = fs::File::open(path).map_err(|e| IngestError(e.to_string()))?;
    let decoder = zstd::stream::read::Decoder::new(file).map_err(|e| IngestError(e.to_string()))?;
    let reader = BufReader::new(decoder);
    let mut out = BTreeSet::new();
    for line in reader.lines() {
        let line = line.map_err(|e| IngestError(e.to_string()))?;
        if line.trim().is_empty() {
            continue;
        }
        let rec: NormalizedRecord =
            serde_json::from_str(&line).map_err(|e| IngestError(e.to_string()))?;
        out.insert(rec.record_id);
    }
    Ok(out)
}
