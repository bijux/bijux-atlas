// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::query::{
    query_genes, GeneFields, GeneFilter, GeneQueryRequest, IntervalSemantics, QueryLimits,
    RegionFilter,
};
use rusqlite::Connection;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    description: String,
    region: String,
    interval_mode: String,
    limit: usize,
    #[serde(default)]
    expected_gene_ids: Option<Vec<String>>,
    #[serde(default)]
    expected_error_contains: Option<String>,
}

fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().expect("open memory db");
    conn.execute_batch(
        "
        CREATE TABLE gene_summary (
          id INTEGER PRIMARY KEY,
          gene_id TEXT NOT NULL,
          name TEXT NOT NULL,
          name_normalized TEXT NOT NULL,
          biotype TEXT NOT NULL,
          seqid TEXT NOT NULL,
          start INTEGER NOT NULL,
          end INTEGER NOT NULL,
          transcript_count INTEGER NOT NULL,
          exon_count INTEGER NOT NULL DEFAULT 0,
          total_exon_span INTEGER NOT NULL DEFAULT 0,
          cds_present INTEGER NOT NULL DEFAULT 0,
          sequence_length INTEGER NOT NULL
        );
        CREATE TABLE dataset_stats (
          dimension TEXT NOT NULL,
          value TEXT NOT NULL,
          gene_count INTEGER NOT NULL,
          PRIMARY KEY (dimension, value)
        );
        CREATE VIRTUAL TABLE gene_summary_rtree USING rtree(gene_rowid, start, end);
        CREATE INDEX idx_gene_summary_gene_id ON gene_summary(gene_id);
        CREATE INDEX idx_gene_summary_name_normalized ON gene_summary(name_normalized);
        CREATE INDEX idx_gene_summary_biotype ON gene_summary(biotype);
        CREATE INDEX idx_gene_summary_region ON gene_summary(seqid, start, end);
        ",
    )
    .expect("schema");

    let rows = vec![
        (1, "gene1", "BRCA1", "protein_coding", "chr1", 10, 40, 31),
        (2, "gene2", "BRCA2", "protein_coding", "chr1", 50, 90, 41),
        (3, "gene3", "TP53", "lncRNA", "chr2", 5, 25, 21),
    ];
    for (id, gene_id, name, biotype, seqid, start, end, length) in rows {
        conn.execute(
            "INSERT INTO gene_summary (id, gene_id, name, name_normalized, biotype, seqid, start, end, transcript_count, sequence_length)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9)",
            rusqlite::params![
                id,
                gene_id,
                name,
                name.to_ascii_lowercase(),
                biotype,
                seqid,
                start,
                end,
                length
            ],
        )
        .expect("insert row");
        conn.execute(
            "INSERT INTO gene_summary_rtree (gene_rowid, start, end) VALUES (?1, ?2, ?3)",
            rusqlite::params![id, start as f64, end as f64],
        )
        .expect("insert rtree");
    }
    conn.execute_batch(
        "
        INSERT INTO dataset_stats (dimension, value, gene_count)
        SELECT 'biotype', biotype, COUNT(*) FROM gene_summary GROUP BY biotype;
        INSERT INTO dataset_stats (dimension, value, gene_count)
        SELECT 'seqid', seqid, COUNT(*) FROM gene_summary GROUP BY seqid;
        ",
    )
    .expect("stats");
    conn
}

fn limits() -> QueryLimits {
    QueryLimits {
        max_limit: 500,
        max_transcript_limit: 500,
        max_region_span: 5_000_000,
        max_region_estimated_rows: 1_000,
        max_prefix_cost_units: 80_000,
        heavy_projection_limit: 200,
        min_prefix_len: 2,
        max_prefix_len: 64,
        max_work_units: 2_000,
        max_serialization_bytes: 512 * 1024,
    }
}

fn parse_region(raw: &str) -> RegionFilter {
    let (seqid, span) = raw.split_once(':').expect("region split");
    let (start, end) = span.split_once('-').expect("span split");
    RegionFilter {
        seqid: seqid.to_string(),
        start: start.parse::<u64>().expect("start"),
        end: end.parse::<u64>().expect("end"),
    }
}

fn parse_interval_mode(raw: &str) -> IntervalSemantics {
    match raw {
        "overlap" => IntervalSemantics::Overlap,
        "containment" => IntervalSemantics::Containment,
        "boundary_touch" => IntervalSemantics::BoundaryTouch,
        _ => panic!("unsupported interval mode in fixture: {raw}"),
    }
}

#[test]
fn query_contract_fixtures_lock_edge_case_semantics() {
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/query_contract/edge_cases.json");
    let raw = std::fs::read_to_string(&fixture_path).expect("read fixture file");
    let fixture: Fixture = serde_json::from_str(&raw).expect("parse fixture");
    let conn = setup_db();

    for case in fixture.cases {
        let req = GeneQueryRequest {
            fields: GeneFields::default(),
            filter: GeneFilter {
                region: Some(parse_region(&case.region)),
                interval: parse_interval_mode(&case.interval_mode),
                ..Default::default()
            },
            limit: case.limit,
            cursor: None,
            dataset_key: Some("110/homo_sapiens/GRCh38".to_string()),
            allow_full_scan: false,
        };
        match (
            case.expected_gene_ids.as_ref(),
            case.expected_error_contains.as_deref(),
        ) {
            (Some(expected_ids), None) => {
                let got = query_genes(&conn, &req, &limits(), b"fixture-secret")
                    .expect("query")
                    .rows
                    .into_iter()
                    .map(|row| row.gene_id)
                    .collect::<Vec<_>>();
                assert_eq!(
                    got, *expected_ids,
                    "fixture case {} failed: {}",
                    case.id, case.description
                );
            }
            (None, Some(expected_error)) => {
                let err = query_genes(&conn, &req, &limits(), b"fixture-secret")
                    .expect_err("expected query rejection");
                assert!(
                    err.message.contains(expected_error),
                    "fixture case {} expected error containing {:?}, got {:?}",
                    case.id,
                    expected_error,
                    err.message
                );
            }
            _ => panic!(
                "fixture case {} must define exactly one of expected_gene_ids or expected_error_contains",
                case.id
            ),
        }
    }
}
