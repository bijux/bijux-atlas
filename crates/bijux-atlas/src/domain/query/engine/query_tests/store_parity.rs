// SPDX-License-Identifier: Apache-2.0

use super::setup_and_core::{limits, setup_db};
use crate::domain::query::*;

#[derive(Debug, Clone)]
struct RefGene {
    gene_id: &'static str,
    name: &'static str,
    biotype: &'static str,
    seqid: &'static str,
    start: u64,
    end: u64,
}

fn fixture_rows() -> Vec<RefGene> {
    vec![
        RefGene {
            gene_id: "gene1",
            name: "BRCA1",
            biotype: "protein_coding",
            seqid: "chr1",
            start: 10,
            end: 40,
        },
        RefGene {
            gene_id: "gene2",
            name: "BRCA2",
            biotype: "protein_coding",
            seqid: "chr1",
            start: 50,
            end: 90,
        },
        RefGene {
            gene_id: "gene3",
            name: "TP53",
            biotype: "lncRNA",
            seqid: "chr2",
            start: 5,
            end: 25,
        },
        RefGene {
            gene_id: "gene4",
            name: "TNF",
            biotype: "lncRNA",
            seqid: "chr2",
            start: 30,
            end: 45,
        },
        RefGene {
            gene_id: "gene5",
            name: "BRCA_ABC",
            biotype: "unknown",
            seqid: "chr2",
            start: 50,
            end: 60,
        },
        RefGene {
            gene_id: "gene6",
            name: "DUPNAME",
            biotype: "protein_coding",
            seqid: "chr1",
            start: 95,
            end: 105,
        },
        RefGene {
            gene_id: "gene7",
            name: "DUPNAME",
            biotype: "protein_coding",
            seqid: "chr1",
            start: 95,
            end: 105,
        },
    ]
}

fn matches_region(row: &RefGene, req: &GeneQueryRequest) -> bool {
    let Some(region) = &req.filter.region else {
        return true;
    };
    if row.seqid != region.seqid {
        return false;
    }
    match req.filter.interval {
        IntervalSemantics::Overlap => row.start <= region.end && row.end >= region.start,
        IntervalSemantics::Containment => row.start >= region.start && row.end <= region.end,
        IntervalSemantics::BoundaryTouch => row.end == region.start || row.start == region.end,
    }
}

fn reference_gene_ids(req: &GeneQueryRequest) -> Vec<String> {
    let mut out = fixture_rows()
        .into_iter()
        .filter(|row| {
            if let Some(gene_id) = &req.filter.gene_id {
                if row.gene_id != gene_id {
                    return false;
                }
            }
            if let Some(name) = &req.filter.name {
                if normalize_name_lookup(row.name) != normalize_name_lookup(name) {
                    return false;
                }
            }
            if let Some(prefix) = &req.filter.name_prefix {
                if !normalize_name_lookup(row.name).starts_with(&normalize_name_lookup(prefix)) {
                    return false;
                }
            }
            if let Some(biotype) = &req.filter.biotype {
                if row.biotype != biotype {
                    return false;
                }
            }
            matches_region(row, req)
        })
        .collect::<Vec<_>>();

    match req.filter.sort {
        QuerySort::GeneIdAsc => {
            out.sort_by(|a, b| a.gene_id.cmp(b.gene_id));
        }
        QuerySort::RegionAsc => {
            out.sort_by(|a, b| {
                a.seqid
                    .cmp(b.seqid)
                    .then(a.start.cmp(&b.start))
                    .then(a.gene_id.cmp(b.gene_id))
            });
        }
        QuerySort::Auto => {
            if req.filter.region.is_some() {
                out.sort_by(|a, b| {
                    a.seqid
                        .cmp(b.seqid)
                        .then(a.start.cmp(&b.start))
                        .then(a.gene_id.cmp(b.gene_id))
                });
            } else {
                out.sort_by(|a, b| a.gene_id.cmp(b.gene_id));
            }
        }
    }

    out.into_iter()
        .take(req.limit)
        .map(|r| r.gene_id.to_string())
        .collect()
}

#[test]
fn sqlite_results_match_store_neutral_reference_semantics() {
    let conn = setup_db();
    let mut cases = vec![
        GeneQueryRequest {
            fields: GeneFields::default(),
            filter: GeneFilter {
                gene_id: Some("gene1".to_string()),
                ..Default::default()
            },
            limit: 10,
            cursor: None,
            dataset_key: None,
            allow_full_scan: false,
        },
        GeneQueryRequest {
            fields: GeneFields::default(),
            filter: GeneFilter {
                name_prefix: Some("BR".to_string()),
                ..Default::default()
            },
            limit: 10,
            cursor: None,
            dataset_key: None,
            allow_full_scan: false,
        },
        GeneQueryRequest {
            fields: GeneFields::default(),
            filter: GeneFilter {
                biotype: Some("protein_coding".to_string()),
                ..Default::default()
            },
            limit: 10,
            cursor: None,
            dataset_key: None,
            allow_full_scan: false,
        },
    ];

    let mut region_overlap = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            region: Some(RegionFilter {
                seqid: "chr1".to_string(),
                start: 40,
                end: 50,
            }),
            ..Default::default()
        },
        limit: 10,
        cursor: None,
        dataset_key: None,
        allow_full_scan: false,
    };
    region_overlap.filter.interval = IntervalSemantics::Overlap;
    cases.push(region_overlap.clone());
    region_overlap.filter.interval = IntervalSemantics::Containment;
    cases.push(region_overlap.clone());
    region_overlap.filter.interval = IntervalSemantics::BoundaryTouch;
    region_overlap.filter.sort = QuerySort::RegionAsc;
    cases.push(region_overlap);

    for req in cases {
        let expected = reference_gene_ids(&req);
        let got = query_genes(&conn, &req, &limits(), b"s")
            .expect("sqlite query")
            .rows
            .into_iter()
            .map(|r| r.gene_id)
            .collect::<Vec<_>>();
        assert_eq!(got, expected, "store parity failed for request: {req:?}");
    }
}
