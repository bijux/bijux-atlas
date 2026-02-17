use crate::gff3::Gff3Record;
use crate::{IngestError, IngestOptions};
use bijux_atlas_core::canonical;
use bijux_atlas_model::{DuplicateGeneIdPolicy, IngestAnomalyReport, StrictnessMode};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone)]
pub struct GeneRecord {
    pub gene_id: String,
    pub gene_name: String,
    pub biotype: String,
    pub seqid: String,
    pub start: u64,
    pub end: u64,
    pub transcript_count: u64,
    pub exon_count: u64,
    pub total_exon_span: u64,
    pub cds_present: bool,
    pub sequence_length: u64,
}

#[derive(Debug, Clone)]
pub struct TranscriptRecord {
    pub transcript_id: String,
    pub parent_gene_id: String,
    pub transcript_type: String,
    pub biotype: Option<String>,
    pub seqid: String,
    pub start: u64,
    pub end: u64,
    pub exon_count: u64,
    pub total_exon_span: u64,
    pub cds_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentErrorClass {
    MissingParentAttribute,
    MultipleParents,
    MissingReferencedParent,
}

#[derive(Debug, Clone)]
pub struct ExtractResult {
    pub gene_rows: Vec<GeneRecord>,
    pub transcript_rows: Vec<TranscriptRecord>,
    pub anomaly: IngestAnomalyReport,
    pub biotype_distribution: BTreeMap<String, u64>,
    pub contig_distribution: BTreeMap<String, u64>,
}

pub fn extract_gene_rows(
    records: Vec<Gff3Record>,
    contig_lengths: &BTreeMap<String, u64>,
    opts: &IngestOptions,
) -> Result<ExtractResult, IngestError> {
    let mut genes: HashMap<String, Vec<GeneRecord>> = HashMap::new();
    let mut transcript_parents: Vec<(String, ParentErrorClass)> = Vec::new();
    let mut transcript_rows_pending: Vec<TranscriptRecord> = Vec::new();
    let mut transcript_exon_counts: HashMap<String, u64> = HashMap::new();
    let mut transcript_exon_span: HashMap<String, u64> = HashMap::new();
    let mut transcript_has_cds: HashMap<String, bool> = HashMap::new();
    let mut anomaly = IngestAnomalyReport::default();

    let mut seen_feature_ids: HashMap<String, String> = HashMap::new();

    for rec in records {
        let seqid = opts.seqid_policy.normalize(&rec.seqid);

        for dup in rec.duplicate_attr_keys {
            anomaly.overlapping_ids.push(dup);
        }

        if let Some(fid) = rec.attrs.get("ID") {
            if let Some(previous_kind) = seen_feature_ids.get(fid) {
                if previous_kind != &rec.feature_type {
                    anomaly.overlapping_ids.push(fid.clone());
                }
            } else {
                seen_feature_ids.insert(fid.clone(), rec.feature_type.clone());
            }
        }

        if rec.feature_type == "gene" {
            let gff3_id = rec
                .attrs
                .get("ID")
                .cloned()
                .ok_or_else(|| IngestError("gene feature missing ID attribute".to_string()))?;
            let gene_id = opts
                .gene_identifier_policy
                .resolve(
                    &rec.attrs,
                    &gff3_id,
                    matches!(opts.strictness, StrictnessMode::Strict),
                )
                .map_err(|e| IngestError(e.to_string()))?;

            let Some(contig_len) = contig_lengths.get(&seqid) else {
                anomaly.unknown_contigs.push(seqid.clone());
                if matches!(opts.strictness, StrictnessMode::Strict) {
                    return Err(IngestError(format!("contig not found in FAI: {seqid}")));
                }
                continue;
            };
            if rec.end > *contig_len {
                let msg = format!(
                    "gene {gene_id} coordinate end {} exceeds contig {seqid} length {contig_len}",
                    rec.end
                );
                if matches!(opts.strictness, StrictnessMode::Strict) {
                    return Err(IngestError(msg));
                }
                anomaly.unknown_contigs.push(seqid.clone());
                continue;
            }

            let record = GeneRecord {
                gene_id: gene_id.clone(),
                gene_name: opts.gene_name_policy.resolve(&rec.attrs, &gene_id),
                biotype: opts.biotype_policy.resolve(&rec.attrs),
                seqid,
                start: rec.start,
                end: rec.end,
                transcript_count: 0,
                exon_count: 0,
                total_exon_span: 0,
                cds_present: false,
                sequence_length: rec.end - rec.start + 1,
            };
            genes.entry(gene_id).or_default().push(record);
        } else if opts.transcript_type_policy.accepts(&rec.feature_type) {
            let tx_id = rec
                .attrs
                .get("ID")
                .cloned()
                .unwrap_or_else(|| "<missing transcript id>".to_string());
            let Some(parent_attr) = rec.attrs.get("Parent") else {
                transcript_parents.push((tx_id.clone(), ParentErrorClass::MissingParentAttribute));
                anomaly.missing_transcript_parents.push(tx_id);
                if matches!(opts.strictness, StrictnessMode::Strict) {
                    return Err(IngestError(
                        "transcript feature missing Parent attribute".to_string(),
                    ));
                }
                continue;
            };

            let parents: Vec<String> = parent_attr
                .split(',')
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .map(ToString::to_string)
                .collect();

            if parents.len() > 1 {
                anomaly.multiple_parent_transcripts.push(tx_id.clone());
                if matches!(opts.strictness, StrictnessMode::Strict) {
                    return Err(IngestError(format!(
                        "transcript {tx_id} has multiple Parent references"
                    )));
                }
                for p in parents {
                    transcript_parents.push((p, ParentErrorClass::MultipleParents));
                }
            } else if let Some(p) = parents.into_iter().next() {
                transcript_parents.push((p.clone(), ParentErrorClass::MissingReferencedParent));
                transcript_rows_pending.push(TranscriptRecord {
                    transcript_id: tx_id,
                    parent_gene_id: p,
                    transcript_type: rec.feature_type.clone(),
                    biotype: rec
                        .attrs
                        .get("transcript_biotype")
                        .or_else(|| rec.attrs.get("biotype"))
                        .or_else(|| rec.attrs.get("gene_biotype"))
                        .cloned(),
                    seqid,
                    start: rec.start,
                    end: rec.end,
                    exon_count: 0,
                    total_exon_span: 0,
                    cds_present: false,
                });
            }
        } else if rec.feature_type == "exon" || rec.feature_type == "CDS" {
            let Some(parent_attr) = rec.attrs.get("Parent") else {
                continue;
            };
            let parents = parent_attr
                .split(',')
                .map(str::trim)
                .filter(|x| !x.is_empty());
            for tx_id in parents {
                if rec.feature_type == "exon" {
                    *transcript_exon_counts.entry(tx_id.to_string()).or_insert(0) += 1;
                    *transcript_exon_span.entry(tx_id.to_string()).or_insert(0) +=
                        rec.end.saturating_sub(rec.start) + 1;
                } else if rec.feature_type == "CDS" {
                    transcript_has_cds.insert(tx_id.to_string(), true);
                }
            }
        }
    }

    let mut deduped: HashMap<String, GeneRecord> = HashMap::new();
    let mut keys: Vec<String> = genes.keys().cloned().collect();
    keys.sort();
    for key in keys {
        let Some(mut candidates) = genes.remove(&key) else {
            continue;
        };
        if candidates.len() > 1 {
            anomaly.duplicate_gene_ids.push(key.clone());
            match opts.duplicate_gene_id_policy {
                DuplicateGeneIdPolicy::Fail => {
                    if matches!(opts.strictness, StrictnessMode::Strict) {
                        return Err(IngestError(format!("duplicate gene_id: {key}")));
                    }
                }
                DuplicateGeneIdPolicy::DedupeKeepLexicographicallySmallest => {
                    candidates.sort_by(|a, b| {
                        a.seqid
                            .cmp(&b.seqid)
                            .then(a.start.cmp(&b.start))
                            .then(a.end.cmp(&b.end))
                            .then(a.gene_name.cmp(&b.gene_name))
                            .then(a.biotype.cmp(&b.biotype))
                    });
                }
                _ => {
                    if matches!(opts.strictness, StrictnessMode::Strict) {
                        return Err(IngestError(
                            "unsupported duplicate gene_id policy variant".to_string(),
                        ));
                    }
                }
            }
        }
        if let Some(first) = candidates.into_iter().next() {
            deduped.insert(key, first);
        }
    }

    for (parent, class) in transcript_parents {
        if let Some(gene) = deduped.get_mut(&parent) {
            gene.transcript_count += 1;
            if class == ParentErrorClass::MultipleParents {
                // valid but tracked as anomaly for QC visibility
                anomaly
                    .missing_parents
                    .push(format!("multiple_parent:{parent}"));
            }
        } else {
            anomaly.missing_parents.push(parent.clone());
            anomaly.missing_transcript_parents.push(parent.clone());
            if matches!(opts.strictness, StrictnessMode::Strict) {
                return Err(IngestError(format!(
                    "transcript parent {parent} does not reference a known gene"
                )));
            }
        }
    }

    for tx in &mut transcript_rows_pending {
        tx.exon_count = transcript_exon_counts
            .get(&tx.transcript_id)
            .copied()
            .unwrap_or(0);
        tx.total_exon_span = transcript_exon_span
            .get(&tx.transcript_id)
            .copied()
            .unwrap_or(0);
        tx.cds_present = transcript_has_cds
            .get(&tx.transcript_id)
            .copied()
            .unwrap_or(false);
    }
    transcript_rows_pending.retain(|tx| deduped.contains_key(&tx.parent_gene_id));
    transcript_rows_pending.sort_by(|a, b| {
        a.seqid
            .cmp(&b.seqid)
            .then(a.start.cmp(&b.start))
            .then(a.transcript_id.cmp(&b.transcript_id))
    });

    for tx in &transcript_rows_pending {
        if let Some(gene) = deduped.get_mut(&tx.parent_gene_id) {
            gene.exon_count += tx.exon_count;
            gene.total_exon_span += tx.total_exon_span;
            gene.cds_present = gene.cds_present || tx.cds_present;
        }
    }

    anomaly.missing_parents = canonical::stable_sort_by_key(anomaly.missing_parents, |x| x.clone());
    anomaly.missing_transcript_parents =
        canonical::stable_sort_by_key(anomaly.missing_transcript_parents, |x| x.clone());
    anomaly.multiple_parent_transcripts =
        canonical::stable_sort_by_key(anomaly.multiple_parent_transcripts, |x| x.clone());
    anomaly.unknown_contigs = canonical::stable_sort_by_key(anomaly.unknown_contigs, |x| x.clone());
    anomaly.overlapping_ids = canonical::stable_sort_by_key(anomaly.overlapping_ids, |x| x.clone());
    anomaly.duplicate_gene_ids =
        canonical::stable_sort_by_key(anomaly.duplicate_gene_ids, |x| x.clone());
    anomaly.missing_parents.dedup();
    anomaly.missing_transcript_parents.dedup();
    anomaly.multiple_parent_transcripts.dedup();
    anomaly.unknown_contigs.dedup();
    anomaly.overlapping_ids.dedup();
    anomaly.duplicate_gene_ids.dedup();

    let mut gene_rows: Vec<GeneRecord> = deduped.into_values().collect();
    gene_rows.sort_by(|a, b| {
        a.seqid
            .cmp(&b.seqid)
            .then(a.start.cmp(&b.start))
            .then(a.end.cmp(&b.end))
            .then(a.gene_id.cmp(&b.gene_id))
    });

    let mut biotype_distribution: BTreeMap<String, u64> = BTreeMap::new();
    let mut contig_distribution: BTreeMap<String, u64> = BTreeMap::new();
    for g in &gene_rows {
        *biotype_distribution.entry(g.biotype.clone()).or_insert(0) += 1;
        *contig_distribution.entry(g.seqid.clone()).or_insert(0) += 1;
    }

    Ok(ExtractResult {
        gene_rows,
        transcript_rows: transcript_rows_pending,
        anomaly,
        biotype_distribution,
        contig_distribution,
    })
}

pub fn parallelism_policy(max_threads: usize) -> Result<usize, IngestError> {
    if max_threads == 0 {
        return Err(IngestError("max_threads must be >= 1".to_string()));
    }
    // Determinism-first: current implementation is single-threaded transform.
    Ok(1)
}
