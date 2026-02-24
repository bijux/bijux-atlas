// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use crate::extract::{extract_gene_rows, ExtractResult};
use crate::fai::{self, ContigStats};
use crate::gff3::parse_gff3_records;
use crate::job::IngestJob;
use crate::{IngestError, IngestOptions};

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

    let records = parse_gff3_records(&job.inputs.gff3_path)?;
    let mut extract = extract_gene_rows(records, &contig_lengths, opts)?;
    apply_deterministic_ordering(&mut extract);

    Ok(DecodedIngest {
        contig_stats,
        extract,
    })
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
