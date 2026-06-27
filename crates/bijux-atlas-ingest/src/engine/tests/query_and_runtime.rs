// SPDX-License-Identifier: Apache-2.0

use super::support::*;

#[test]
fn duplicate_transcript_policy_rejects_in_strict_mode() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.gff3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/policies/duplicate_transcript_ids.gff3");
    assert!(ingest_dataset(&o).is_err());
}

#[test]
fn duplicate_transcript_policy_can_dedupe() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Lenient);
    o.gff3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/policies/duplicate_transcript_ids.gff3");
    o.duplicate_transcript_id_policy =
        DuplicateTranscriptIdPolicy::DedupeKeepLexicographicallySmallest;
    let run = ingest_dataset(&o).expect("dedupe policy should pass");
    let conn = rusqlite::Connection::open(run.sqlite_path).expect("open sqlite");
    let tx_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM transcript_summary", [], |r| r.get(0))
        .expect("tx count");
    assert_eq!(tx_count, 1);
}

#[test]
fn feature_ordering_independence_holds() {
    let root_a = tempdir().expect("tempdir");
    let run_a = ingest_dataset(&opts(root_a.path(), StrictnessMode::Strict)).expect("baseline");
    let root_b = tempdir().expect("tempdir2");
    let mut o = opts(root_b.path(), StrictnessMode::Strict);
    o.gff3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/policies/unordered_features.gff3");
    let run_b1 = ingest_dataset(&o).expect("unordered ingest #1");
    let root_c = tempdir().expect("tempdir3");
    o.output_root = root_c.path().to_path_buf();
    let run_b2 = ingest_dataset(&o).expect("unordered ingest #2");
    assert_eq!(
        run_b1.manifest.checksums.sqlite_sha256,
        run_b2.manifest.checksums.sqlite_sha256
    );
    assert_eq!(run_a.manifest.stats, run_b1.manifest.stats);
    assert_eq!(
        run_b1.manifest.canonical_query_semantic_sha256,
        run_b2.manifest.canonical_query_semantic_sha256
    );
    assert_eq!(
        run_b1.manifest.canonical_lineage_sha256,
        run_b2.manifest.canonical_lineage_sha256
    );
    let summary_a = read_canonical_summary(&run_b1);
    let summary_b = read_canonical_summary(&run_b2);
    assert_eq!(
        summary_a.pointer("/hashes/query_semantic_sha256"),
        summary_b.pointer("/hashes/query_semantic_sha256")
    );
    assert_eq!(
        summary_a.pointer("/summary/feature_type_counts/gene"),
        summary_b.pointer("/summary/feature_type_counts/gene")
    );
}

#[test]
fn canonical_summary_and_manifest_hashes_are_emitted() {
    let root = tempdir().expect("tempdir");
    let run = ingest_dataset(&opts(root.path(), StrictnessMode::Strict)).expect("ingest");
    let summary = read_canonical_summary(&run);
    let gene_count = summary
        .pointer("/summary/genes")
        .and_then(serde_json::Value::as_u64)
        .expect("summary genes");
    assert_eq!(gene_count, run.manifest.stats.gene_count);
    assert_eq!(
        summary.pointer("/hashes/query_semantic_sha256"),
        Some(&serde_json::json!(run
            .manifest
            .canonical_query_semantic_sha256
            .clone()))
    );
    assert_eq!(
        summary.pointer("/hashes/lineage_sensitive_sha256"),
        Some(&serde_json::json!(run
            .manifest
            .canonical_lineage_sha256
            .clone()))
    );
    assert_eq!(run.manifest.canonical_model_schema_version, 1);
    assert_eq!(
        run.manifest.canonical_feature_summary_path,
        "derived/canonical_summary.json".to_string()
    );
}

#[test]
fn canonical_query_semantic_hash_is_stable_across_repeated_runs() {
    let root_a = tempdir().expect("tempdir");
    let run_a = ingest_dataset(&opts(root_a.path(), StrictnessMode::Strict)).expect("run a");
    let root_b = tempdir().expect("tempdir");
    let run_b = ingest_dataset(&opts(root_b.path(), StrictnessMode::Strict)).expect("run b");
    assert_eq!(
        run_a.manifest.canonical_query_semantic_sha256,
        run_b.manifest.canonical_query_semantic_sha256
    );
    assert_eq!(
        run_a.manifest.canonical_lineage_sha256,
        run_b.manifest.canonical_lineage_sha256
    );
}

#[test]
fn report_contains_structured_rejections() {
    let root = tempdir().expect("tempdir");
    let gff = root.path().join("unknown_feature_lenient.gff3");
    std::fs::write(
            &gff,
            "chr1\tsrc\tgene\t1\t10\t.\t+\t.\tID=g1;Name=G1\nchr1\tsrc\trepeat_region\t1\t10\t.\t+\t.\tID=r1\n",
        )
        .expect("write gff3");
    let mut o = opts(root.path(), StrictnessMode::Lenient);
    o.gff3_path = gff;
    let run = ingest_dataset(&o).expect("lenient ingest");
    assert!(!run.anomaly_report.rejections.is_empty());
    assert_eq!(
        run.anomaly_report.rejections[0].code,
        "GFF3_UNKNOWN_FEATURE"
    );
    let qc: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&run.qc_report_path).expect("read qc report"))
            .expect("parse qc report");
    assert!(
        qc.pointer("/anomaly_classes/rejections")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            > 0
    );
    assert_eq!(
        qc.pointer("/anomaly_classes/unknown_feature_types")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
}

