// SPDX-License-Identifier: Apache-2.0

use super::support::*;

#[test]
fn ingest_is_deterministic_and_matches_contract() {
    let root = tempdir().expect("tempdir");
    let run1 = ingest_dataset(&opts(root.path(), StrictnessMode::Strict)).expect("run1");
    let alt = tempdir().expect("tempdir2");
    let run2 = ingest_dataset(&opts(alt.path(), StrictnessMode::Strict)).expect("run2");

    assert_eq!(
        run1.manifest.checksums.sqlite_sha256,
        run2.manifest.checksums.sqlite_sha256
    );
    assert_eq!(run1.manifest.stats.gene_count, 2);
    assert_eq!(run1.manifest.stats.transcript_count, 3);
    assert!(run1.release_gene_index_path.exists());
}

#[test]
fn deterministic_across_parallelism_settings() {
    let root = tempdir().expect("tempdir");
    let mut o1 = opts(root.path(), StrictnessMode::Strict);
    o1.max_threads = 1;
    let run1 = ingest_dataset(&o1).expect("run1");

    let alt = tempdir().expect("tempdir2");
    let mut o2 = opts(alt.path(), StrictnessMode::Strict);
    o2.max_threads = 8;
    let run2 = ingest_dataset(&o2).expect("run2");

    assert_eq!(
        run1.manifest.dataset_signature_sha256,
        run2.manifest.dataset_signature_sha256
    );
    assert_eq!(
        run1.manifest.checksums.sqlite_sha256,
        run2.manifest.checksums.sqlite_sha256
    );
}

#[test]
fn strict_mode_rejects_missing_parent() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.gff3_path = fixture_dir().join("genes_missing_parent.gff3");
    assert!(ingest_dataset(&o).is_err());
}

#[test]
fn strict_mode_rejects_transcript_parent_that_is_not_gene() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    let gff = root.path().join("bad-parent.gff3");
    std::fs::write(
        &gff,
        "##gff-version 3\nchr1\tsrc\tgene\t1\t20\t.\t+\t.\tID=gene1\nchr1\tsrc\tmRNA\t1\t20\t.\t+\t.\tID=tx1;Parent=not_a_gene\n",
    )
    .expect("write gff");
    o.gff3_path = gff;
    let err = ingest_dataset(&o).expect_err("bad transcript parent must fail");
    assert!(
        err.to_string().contains("GFF3_PARENT_NOT_GENE"),
        "unexpected error: {}",
        err
    );
}

#[test]
fn strict_mode_rejects_exon_parent_that_is_not_transcript() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    let gff = root.path().join("bad-child-parent.gff3");
    std::fs::write(
        &gff,
        "##gff-version 3\nchr1\tsrc\tgene\t1\t20\t.\t+\t.\tID=gene1\nchr1\tsrc\texon\t1\t20\t.\t+\t.\tID=ex1;Parent=missing_tx\n",
    )
    .expect("write gff");
    o.gff3_path = gff;
    let err = ingest_dataset(&o).expect_err("bad child parent must fail");
    assert!(
        err.to_string().contains("GFF3_PARENT_NOT_TRANSCRIPT"),
        "unexpected error: {}",
        err
    );
}

#[test]
fn report_only_collects_anomalies() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::ReportOnly);
    o.gff3_path = fixture_dir().join("genes_missing_parent.gff3");
    let result = ingest_dataset(&o).expect("report only should succeed");
    assert!(!result.anomaly_report.missing_parents.is_empty());
}

#[test]
fn strict_warn_mode_fails_on_qc_warn() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::ReportOnly);
    o.gff3_path = fixture_dir().join("genes_missing_parent.gff3");
    o.fail_on_warn = true;
    let err = ingest_dataset(&o).expect_err("strict warn must fail");
    assert!(err.to_string().contains("INGEST_WARN_POLICY_REJECTED"));
}

#[test]
fn anomaly_threshold_gate_refuses_ingest_when_warn_budget_exceeded() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::ReportOnly);
    o.gff3_path = fixture_dir().join("genes_missing_parent.gff3");
    o.max_warn_anomalies = Some(0);
    let err = ingest_dataset(&o).expect_err("warn anomaly threshold must fail");
    assert!(err.to_string().contains("max_warn_anomalies"));
}

