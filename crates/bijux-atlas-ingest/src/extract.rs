use crate::gff3::Gff3Record;
use crate::{IngestError, IngestOptions};
use bijux_atlas_core::canonical;
use bijux_atlas_model::{
    DuplicateGeneIdPolicy, DuplicateTranscriptIdPolicy, FeatureIdUniquenessPolicy,
    GeneIdentifierPolicy, IngestAnomalyReport, IngestRejection, StrictnessMode,
    UnknownFeaturePolicy,
};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
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
    let mut child_parent_refs: Vec<String> = Vec::new();
    let mut normalized_seqid_sources: HashMap<String, HashSet<String>> = HashMap::new();
    let parent_cycles = detect_parent_cycles(&records);
    if !parent_cycles.is_empty() {
        anomaly.parent_cycles = parent_cycles;
        if matches!(opts.strictness, StrictnessMode::Strict) {
            return Err(IngestError(
                "cyclic Parent graph detected in GFF3 features".to_string(),
            ));
        }
    }

    for rec in records {
        let seqid = opts.seqid_policy.normalize(&rec.seqid);
        normalized_seqid_sources
            .entry(seqid.clone())
            .or_default()
            .insert(rec.seqid.clone());
        if opts.reject_normalized_seqid_collisions
            && normalized_seqid_sources
                .get(&seqid)
                .map(|s| s.len() > 1)
                .unwrap_or(false)
            && matches!(opts.strictness, StrictnessMode::Strict)
        {
            let sources = normalized_seqid_sources
                .get(&seqid)
                .cloned()
                .unwrap_or_default();
            return Err(IngestError(format!(
                "GFF3_SEQID_COLLISION line={} canonical={} sources={:?}",
                rec.line, seqid, sources
            )));
        }

        for dup in &rec.duplicate_attr_keys {
            anomaly.overlapping_ids.push(dup.clone());
        }

        if let Some(fid_raw) = rec.attrs.get("ID") {
            let mut fid = fid_raw.clone();
            if matches!(
                opts.feature_id_uniqueness_policy,
                FeatureIdUniquenessPolicy::NormalizeAsciiLowercaseReject
            ) {
                fid = fid.to_ascii_lowercase();
            }
            let key = if matches!(
                opts.feature_id_uniqueness_policy,
                FeatureIdUniquenessPolicy::NamespaceByFeatureType
            ) {
                format!("{}::{fid}", rec.feature_type)
            } else {
                fid.clone()
            };
            if let Some(previous_kind) = seen_feature_ids.get(&key) {
                anomaly.overlapping_ids.push(fid_raw.clone());
                let reject_on_duplicate = !matches!(
                    opts.feature_id_uniqueness_policy,
                    FeatureIdUniquenessPolicy::NamespaceByFeatureType
                ) && rec.feature_type != "gene";
                if reject_on_duplicate
                    && matches!(opts.strictness, StrictnessMode::Strict)
                {
                    return Err(IngestError(format!(
                        "duplicate feature ID detected: {fid_raw} ({previous_kind}, {})",
                        rec.feature_type
                    )));
                }
            } else {
                seen_feature_ids.insert(key, rec.feature_type.clone());
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
            if used_gene_id_fallback(&rec.attrs, opts) {
                anomaly
                    .attribute_fallbacks
                    .push(format!("gene_id_fallback:{gff3_id}"));
            }

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
            if record.gene_name == gene_id {
                anomaly
                    .attribute_fallbacks
                    .push(format!("gene_name_fallback:{gene_id}"));
            }
            if record.biotype == opts.biotype_policy.unknown_value {
                anomaly
                    .attribute_fallbacks
                    .push(format!("biotype_fallback:{gene_id}"));
            }
            genes.entry(gene_id).or_default().push(record);
        } else if opts.transcript_type_policy.accepts(&rec.feature_type) {
            let Some(tx_id) = opts.transcript_id_policy.resolve(&rec.attrs) else {
                let missing_key = format!(
                    "missing transcript id for {}:{}-{}",
                    seqid, rec.start, rec.end
                );
                anomaly.missing_required_fields.push(missing_key.clone());
                anomaly.rejections.push(IngestRejection::new(
                    rec.line,
                    "GFF3_MISSING_TRANSCRIPT_ID".to_string(),
                    rec.raw_line.clone(),
                ));
                if matches!(opts.strictness, StrictnessMode::Strict) {
                    return Err(IngestError(missing_key));
                }
                continue;
            };
            let Some(parent_attr) = rec.attrs.get("Parent") else {
                transcript_parents.push((tx_id.clone(), ParentErrorClass::MissingParentAttribute));
                anomaly.missing_transcript_parents.push(tx_id);
                anomaly.rejections.push(IngestRejection::new(
                    rec.line,
                    "GFF3_MISSING_PARENT".to_string(),
                    rec.raw_line.clone(),
                ));
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
                anomaly.rejections.push(IngestRejection::new(
                    rec.line,
                    "GFF3_MULTI_PARENT_TRANSCRIPT".to_string(),
                    rec.raw_line.clone(),
                ));
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
                anomaly.missing_required_fields.push(format!(
                    "{} missing Parent at {}:{}-{}",
                    rec.feature_type, seqid, rec.start, rec.end
                ));
                anomaly.rejections.push(IngestRejection::new(
                    rec.line,
                    "GFF3_MISSING_PARENT".to_string(),
                    rec.raw_line.clone(),
                ));
                if matches!(opts.strictness, StrictnessMode::Strict) {
                    return Err(IngestError(format!(
                        "{} feature missing Parent attribute",
                        rec.feature_type
                    )));
                }
                continue;
            };
            let parents = parent_attr
                .split(',')
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            if parents.len() > 1 && matches!(opts.strictness, StrictnessMode::Strict) {
                return Err(IngestError(format!(
                    "{} has multiple Parent references",
                    rec.feature_type
                )));
            }
            if parents.len() > 1 {
                anomaly.rejections.push(IngestRejection::new(
                    rec.line,
                    "GFF3_MULTI_PARENT_CHILD".to_string(),
                    rec.raw_line.clone(),
                ));
            }
            for tx_id in parents {
                child_parent_refs.push(tx_id.clone());
                if rec.feature_type == "exon" {
                    *transcript_exon_counts.entry(tx_id.clone()).or_insert(0) += 1;
                    *transcript_exon_span.entry(tx_id).or_insert(0) +=
                        rec.end.saturating_sub(rec.start) + 1;
                } else if rec.feature_type == "CDS" {
                    transcript_has_cds.insert(tx_id, true);
                }
            }
        } else {
            anomaly.unknown_feature_types.push(rec.feature_type.clone());
            anomaly.rejections.push(IngestRejection::new(
                rec.line,
                "GFF3_UNKNOWN_FEATURE".to_string(),
                rec.raw_line.clone(),
            ));
            if matches!(opts.unknown_feature_policy, UnknownFeaturePolicy::Reject)
                && matches!(opts.strictness, StrictnessMode::Strict)
            {
                return Err(IngestError(format!(
                    "unknown GFF3 feature type: {}",
                    rec.feature_type
                )));
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
            let seqids: std::collections::BTreeSet<String> =
                candidates.iter().map(|x| x.seqid.clone()).collect();
            if seqids.len() > 1 {
                anomaly
                    .overlapping_gene_ids_across_contigs
                    .push(key.clone());
                if !opts.allow_overlap_gene_ids_across_contigs
                    && matches!(opts.strictness, StrictnessMode::Strict)
                {
                    return Err(IngestError(format!(
                        "gene_id {key} appears across multiple contigs: {:?}",
                        seqids
                    )));
                }
            }
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
    let mut retained = Vec::with_capacity(transcript_rows_pending.len());
    let transcript_ids: HashSet<String> = transcript_rows_pending
        .iter()
        .map(|tx| tx.transcript_id.clone())
        .collect();
    for parent_tx in child_parent_refs {
        if !transcript_ids.contains(&parent_tx) {
            anomaly.orphan_transcripts.push(parent_tx);
        }
    }
    for tx in transcript_rows_pending {
        if deduped.contains_key(&tx.parent_gene_id) {
            retained.push(tx);
        } else {
            anomaly.orphan_transcripts.push(tx.transcript_id);
        }
    }
    let mut transcript_rows_pending = retained;
    if !transcript_rows_pending.is_empty() {
        let mut by_tx: HashMap<String, Vec<TranscriptRecord>> = HashMap::new();
        for tx in transcript_rows_pending {
            by_tx.entry(tx.transcript_id.clone()).or_default().push(tx);
        }
        let mut merged = Vec::new();
        let mut keys: Vec<String> = by_tx.keys().cloned().collect();
        keys.sort();
        for k in keys {
            let mut group = by_tx.remove(&k).unwrap_or_default();
            if group.len() > 1 {
                if matches!(
                    opts.duplicate_transcript_id_policy,
                    DuplicateTranscriptIdPolicy::Reject
                )
                    && matches!(opts.strictness, StrictnessMode::Strict)
                {
                    return Err(IngestError(format!("duplicate transcript_id: {k}")));
                }
                group.sort_by(|a, b| {
                    a.seqid
                        .cmp(&b.seqid)
                        .then(a.start.cmp(&b.start))
                        .then(a.end.cmp(&b.end))
                        .then(a.parent_gene_id.cmp(&b.parent_gene_id))
                });
                anomaly.rejections.push(IngestRejection::new(
                    0,
                    "GFF3_DUPLICATE_TRANSCRIPT_ID".to_string(),
                    k.clone(),
                ));
            }
            if let Some(first) = group.into_iter().next() {
                merged.push(first);
            }
        }
        transcript_rows_pending = merged;
    }
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
    anomaly.overlapping_gene_ids_across_contigs =
        canonical::stable_sort_by_key(anomaly.overlapping_gene_ids_across_contigs, |x| x.clone());
    anomaly.orphan_transcripts =
        canonical::stable_sort_by_key(anomaly.orphan_transcripts, |x| x.clone());
    anomaly.parent_cycles = canonical::stable_sort_by_key(anomaly.parent_cycles, |x| x.clone());
    anomaly.attribute_fallbacks =
        canonical::stable_sort_by_key(anomaly.attribute_fallbacks, |x| x.clone());
    anomaly.unknown_feature_types =
        canonical::stable_sort_by_key(anomaly.unknown_feature_types, |x| x.clone());
    anomaly.missing_required_fields =
        canonical::stable_sort_by_key(anomaly.missing_required_fields, |x| x.clone());
    anomaly.rejections = canonical::stable_sort_by_key(anomaly.rejections, |x| {
        (x.line, x.code.clone(), x.sample.clone())
    });
    anomaly.missing_parents.dedup();
    anomaly.missing_transcript_parents.dedup();
    anomaly.multiple_parent_transcripts.dedup();
    anomaly.unknown_contigs.dedup();
    anomaly.overlapping_ids.dedup();
    anomaly.duplicate_gene_ids.dedup();
    anomaly.overlapping_gene_ids_across_contigs.dedup();
    anomaly.orphan_transcripts.dedup();
    anomaly.parent_cycles.dedup();
    anomaly.attribute_fallbacks.dedup();
    anomaly.unknown_feature_types.dedup();
    anomaly.missing_required_fields.dedup();
    anomaly
        .rejections
        .dedup_by(|a, b| a.line == b.line && a.code == b.code && a.sample == b.sample);

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

fn used_gene_id_fallback(
    attrs: &std::collections::BTreeMap<String, String>,
    opts: &IngestOptions,
) -> bool {
    match &opts.gene_identifier_policy {
        GeneIdentifierPolicy::Gff3Id => false,
        GeneIdentifierPolicy::PreferEnsemblStableId {
            attribute_keys,
            fallback_to_gff3_id,
        } => {
            if !fallback_to_gff3_id {
                return false;
            }
            !attribute_keys
                .iter()
                .any(|k| attrs.get(k).map(|v| !v.trim().is_empty()).unwrap_or(false))
        }
        _ => false,
    }
}

fn detect_parent_cycles(records: &[Gff3Record]) -> Vec<String> {
    let mut edges: HashMap<String, Vec<String>> = HashMap::new();
    for rec in records {
        let Some(id) = rec.attrs.get("ID").cloned() else {
            continue;
        };
        let parents = rec
            .attrs
            .get("Parent")
            .map(|x| {
                x.split(',')
                    .map(str::trim)
                    .filter(|p| !p.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        edges.insert(id, parents);
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Color {
        White,
        Gray,
        Black,
    }
    let mut color: HashMap<String, Color> =
        edges.keys().map(|k| (k.clone(), Color::White)).collect();
    let mut cycles = Vec::new();

    fn dfs(
        node: &str,
        edges: &HashMap<String, Vec<String>>,
        color: &mut HashMap<String, Color>,
        stack: &mut Vec<String>,
        out: &mut Vec<String>,
    ) {
        color.insert(node.to_string(), Color::Gray);
        stack.push(node.to_string());
        if let Some(nexts) = edges.get(node) {
            for nxt in nexts {
                let c = *color.get(nxt).unwrap_or(&Color::White);
                if c == Color::Gray {
                    out.push(format!("cycle:{}->{}", node, nxt));
                } else if c == Color::White {
                    dfs(nxt, edges, color, stack, out);
                }
            }
        }
        stack.pop();
        color.insert(node.to_string(), Color::Black);
    }

    let keys: Vec<String> = edges.keys().cloned().collect();
    for k in keys {
        if color.get(&k) == Some(&Color::White) {
            dfs(&k, &edges, &mut color, &mut Vec::new(), &mut cycles);
        }
    }
    cycles
}

pub fn parallelism_policy(max_threads: usize) -> Result<usize, IngestError> {
    if max_threads == 0 {
        return Err(IngestError("max_threads must be >= 1".to_string()));
    }
    // Determinism-first: current implementation is single-threaded transform.
    Ok(1)
}
