// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::canonical_model::{build_canonical_model, CanonicalModel};
use super::extract::{extract_gene_rows, ExtractResult};
use super::fai::{self, ContigStats};
use super::gff3::{
    parse_gff3_records, parse_sequence_regions, validate_sequence_region_conflicts,
};
use super::job::IngestJob;
use super::{IngestError, IngestOptions};
use crate::domain::query::canonical_contig_label;

pub struct DecodedIngest {
    pub contig_stats: BTreeMap<String, ContigStats>,
    pub extract: ExtractResult,
    pub canonical_model: CanonicalModel,
    pub canonical_query_semantic_payload: serde_json::Value,
}

pub fn decode_ingest_inputs(job: &IngestJob) -> Result<DecodedIngest, IngestError> {
    let opts: &IngestOptions = &job.options;

    if !job.inputs.fai_path.exists() {
        if opts.dev_allow_auto_generate_fai {
            fai::write_fai_from_fasta(&job.inputs.fasta_path, &job.inputs.fai_path)?;
        } else {
            return Err(IngestError(
                format!(
                    "FAI_REQUIRED_FOR_INGEST: missing {}. Generate with `samtools faidx {}` or enable --dev-auto-generate-fai for controlled development runs.",
                    job.inputs.fai_path.display(),
                    job.inputs.fasta_path.display()
                ),
            ));
        }
    }

    let contig_lengths = fai::read_fai_contig_lengths(&job.inputs.fai_path)?;
    let contig_stats = if opts.fasta_scanning_enabled {
        fai::read_fasta_contig_stats(
            &job.inputs.fasta_path,
            opts.compute_contig_fractions,
            opts.fasta_scan_max_bases,
        )?
    } else {
        contig_lengths
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    fai::ContigStats {
                        length: *v,
                        gc_fraction: None,
                        n_fraction: None,
                    },
                )
            })
            .collect()
    };

    let sequence_regions = parse_sequence_regions(&job.inputs.gff3_path)?;
    validate_sequence_region_conflicts(&sequence_regions)?;
    validate_sequence_regions_against_fai(&sequence_regions, &contig_lengths)?;
    let records = parse_gff3_records(&job.inputs.gff3_path)?;
    validate_scientific_reference_coherence(&records, opts)?;
    validate_gff3_reference_names(&records, &contig_lengths)?;
    let mut extract = extract_gene_rows(records.clone(), &contig_lengths, opts)?;
    apply_deterministic_ordering(&mut extract);
    let canonical = build_canonical_model(&records, &extract)?;

    Ok(DecodedIngest {
        contig_stats,
        extract,
        canonical_model: canonical.model,
        canonical_query_semantic_payload: canonical.query_semantic_payload,
    })
}

fn validate_scientific_reference_coherence(
    records: &[super::gff3::Gff3Record],
    opts: &IngestOptions,
) -> Result<(), IngestError> {
    let mut core_sources: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut core_normalized: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for rec in records {
        let label = canonical_contig_label(&rec.seqid);
        let is_core = label.parse::<u64>().is_ok() || matches!(label.as_str(), "x" | "y" | "mitochondrial");
        if !is_core {
            continue;
        }
        core_sources
            .entry(label.clone())
            .or_default()
            .insert(rec.seqid.clone());
        core_normalized
            .entry(label)
            .or_default()
            .insert(opts.seqid_policy.normalize(&rec.seqid));
    }
    let mut incoherent = Vec::new();
    for (label, raw_set) in core_sources {
        if raw_set.len() <= 1 {
            continue;
        }
        let normalized_set = core_normalized.get(&label).cloned().unwrap_or_default();
        if normalized_set.len() > 1 {
            incoherent.push(format!(
                "{}=>raw={:?}, normalized={:?}",
                label, raw_set, normalized_set
            ));
        }
    }
    if !incoherent.is_empty() {
        return Err(IngestError(format!(
            "SCIENTIFIC_INCOHERENT_SOURCE_COMBINATION: inconsistent contig naming families detected. Resolve with explicit seqid aliases or corrected sources. details={:?}",
            incoherent
        )));
    }
    Ok(())
}

fn validate_sequence_regions_against_fai(
    regions: &[super::gff3::SequenceRegion],
    contig_lengths: &BTreeMap<String, u64>,
) -> Result<(), IngestError> {
    let mut missing = BTreeSet::new();
    for region in regions {
        if !contig_lengths.contains_key(&region.seqid) {
            missing.insert(region.seqid.clone());
        }
    }
    if !missing.is_empty() {
        let refs = missing.into_iter().collect::<Vec<_>>().join(", ");
        return Err(IngestError(format!(
            "GFF3_SEQUENCE_REGION_NOT_IN_FASTA_FAI: [{}]. Fix by aligning sequence-region declarations with FASTA/FAI contig IDs.",
            refs
        )));
    }
    Ok(())
}

fn validate_gff3_reference_names(
    records: &[super::gff3::Gff3Record],
    contig_lengths: &BTreeMap<String, u64>,
) -> Result<(), IngestError> {
    let mut missing = BTreeSet::new();
    for rec in records {
        if !contig_lengths.contains_key(&rec.seqid) {
            missing.insert(rec.seqid.clone());
        }
    }
    if !missing.is_empty() {
        let refs = missing.into_iter().collect::<Vec<_>>().join(", ");
        return Err(IngestError(format!(
            "GFF3_REFERENCE_NOT_IN_FASTA_FAI: [{}]. Fix source inputs so every GFF3 seqid exists in FASTA/FAI with matching naming.",
            refs
        )));
    }
    Ok(())
}

fn apply_deterministic_ordering(extract: &mut ExtractResult) {
    extract.gene_rows.sort_by(|a, b| {
        a.seqid
            .cmp(&b.seqid)
            .then(a.start.cmp(&b.start))
            .then(a.gene_id.cmp(&b.gene_id))
    });
    extract.transcript_rows.sort_by(|a, b| {
        a.seqid
            .cmp(&b.seqid)
            .then(a.start.cmp(&b.start))
            .then(a.transcript_id.cmp(&b.transcript_id))
    });
    extract.exon_rows.sort_by(|a, b| {
        a.seqid
            .cmp(&b.seqid)
            .then(a.start.cmp(&b.start))
            .then(a.exon_id.cmp(&b.exon_id))
    });
}