#[test]
fn report_only_writes_qc_and_anomaly_without_sqlite_manifest() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::ReportOnly);
    o.report_only = true;
    let out = ingest_dataset(&o).expect("report-only ingest");
    assert!(out.qc_report_path.exists());
    assert!(out.anomaly_report_path.exists());
    assert!(!out.sqlite_path.exists());
    assert!(!out.manifest_path.exists());
}

#[test]
fn normalized_replay_matches_db_content_counts() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.emit_normalized_debug = true;
    o.normalized_replay_mode = true;
    let run = ingest_dataset(&o).expect("ingest");
    let normalized = run.normalized_debug_path.clone().expect("normalized path");
    assert!(normalized.exists());
    let replay = replay_normalized_counts(&normalized).expect("replay");
    let conn = rusqlite::Connection::open(&run.sqlite_path).expect("open sqlite");
    let gene_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM gene_summary", [], |r| r.get(0))
        .expect("genes");
    let tx_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM transcript_summary", [], |r| r.get(0))
        .expect("tx");
    let exon_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM exons", [], |r| r.get(0))
        .expect("exons");
    assert_eq!(replay.genes as i64, gene_count);
    assert_eq!(replay.transcripts as i64, tx_count);
    assert_eq!(replay.exons as i64, exon_count);
}

#[test]
fn normalized_output_is_blocked_in_prod_mode() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.prod_mode = true;
    o.emit_normalized_debug = true;
    let err = ingest_dataset(&o).expect_err("prod mode must reject normalized");
    assert!(err.to_string().contains("disabled in production mode"));
}

#[test]
fn cyclic_parent_graph_is_detected() {
    let root = tempdir().expect("tempdir");
    let err = {
        let mut o = opts(root.path(), StrictnessMode::Strict);
        o.gff3_path = fixture_dir().join("genes_parent_cycle.gff3");
        ingest_dataset(&o).expect_err("cycle must fail in strict mode")
    };
    assert!(err.to_string().contains("cyclic Parent graph"));
    drop(err);
    root.close().expect("close tempdir");
}

#[test]
fn overlapping_gene_ids_across_contigs_requires_explicit_allow() {
    let root = tempdir().expect("tempdir");
    let mut strict = opts(root.path(), StrictnessMode::Strict);
    strict.gff3_path = fixture_dir().join("genes_overlap_contig_ids.gff3");
    let err = ingest_dataset(&strict).expect_err("strict overlap must fail");
    assert!(err.to_string().contains("appears across multiple contigs"));

    let ok_root = tempdir().expect("tempdir2");
    let mut allowed = opts(ok_root.path(), StrictnessMode::Lenient);
    allowed.gff3_path = fixture_dir().join("genes_overlap_contig_ids.gff3");
    allowed.allow_overlap_gene_ids_across_contigs = true;
    let run = ingest_dataset(&allowed).expect("allowed overlap should ingest");
    assert!(!run
        .anomaly_report
        .overlapping_gene_ids_across_contigs
        .is_empty());
}

#[test]
fn contig_coordinate_validation_rejects_out_of_bounds() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.gff3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/contigs/genes_invalid_coord.gff3");
    o.fasta_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contigs/genome.fa");
    o.fai_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contigs/genome.fa.fai");
    assert!(ingest_dataset(&o).is_err());
}

#[test]
fn contig_length_mismatch_between_fasta_and_fai_fails() {
    let root = tempdir().expect("tempdir");
    let gff = root.path().join("genes.gff3");
    let fasta = root.path().join("genome.fa");
    let fai = root.path().join("genome.fa.fai");
    std::fs::write(
            &gff,
            "chr1\tsrc\tgene\t1\t20\t.\t+\t.\tID=g1;Name=G1\nchr1\tsrc\ttranscript\t1\t20\t.\t+\t.\tID=tx1;Parent=g1\n",
        )
        .expect("gff");
    std::fs::write(&fasta, ">chr1\nACGTACGTACGTACGTACGT\n").expect("fasta");
    std::fs::write(&fai, "chr1\t10\n").expect("fai");

    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.gff3_path = gff;
    o.fasta_path = fasta;
    o.fai_path = fai;
    let err = ingest_dataset(&o).expect_err("fai mismatch should fail");
    assert!(err.to_string().contains("exceeds contig"));
}

