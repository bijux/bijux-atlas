// SPDX-License-Identifier: Apache-2.0

use super::support::*;

#[test]
fn ingest_sqlite_meta_includes_build_pragmas() {
    let root = tempdir().expect("tempdir");
    let run = ingest_dataset(&opts(root.path(), StrictnessMode::Strict)).expect("ingest");
    let conn = rusqlite::Connection::open(run.sqlite_path).expect("open sqlite");
    let schema_version: String = conn
        .query_row(
            "SELECT v FROM atlas_meta WHERE k='schema_version'",
            [],
            |r| r.get(0),
        )
        .expect("schema_version");
    let journal_mode: String = conn
        .query_row(
            "SELECT v FROM atlas_meta WHERE k='ingest_journal_mode'",
            [],
            |r| r.get(0),
        )
        .expect("journal mode");
    assert_eq!(schema_version, "4");
    assert_eq!(journal_mode, "WAL");
    let schema_table_version: i64 = conn
        .query_row(
            "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .expect("schema_version table");
    assert_eq!(schema_table_version, 4);
    let contigs: i64 = conn
        .query_row("SELECT COUNT(*) FROM contigs", [], |r| r.get(0))
        .expect("contigs table count");
    assert!(contigs > 0);
    let fasta_hash: String = conn
        .query_row("SELECT v FROM atlas_meta WHERE k='fasta_sha256'", [], |r| {
            r.get(0)
        })
        .expect("fasta hash");
    let fai_hash: String = conn
        .query_row("SELECT v FROM atlas_meta WHERE k='fai_sha256'", [], |r| {
            r.get(0)
        })
        .expect("fai hash");
    let created_by: String = conn
        .query_row("SELECT v FROM atlas_meta WHERE k='created_by'", [], |r| {
            r.get(0)
        })
        .expect("created_by");
    assert_eq!(fasta_hash.len(), 64);
    assert_eq!(fai_hash.len(), 64);
    assert_eq!(
        created_by,
        format!("{}@{}", crate::CRATE_NAME, crate::version::runtime_semver())
    );
}

#[test]
fn fixture_matrix_edgecases_runs_leniently() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/edgecases");
    let mut count = 0usize;
    let mut succeeded = 0usize;
    for entry in std::fs::read_dir(dir).expect("read edgecases") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|x| x.to_str()) != Some("gff3") {
            continue;
        }
        let root = tempdir().expect("tempdir");
        let mut o = opts(root.path(), StrictnessMode::Lenient);
        o.gff3_path = path;
        match ingest_dataset(&o) {
            Ok(_) => succeeded += 1,
            Err(err) => {
                let msg = err.to_string();
                assert!(
                    msg.contains("gene_count must be > 0")
                        || msg.contains("contig")
                        || msg.contains("invalid")
                        || msg.contains("GFF3_REFERENCE_NOT_IN_FASTA_FAI"),
                    "unexpected edgecase failure: {msg}"
                );
            }
        }
        count += 1;
    }
    assert!(count >= 10, "expected edgecase fixture matrix coverage");
    assert!(
        succeeded >= 6,
        "expected most edgecases to ingest successfully"
    );
}

#[test]
fn realistic_fixture_smoke_is_deterministic() {
    let realistic = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/realistic");
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Lenient);
    o.gff3_path = realistic.join("genes.gff3");
    o.fasta_path = realistic.join("genome.fa");
    o.fai_path = realistic.join("genome.fa.fai");
    let r1 = ingest_dataset(&o).expect("realistic run1");

    let alt = tempdir().expect("tempdir2");
    o.output_root = alt.path().to_path_buf();
    let r2 = ingest_dataset(&o).expect("realistic run2");

    assert_eq!(
        r1.manifest.checksums.sqlite_sha256,
        r2.manifest.checksums.sqlite_sha256
    );
    let i1 = std::fs::read_to_string(r1.release_gene_index_path).expect("index1");
    let i2 = std::fs::read_to_string(r2.release_gene_index_path).expect("index2");
    assert_eq!(i1, i2);
}

#[test]
fn sharded_ingest_emits_catalog_and_shards() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.sharding_plan = ShardingPlan::Contig;
    o.shard_partitions = 0;
    let run = ingest_dataset(&o).expect("sharded ingest");
    let catalog_path = run.shard_catalog_path.expect("catalog path");
    assert!(catalog_path.exists());
    let catalog = run.shard_catalog.expect("catalog");
    assert!(!catalog.shards.is_empty());
    for shard in &catalog.shards {
        let shard_file = run
            .sqlite_path
            .parent()
            .expect("derived dir")
            .join(&shard.sqlite_path);
        assert!(
            shard_file.exists(),
            "missing shard file {}",
            shard_file.display()
        );
    }
}

#[test]
fn contig_sharding_yields_same_gene_ids_as_monolithic() {
    let realistic = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/realistic");
    let mono_root = tempdir().expect("mono");
    let mut mono_opts = opts(mono_root.path(), StrictnessMode::Lenient);
    mono_opts.gff3_path = realistic.join("genes.gff3");
    mono_opts.fasta_path = realistic.join("genome.fa");
    mono_opts.fai_path = realistic.join("genome.fa.fai");
    let mono = ingest_dataset(&mono_opts).expect("mono");
    let mono_ids = read_gene_ids(&mono.sqlite_path);

    let shard_root = tempdir().expect("shard");
    let mut o = opts(shard_root.path(), StrictnessMode::Lenient);
    o.gff3_path = realistic.join("genes.gff3");
    o.fasta_path = realistic.join("genome.fa");
    o.fai_path = realistic.join("genome.fa.fai");
    o.sharding_plan = ShardingPlan::Contig;
    let sharded = ingest_dataset(&o).expect("sharded");
    let catalog = sharded.shard_catalog.expect("catalog");
    assert!(
        catalog.shards.len() > 1,
        "fixture should produce multi-contig shards"
    );
    let mut shard_ids = std::collections::BTreeSet::new();
    for shard in &catalog.shards {
        let p = sharded
            .sqlite_path
            .parent()
            .expect("derived")
            .join(&shard.sqlite_path);
        for id in read_gene_ids(&p) {
            shard_ids.insert(id);
        }
    }

    assert_eq!(mono_ids, shard_ids);
}

