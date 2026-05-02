// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::extract::{extract_gene_rows, ExtractResult};
use super::fai::{self, ContigStats};
use super::gff3::{
    parse_gff3_records, parse_sequence_regions, validate_sequence_region_conflicts,
};
use super::job::IngestJob;
use super::{IngestError, IngestOptions};

pub struct DecodedIngest {
    pub contig_stats: BTreeMap<String, ContigStats>,
    pub extract: ExtractResult,
}

pub fn decode_ingest_inputs(job: &IngestJob) -> Result<DecodedIngest, IngestError> {
    let opts: &IngestOptions = &job.options;

    if !job.inputs.fai_path.exists() {
        if opts.dev_allow_auto_generate_fai {
            fai::write_fai_from_fasta(&job.inputs.fasta_path, &job.inputs.fai_path)?;
        } else {
            return Err(IngestError(
                "FAI index is required for ingest (enable dev auto-generate explicitly)"
                    .to_string(),
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
    validate_gff3_reference_names(&records, &contig_lengths)?;
    let mut extract = extract_gene_rows(records, &contig_lengths, opts)?;
    apply_deterministic_ordering(&mut extract);

    Ok(DecodedIngest {
        contig_stats,
        extract,
    })
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