#[test]
fn gene_out_of_range_fails_contractually() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.gff3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/contigs/genes_invalid_coord.gff3");
    o.fasta_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contigs/genome.fa");
    o.fai_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contigs/genome.fa.fai");
    let err = ingest_dataset(&o).expect_err("out of range must fail");
    assert!(err.to_string().contains("exceeds contig"));
}

#[test]
fn unknown_contig_is_contractual_deterministic_failure() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.gff3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/edgecases/case_9_unknown_contig.gff3");
    let e1 = ingest_dataset(&o).expect_err("unknown contig must fail");
    let e2 = ingest_dataset(&o).expect_err("unknown contig must fail deterministically");
    assert_eq!(e1.to_string(), e2.to_string());
}

#[test]
fn missing_fai_fails_by_default_but_can_autogenerate_in_dev_mode() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.fai_path = root.path().join("autogen.fai");
    let err = ingest_dataset(&o).expect_err("missing fai must fail by default");
    assert!(err.to_string().contains("FAI_REQUIRED_FOR_INGEST"));

    let mut dev = opts(root.path(), StrictnessMode::Strict);
    dev.fai_path = root.path().join("autogen-dev.fai");
    dev.dev_allow_auto_generate_fai = true;
    let run = ingest_dataset(&dev).expect("dev autogen should pass");
    assert!(run.sqlite_path.exists());
    assert!(dev.fai_path.exists());
}

#[test]
fn explain_query_plan_uses_index_strategy() {
    let root = tempdir().expect("tempdir");
    let run = ingest_dataset(&opts(root.path(), StrictnessMode::Strict)).expect("ingest");
    let plans = explain_region_query_plan(&run.sqlite_path).expect("plan");
    let joined = plans.join("\n").to_ascii_lowercase();
    assert!(
        joined.contains("index") || joined.contains("rtree"),
        "expected index/rtree usage in plan: {joined}"
    );
}

#[test]
fn explain_lookup_plans_avoid_full_table_scans() {
    let root = tempdir().expect("tempdir");
    let run = ingest_dataset(&opts(root.path(), StrictnessMode::Strict)).expect("ingest");
    for plan in [
        explain_plan_for_gene_id_query(&run.sqlite_path).expect("gene-id plan"),
        explain_plan_for_name_query(&run.sqlite_path).expect("name plan"),
    ] {
        let joined = plan.join("\n").to_ascii_lowercase();
        assert!(
            joined.contains("index"),
            "expected indexed lookup plan: {joined}"
        );
    }
}

#[test]
fn explain_plan_snapshots_cover_core_query_shapes() {
    let root = tempdir().expect("tempdir");
    let run = ingest_dataset(&opts(root.path(), StrictnessMode::Strict)).expect("ingest");
    let region = explain_region_query_plan(&run.sqlite_path).expect("region");
    let gene = explain_plan_for_gene_id_query(&run.sqlite_path).expect("gene");
    let name = explain_plan_for_name_query(&run.sqlite_path).expect("name");
    let region_txt = region.join("\n").to_ascii_lowercase();
    let gene_txt = gene.join("\n").to_ascii_lowercase();
    let name_txt = name.join("\n").to_ascii_lowercase();
    assert!(
        region_txt.contains("rtree")
            || region_txt.contains("idx_gene_summary_region")
            || region_txt.contains("idx_gene_summary_cover_region")
    );
    assert!(
        gene_txt.contains("idx_gene_summary_gene_id")
            || gene_txt.contains("idx_gene_summary_cover_lookup")
            || gene_txt.contains("idx_gene_summary_cover_region")
    );
    assert!(
        name_txt.contains("idx_gene_summary_name")
            || name_txt.contains("idx_gene_summary_cover_lookup")
            || name_txt.contains("idx_gene_summary_cover_region")
    );
}