#[test]
fn sharding_respects_max_shards_policy() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.sharding_plan = ShardingPlan::Contig;
    o.max_shards = 1;
    let err = ingest_dataset(&o).expect_err("expected max_shards failure");
    assert!(err.to_string().contains("max_shards"));
}

fn read_gene_ids(path: &std::path::Path) -> std::collections::BTreeSet<String> {
    let conn = rusqlite::Connection::open(path).expect("open sqlite");
    let mut stmt = conn
        .prepare("SELECT gene_id FROM gene_summary ORDER BY gene_id")
        .expect("prepare");
    stmt.query_map([], |r| r.get::<_, String>(0))
        .expect("query")
        .collect::<Result<std::collections::BTreeSet<_>, _>>()
        .expect("collect ids")
}

#[test]
fn unknown_feature_policy_can_reject() {
    let root = tempdir().expect("tempdir");
    let gff = root.path().join("unknown_feature.gff3");
    std::fs::write(
            &gff,
            "chr1\tsrc\tgene\t1\t10\t.\t+\t.\tID=g1;Name=G1\nchr1\tsrc\trepeat_region\t1\t10\t.\t+\t.\tID=r1\n",
        )
        .expect("write gff3");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.gff3_path = gff;
    o.unknown_feature_policy = UnknownFeaturePolicy::Reject;
    let err = ingest_dataset(&o).expect_err("unknown feature should fail");
    assert!(err.to_string().contains("unknown GFF3 feature type"));
}

#[test]
fn transcript_id_policy_supports_transcript_id_attribute() {
    let root = tempdir().expect("tempdir");
    let gff = root.path().join("transcript_id_variant.gff3");
    std::fs::write(
            &gff,
            "chr1\tsrc\tgene\t1\t50\t.\t+\t.\tID=g1;Name=G1\nchr1\tsrc\ttranscript\t1\t50\t.\t+\t.\ttranscript_id=tx1;Parent=g1\n",
        )
        .expect("write gff3");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.gff3_path = gff;
    let run = ingest_dataset(&o).expect("ingest with transcript_id attr");
    assert_eq!(run.manifest.stats.transcript_count, 1);
}

#[test]
fn feature_id_uniqueness_policy_normalized_rejects_case_collisions() {
    let root = tempdir().expect("tempdir");
    let gff = root.path().join("id_case_collision.gff3");
    std::fs::write(
            &gff,
            "chr1\tsrc\tgene\t1\t50\t.\t+\t.\tID=g1;Name=G1\nchr1\tsrc\ttranscript\t1\t50\t.\t+\t.\tID=Tx1;Parent=g1\nchr1\tsrc\ttranscript\t2\t40\t.\t+\t.\tID=tx1;Parent=g1\n",
        )
        .expect("write gff3");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.gff3_path = gff;
    o.feature_id_uniqueness_policy = FeatureIdUniquenessPolicy::NormalizeAsciiLowercaseReject;
    let err = ingest_dataset(&o).expect_err("case-colliding IDs must fail");
    assert!(err.to_string().contains("duplicate feature ID"));
}

#[test]
fn tiny_fixture_matches_cross_machine_golden_hashes() {
    const SQLITE_LOGICAL_FINGERPRINT_SHA256: &str =
        "67a2095680d2ce06ff89ea15ddeafa3fc13c1552452d44c7a4cd7c0a444b413a";
    const DATASET_SIGNATURE_SHA256: &str =
        "d273f80509add42ab87efe008c5ad2361e0efb411b295e7a9a8371ec281bd9df";

    let root = tempdir().expect("tempdir");
    let run = ingest_dataset(&opts(root.path(), StrictnessMode::Strict)).expect("ingest");
    let sqlite_fingerprint = sqlite_logical_fingerprint(&run.sqlite_path);
    assert_eq!(
        sqlite_fingerprint, SQLITE_LOGICAL_FINGERPRINT_SHA256,
        "sqlite logical fingerprint drifted; update the golden only after confirming \
         schema/content changes are intentional"
    );
    assert_eq!(
        run.manifest.dataset_signature_sha256,
        DATASET_SIGNATURE_SHA256
    );
}

#[test]
fn strict_mode_rejects_invalid_strand() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.gff3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/policies/invalid_strand.gff3");
    let err = ingest_dataset(&o).expect_err("invalid strand must fail");
    assert!(err.to_string().contains("GFF3_INVALID_STRAND"));
}

#[test]
fn strict_mode_rejects_invalid_cds_phase() {
    let root = tempdir().expect("tempdir");
    let mut o = opts(root.path(), StrictnessMode::Strict);
    o.gff3_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/policies/invalid_cds_phase.gff3");
    let err = ingest_dataset(&o).expect_err("invalid phase must fail");
    assert!(err.to_string().contains("GFF3_INVALID_PHASE"));
}
