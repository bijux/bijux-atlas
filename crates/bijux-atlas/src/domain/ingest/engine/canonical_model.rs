// SPDX-License-Identifier: Apache-2.0

use super::extract::{ExonRecord, ExtractResult, GeneRecord, TranscriptRecord};
use super::gff3::Gff3Record;
use super::IngestError;
use crate::domain::canonical;
use crate::domain::sha256_hex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalFeatureType {
    Gene,
    Transcript,
    Exon,
    Cds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationCompleteness {
    Complete,
    Partial,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureCodingClass {
    Coding,
    NonCoding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneAnnotationClass {
    Standard,
    Pseudogene,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageRecord {
    pub gff3_line: usize,
    pub seqid: String,
    pub feature_type: String,
    pub source_feature_id: Option<String>,
    pub parent_ids: Vec<String>,
    pub record_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalExon {
    pub exon_id: String,
    pub transcript_id: String,
    pub seqid: String,
    pub start: u64,
    pub end: u64,
    pub exon_length: u64,
    pub feature_type: CanonicalFeatureType,
    pub completeness: AnnotationCompleteness,
    pub lineage: Vec<LineageRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalCds {
    pub cds_id: String,
    pub transcript_id: String,
    pub seqid: String,
    pub start: u64,
    pub end: u64,
    pub cds_length: u64,
    pub phase: Option<String>,
    pub feature_type: CanonicalFeatureType,
    pub completeness: AnnotationCompleteness,
    pub lineage: Vec<LineageRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalTranscript {
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
    pub sequence_length: u64,
    pub spliced_length: Option<u64>,
    pub cds_span_length: Option<u64>,
    pub feature_type: CanonicalFeatureType,
    pub coding_class: FeatureCodingClass,
    pub completeness: AnnotationCompleteness,
    pub exons: Vec<CanonicalExon>,
    pub cds_segments: Vec<CanonicalCds>,
    pub lineage: Vec<LineageRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalGene {
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
    pub feature_type: CanonicalFeatureType,
    pub annotation_class: GeneAnnotationClass,
    pub completeness: AnnotationCompleteness,
    pub transcripts: Vec<CanonicalTranscript>,
    pub lineage: Vec<LineageRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalSummary {
    pub genes: u64,
    pub transcripts: u64,
    pub exons: u64,
    pub cds: u64,
    pub contigs: u64,
    pub coding_transcripts: u64,
    pub noncoding_transcripts: u64,
    pub pseudogenes: u64,
    pub partial_genes: u64,
    pub partial_transcripts: u64,
    pub feature_type_counts: BTreeMap<String, u64>,
    pub contig_gene_counts: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalHashes {
    pub query_semantic_sha256: String,
    pub lineage_sensitive_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalModel {
    pub schema_version: u64,
    pub genes: Vec<CanonicalGene>,
    pub summary: CanonicalSummary,
    pub hashes: CanonicalHashes,
}

#[derive(Debug, Clone)]
pub struct CanonicalBuildOutput {
    pub model: CanonicalModel,
    pub query_semantic_payload: serde_json::Value,
}

pub fn build_canonical_model(
    records: &[Gff3Record],
    extract: &ExtractResult,
) -> Result<CanonicalBuildOutput, IngestError> {
    let lineage = build_lineage_index(records)?;
    let mut exons_by_transcript: HashMap<String, Vec<CanonicalExon>> = HashMap::new();
    let mut cds_by_transcript: HashMap<String, Vec<CanonicalCds>> = HashMap::new();

    for exon in &extract.exon_rows {
        let lineage_records = lineage_for_exon(&lineage, exon);
        exons_by_transcript
            .entry(exon.transcript_id.clone())
            .or_default()
            .push(canonical_exon(exon, lineage_records));
    }
    for rec in records.iter().filter(|r| r.feature_type == "CDS") {
        for parent in split_parents(rec.attrs.get("Parent")) {
            cds_by_transcript
                .entry(parent.to_string())
                .or_default()
                .push(canonical_cds(rec, parent));
        }
    }

    let mut transcripts_by_gene: HashMap<String, Vec<CanonicalTranscript>> = HashMap::new();
    for tx in &extract.transcript_rows {
        let mut tx_exons = exons_by_transcript
            .remove(&tx.transcript_id)
            .unwrap_or_default();
        tx_exons.sort_by(compare_exon);
        let mut tx_cds = cds_by_transcript
            .remove(&tx.transcript_id)
            .unwrap_or_default();
        tx_cds.sort_by(compare_cds);
        let tx_lineage = lineage_for_transcript(&lineage, tx);
        transcripts_by_gene
            .entry(tx.parent_gene_id.clone())
            .or_default()
            .push(canonical_transcript(tx, tx_exons, tx_cds, tx_lineage));
    }

    let mut genes: Vec<CanonicalGene> = Vec::new();
    for gene in &extract.gene_rows {
        let mut transcripts = transcripts_by_gene
            .remove(&gene.gene_id)
            .unwrap_or_default();
        transcripts.sort_by(compare_transcript);
        let gene_lineage = lineage_for_gene(&lineage, gene);
        genes.push(canonical_gene(gene, transcripts, gene_lineage));
    }
    genes.sort_by(compare_gene);

    let summary = summarize_model(&genes);
    let query_semantic_payload = semantic_payload(&genes, &summary);
    let query_semantic_sha256 = {
        let bytes = canonical::stable_json_bytes(&query_semantic_payload)
            .map_err(|e| IngestError(e.to_string()))?;
        sha256_hex(&bytes)
    };
    let lineage_sensitive_sha256 = {
        let bytes = canonical::stable_json_bytes(&genes).map_err(|e| IngestError(e.to_string()))?;
        sha256_hex(&bytes)
    };
    let model = CanonicalModel {
        schema_version: 1,
        genes,
        summary,
        hashes: CanonicalHashes {
            query_semantic_sha256,
            lineage_sensitive_sha256,
        },
    };
    Ok(CanonicalBuildOutput {
        model,
        query_semantic_payload,
    })
}

#[must_use]
pub fn contig_order_rank(seqid: &str) -> (u32, u64, String) {
    let trimmed = seqid.trim();
    let lowered = trimmed.to_ascii_lowercase();
    let core = lowered.strip_prefix("chr").unwrap_or(&lowered);
    if let Ok(num) = core.parse::<u64>() {
        return (0, num, lowered);
    }
    if core == "x" {
        return (1, 0, lowered);
    }
    if core == "y" {
        return (2, 0, lowered);
    }
    if core == "m" || core == "mt" || core == "mitochondria" {
        return (3, 0, lowered);
    }
    (4, 0, lowered)
}

fn canonical_gene(
    gene: &GeneRecord,
    transcripts: Vec<CanonicalTranscript>,
    lineage: Vec<LineageRecord>,
) -> CanonicalGene {
    let has_pseudo = gene.biotype.to_ascii_lowercase().contains("pseudo");
    let completeness = if transcripts.is_empty() {
        AnnotationCompleteness::Missing
    } else if transcripts
        .iter()
        .any(|tx| tx.completeness != AnnotationCompleteness::Complete)
    {
        AnnotationCompleteness::Partial
    } else {
        AnnotationCompleteness::Complete
    };
    CanonicalGene {
        gene_id: gene.gene_id.clone(),
        gene_name: gene.gene_name.clone(),
        biotype: gene.biotype.clone(),
        seqid: gene.seqid.clone(),
        start: gene.start,
        end: gene.end,
        transcript_count: gene.transcript_count,
        exon_count: gene.exon_count,
        total_exon_span: gene.total_exon_span,
        cds_present: gene.cds_present,
        sequence_length: gene.sequence_length,
        feature_type: CanonicalFeatureType::Gene,
        annotation_class: if has_pseudo {
            GeneAnnotationClass::Pseudogene
        } else {
            GeneAnnotationClass::Standard
        },
        completeness,
        transcripts,
        lineage,
    }
}

fn canonical_transcript(
    tx: &TranscriptRecord,
    exons: Vec<CanonicalExon>,
    cds_segments: Vec<CanonicalCds>,
    lineage: Vec<LineageRecord>,
) -> CanonicalTranscript {
    let coding_class = if tx.cds_present {
        FeatureCodingClass::Coding
    } else {
        FeatureCodingClass::NonCoding
    };
    let completeness = if tx.exon_count == 0 {
        AnnotationCompleteness::Missing
    } else if tx.exon_count != exons.len() as u64 || (tx.cds_present && cds_segments.is_empty()) {
        AnnotationCompleteness::Partial
    } else {
        AnnotationCompleteness::Complete
    };
    CanonicalTranscript {
        transcript_id: tx.transcript_id.clone(),
        parent_gene_id: tx.parent_gene_id.clone(),
        transcript_type: tx.transcript_type.clone(),
        biotype: tx.biotype.clone(),
        seqid: tx.seqid.clone(),
        start: tx.start,
        end: tx.end,
        exon_count: tx.exon_count,
        total_exon_span: tx.total_exon_span,
        cds_present: tx.cds_present,
        sequence_length: tx.sequence_length,
        spliced_length: tx.spliced_length,
        cds_span_length: tx.cds_span_length,
        feature_type: CanonicalFeatureType::Transcript,
        coding_class,
        completeness,
        exons,
        cds_segments,
        lineage,
    }
}

fn canonical_exon(exon: &ExonRecord, lineage: Vec<LineageRecord>) -> CanonicalExon {
    CanonicalExon {
        exon_id: exon.exon_id.clone(),
        transcript_id: exon.transcript_id.clone(),
        seqid: exon.seqid.clone(),
        start: exon.start,
        end: exon.end,
        exon_length: exon.exon_length,
        feature_type: CanonicalFeatureType::Exon,
        completeness: AnnotationCompleteness::Complete,
        lineage,
    }
}

fn canonical_cds(rec: &Gff3Record, transcript_id: &str) -> CanonicalCds {
    let cds_id = rec
        .attrs
        .get("ID")
        .cloned()
        .unwrap_or_else(|| format!("cds_line_{}_{}", rec.line, transcript_id));
    CanonicalCds {
        cds_id,
        transcript_id: transcript_id.to_string(),
        seqid: rec.seqid.clone(),
        start: rec.start,
        end: rec.end,
        cds_length: rec.end.saturating_sub(rec.start) + 1,
        phase: if rec.phase == "." {
            None
        } else {
            Some(rec.phase.clone())
        },
        feature_type: CanonicalFeatureType::Cds,
        completeness: AnnotationCompleteness::Complete,
        lineage: vec![lineage_from_record(rec)],
    }
}

fn compare_gene(a: &CanonicalGene, b: &CanonicalGene) -> Ordering {
    contig_order_rank(&a.seqid)
        .cmp(&contig_order_rank(&b.seqid))
        .then(a.start.cmp(&b.start))
        .then(a.end.cmp(&b.end))
        .then(a.gene_id.cmp(&b.gene_id))
}

fn compare_transcript(a: &CanonicalTranscript, b: &CanonicalTranscript) -> Ordering {
    contig_order_rank(&a.seqid)
        .cmp(&contig_order_rank(&b.seqid))
        .then(a.start.cmp(&b.start))
        .then(a.end.cmp(&b.end))
        .then(a.transcript_id.cmp(&b.transcript_id))
}

fn compare_exon(a: &CanonicalExon, b: &CanonicalExon) -> Ordering {
    contig_order_rank(&a.seqid)
        .cmp(&contig_order_rank(&b.seqid))
        .then(a.start.cmp(&b.start))
        .then(a.end.cmp(&b.end))
        .then(a.exon_id.cmp(&b.exon_id))
}

fn compare_cds(a: &CanonicalCds, b: &CanonicalCds) -> Ordering {
    contig_order_rank(&a.seqid)
        .cmp(&contig_order_rank(&b.seqid))
        .then(a.start.cmp(&b.start))
        .then(a.end.cmp(&b.end))
        .then(a.cds_id.cmp(&b.cds_id))
}

fn build_lineage_index(
    records: &[Gff3Record],
) -> Result<BTreeMap<(String, String, u64, u64), Vec<LineageRecord>>, IngestError> {
    let mut out: BTreeMap<(String, String, u64, u64), Vec<LineageRecord>> = BTreeMap::new();
    for rec in records {
        let key = (
            rec.feature_type.clone(),
            rec.seqid.clone(),
            rec.start,
            rec.end,
        );
        out.entry(key).or_default().push(lineage_from_record(rec));
    }
    for entries in out.values_mut() {
        entries.sort_by(|a, b| {
            a.gff3_line
                .cmp(&b.gff3_line)
                .then(a.record_sha256.cmp(&b.record_sha256))
        });
    }
    Ok(out)
}

fn lineage_from_record(rec: &Gff3Record) -> LineageRecord {
    let parent_ids = split_parents(rec.attrs.get("Parent"))
        .into_iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    LineageRecord {
        gff3_line: rec.line,
        seqid: rec.seqid.clone(),
        feature_type: rec.feature_type.clone(),
        source_feature_id: rec.attrs.get("ID").cloned(),
        parent_ids,
        record_sha256: sha256_hex(rec.raw_line.as_bytes()),
    }
}

fn lineage_for_gene(
    idx: &BTreeMap<(String, String, u64, u64), Vec<LineageRecord>>,
    gene: &GeneRecord,
) -> Vec<LineageRecord> {
    idx.get(&("gene".to_string(), gene.seqid.clone(), gene.start, gene.end))
        .cloned()
        .unwrap_or_default()
}

fn lineage_for_transcript(
    idx: &BTreeMap<(String, String, u64, u64), Vec<LineageRecord>>,
    tx: &TranscriptRecord,
) -> Vec<LineageRecord> {
    idx.get(&(
        tx.transcript_type.clone(),
        tx.seqid.clone(),
        tx.start,
        tx.end,
    ))
    .cloned()
    .or_else(|| {
        idx.get(&("transcript".to_string(), tx.seqid.clone(), tx.start, tx.end))
            .cloned()
    })
    .or_else(|| {
        idx.get(&("mRNA".to_string(), tx.seqid.clone(), tx.start, tx.end))
            .cloned()
    })
    .unwrap_or_default()
}

fn lineage_for_exon(
    idx: &BTreeMap<(String, String, u64, u64), Vec<LineageRecord>>,
    exon: &ExonRecord,
) -> Vec<LineageRecord> {
    idx.get(&("exon".to_string(), exon.seqid.clone(), exon.start, exon.end))
        .cloned()
        .unwrap_or_default()
}

fn split_parents(parent: Option<&String>) -> Vec<&str> {
    parent
        .map(|p| {
            p.split(',')
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn summarize_model(genes: &[CanonicalGene]) -> CanonicalSummary {
    let mut transcripts = 0_u64;
    let mut exons = 0_u64;
    let mut cds = 0_u64;
    let mut coding_transcripts = 0_u64;
    let mut noncoding_transcripts = 0_u64;
    let mut pseudogenes = 0_u64;
    let mut partial_genes = 0_u64;
    let mut partial_transcripts = 0_u64;
    let mut feature_type_counts = BTreeMap::new();
    let mut contig_gene_counts = BTreeMap::new();

    for gene in genes {
        *feature_type_counts.entry("gene".to_string()).or_insert(0) += 1;
        *contig_gene_counts.entry(gene.seqid.clone()).or_insert(0) += 1;
        if gene.annotation_class == GeneAnnotationClass::Pseudogene {
            pseudogenes += 1;
        }
        if gene.completeness != AnnotationCompleteness::Complete {
            partial_genes += 1;
        }
        for tx in &gene.transcripts {
            transcripts += 1;
            *feature_type_counts
                .entry("transcript".to_string())
                .or_insert(0) += 1;
            if tx.coding_class == FeatureCodingClass::Coding {
                coding_transcripts += 1;
            } else {
                noncoding_transcripts += 1;
            }
            if tx.completeness != AnnotationCompleteness::Complete {
                partial_transcripts += 1;
            }
            exons += tx.exons.len() as u64;
            cds += tx.cds_segments.len() as u64;
            *feature_type_counts.entry("exon".to_string()).or_insert(0) += tx.exons.len() as u64;
            *feature_type_counts.entry("cds".to_string()).or_insert(0) +=
                tx.cds_segments.len() as u64;
        }
    }
    CanonicalSummary {
        genes: genes.len() as u64,
        transcripts,
        exons,
        cds,
        contigs: contig_gene_counts.len() as u64,
        coding_transcripts,
        noncoding_transcripts,
        pseudogenes,
        partial_genes,
        partial_transcripts,
        feature_type_counts,
        contig_gene_counts,
    }
}

fn semantic_payload(genes: &[CanonicalGene], summary: &CanonicalSummary) -> serde_json::Value {
    let gene_items = genes
        .iter()
        .map(|g| {
            serde_json::json!({
                "gene_id": g.gene_id,
                "biotype": g.biotype,
                "seqid": g.seqid,
                "start": g.start,
                "end": g.end,
                "annotation_class": g.annotation_class,
                "completeness": g.completeness,
                "transcripts": g.transcripts.iter().map(|tx| serde_json::json!({
                    "transcript_id": tx.transcript_id,
                    "parent_gene_id": tx.parent_gene_id,
                    "transcript_type": tx.transcript_type,
                    "biotype": tx.biotype,
                    "seqid": tx.seqid,
                    "start": tx.start,
                    "end": tx.end,
                    "coding_class": tx.coding_class,
                    "completeness": tx.completeness,
                    "exons": tx.exons.iter().map(|ex| serde_json::json!({
                        "exon_id": ex.exon_id,
                        "start": ex.start,
                        "end": ex.end
                    })).collect::<Vec<_>>(),
                    "cds_segments": tx.cds_segments.iter().map(|cds| serde_json::json!({
                        "cds_id": cds.cds_id,
                        "start": cds.start,
                        "end": cds.end,
                        "phase": cds.phase
                    })).collect::<Vec<_>>()
                })).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "schema_version": 1,
        "summary": summary,
        "genes": gene_items
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::dataset::IngestAnomalyReport;

    #[test]
    fn contig_order_rank_is_stable_for_numeric_and_special_contigs() {
        let mut contigs = vec![
            "chrX".to_string(),
            "chr2".to_string(),
            "chr10".to_string(),
            "chrM".to_string(),
            "chr1".to_string(),
            "scaffold_42".to_string(),
        ];
        contigs.sort_by_key(|c| contig_order_rank(c));
        assert_eq!(
            contigs,
            vec![
                "chr1".to_string(),
                "chr2".to_string(),
                "chr10".to_string(),
                "chrX".to_string(),
                "chrM".to_string(),
                "scaffold_42".to_string()
            ]
        );
    }

    #[test]
    fn canonical_model_marks_pseudogene_and_missing_transcripts() {
        let extract = ExtractResult {
            gene_rows: vec![GeneRecord {
                gene_id: "g1".to_string(),
                gene_name: "G1".to_string(),
                biotype: "pseudogene".to_string(),
                seqid: "chr1".to_string(),
                start: 10,
                end: 20,
                transcript_count: 0,
                exon_count: 0,
                total_exon_span: 0,
                cds_present: false,
                sequence_length: 11,
            }],
            transcript_rows: vec![],
            exon_rows: vec![],
            anomaly: IngestAnomalyReport::default(),
            biotype_distribution: BTreeMap::new(),
            contig_distribution: BTreeMap::new(),
            total_features: 1,
            unknown_contig_features: 0,
            max_contig_name_length: 4,
            cds_feature_count: 0,
            contig_class_distribution: BTreeMap::new(),
            seqid_normalization_traces: BTreeMap::new(),
            biotype_source_counts: BTreeMap::new(),
        };
        let records = vec![Gff3Record {
            line: 1,
            seqid: "chr1".to_string(),
            feature_type: "gene".to_string(),
            strand: "+".to_string(),
            phase: ".".to_string(),
            start: 10,
            end: 20,
            attrs: BTreeMap::from([
                ("ID".to_string(), "g1".to_string()),
                ("Name".to_string(), "G1".to_string()),
            ]),
            duplicate_attr_keys: Default::default(),
            raw_line: "chr1\tsrc\tgene\t10\t20\t.\t+\t.\tID=g1;Name=G1".to_string(),
        }];
        let model = build_canonical_model(&records, &extract)
            .expect("build canonical model")
            .model;
        assert_eq!(model.summary.pseudogenes, 1);
        assert_eq!(model.summary.partial_genes, 1);
        assert_eq!(
            model.genes[0].annotation_class,
            GeneAnnotationClass::Pseudogene
        );
        assert_eq!(model.genes[0].completeness, AnnotationCompleteness::Missing);
    }
}