#[test]
fn manifest_stores_contig_normalization_aliases() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.seqid_policy = SeqidNormalizationPolicy::from_aliases(std::collections::BTreeMap::from([(
        "1".to_string(),
        "chr1".to_string(),
    )]));
    let run = ingest_dataset(&o).expect("ingest");
    assert_eq!(
        run.manifest
            .contig_normalization_aliases
            .get("1")
            .map(String::as_str),
        Some("chr1")
    );
}

#[test]
fn scientific_fixture_emits_contig_classes_and_reference_build_identity() {
    let root = tempdir().expect("tempdir");
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/scientific");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.gff3_path = fixtures.join("annotations.gff3");
    o.fasta_path = fixtures.join("genome.fa");
    o.fai_path = fixtures.join("genome.fa.fai");
    let run = ingest_dataset(&o).expect("scientific ingest");

    assert!(!run.manifest.reference_build_identity_sha256.is_empty());
    assert_eq!(run.manifest.coordinate_system, "1-based-closed");
    assert_eq!(run.manifest.scientific_prerequisites_status, "complete");
    assert_eq!(
        run.manifest.scientific_profile_path,
        "derived/scientific_profile.json".to_string()
    );

    let scientific_profile_path = run
        .manifest_path
        .parent()
        .expect("manifest dir")
        .join("scientific_profile.json");
    let profile: serde_json::Value = serde_json::from_slice(
        &std::fs::read(scientific_profile_path).expect("read scientific profile"),
    )
    .expect("parse scientific profile");
    assert_eq!(
        profile["coordinate_system"].as_str(),
        Some("1-based-closed")
    );
    assert!(
        profile["contig_class_distribution"]["mitochondrial"]
            .as_u64()
            .unwrap_or(0)
            > 0
    );
    assert!(
        profile["contig_class_distribution"]["plasmid"]
            .as_u64()
            .unwrap_or(0)
            > 0
    );
    assert!(
        profile["contig_class_distribution"]["scaffold"]
            .as_u64()
            .unwrap_or(0)
            > 0
    );
    assert!(
        profile["contig_class_distribution"]["alternate"]
            .as_u64()
            .unwrap_or(0)
            > 0
    );
}

#[test]
fn scientific_incoherent_contig_naming_is_rejected_without_alias() {
    let root = tempdir().expect("tempdir");
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/scientific");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.gff3_path = fixtures.join("incoherent_mixed_core_names.gff3");
    o.fasta_path = fixtures.join("incoherent_genome.fa");
    o.fai_path = fixtures.join("incoherent_genome.fa.fai");
    let err = ingest_dataset(&o).expect_err("incoherent naming should fail");
    assert!(err
        .to_string()
        .contains("SCIENTIFIC_INCOHERENT_SOURCE_COMBINATION"));
}

#[test]
fn scientific_incoherent_contig_naming_can_be_resolved_by_alias_policy() {
    let root = tempdir().expect("tempdir");
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/scientific");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.gff3_path = fixtures.join("incoherent_mixed_core_names.gff3");
    o.fasta_path = fixtures.join("incoherent_genome.fa");
    o.fai_path = fixtures.join("incoherent_genome.fa.fai");
    o.seqid_policy = SeqidNormalizationPolicy::from_aliases(std::collections::BTreeMap::from([(
        "1".to_string(),
        "chr1".to_string(),
    )]));
    o.reject_normalized_seqid_collisions = false;
    let run = ingest_dataset(&o).expect("alias mapping should restore coherence");
    assert_eq!(run.manifest.scientific_prerequisites_status, "insufficient");
    assert!(
        run.anomaly_report
            .scientific_ambiguities
            .iter()
            .any(|entry| entry.contains("multiple_source_seqids_for_normalized:chr1")),
        "normalized seqid collision should be preserved as scientific evidence"
    );
}

#[test]
fn unknown_biotype_is_recorded_as_scientific_ambiguity() {
    let root = tempdir().expect("tempdir");
    let gff = root.path().join("unknown-biotype.gff3");
    std::fs::write(
        &gff,
        "##gff-version 3\nchr1\tsrc\tgene\t1\t20\t.\t+\t.\tID=g1;Name=G1\nchr1\tsrc\tmRNA\t1\t20\t.\t+\t.\tID=tx1;Parent=g1\n",
    )
    .expect("write gff");
    let mut o = opts(root.path(), StrictnessMode::Lenient);
    o.gff3_path = gff;
    let run = ingest_dataset(&o).expect("ingest");
    assert!(
        !run.anomaly_report.scientific_ambiguities.is_empty(),
        "scientific ambiguity evidence should be emitted"
    );
}
