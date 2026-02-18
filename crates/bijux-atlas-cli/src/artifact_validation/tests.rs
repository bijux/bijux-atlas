use super::{
    build_release_diff, compute_gc_plan, validate_qc_thresholds, BuildReleaseDiffArgs, OutputMode,
};
use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{Catalog, CatalogEntry, DatasetId};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn qc_thresholds_pass_for_healthy_report() {
    let qc = json!({
        "counts": {"genes": 10, "transcripts": 20, "exons": 50, "cds": 12},
        "orphan_counts": {"transcripts": 0},
        "duplicate_id_events": {"duplicate_gene_ids": 0},
        "rejected_record_count_by_reason": {"GFF3_UNKNOWN_FEATURE": 0},
        "contig_stats": {"unknown_contig_feature_ratio": 0.0, "total_features": 100}
    });
    let t = json!({
        "min_gene_count": 1,
        "max_orphan_percent": 1.0,
        "max_rejected_percent": 1.0,
        "max_unknown_contig_feature_percent": 0.5,
        "max_duplicate_gene_id_events": 0
    });
    assert!(validate_qc_thresholds(&qc, &t).is_ok());
}

#[test]
fn qc_thresholds_fail_when_orphan_rate_exceeds_max() {
    let qc = json!({
        "counts": {"genes": 10, "transcripts": 10, "exons": 10, "cds": 10},
        "orphan_counts": {"transcripts": 2},
        "duplicate_id_events": {"duplicate_gene_ids": 0},
        "rejected_record_count_by_reason": {},
        "contig_stats": {"unknown_contig_feature_ratio": 0.0, "total_features": 100}
    });
    let t = json!({
        "min_gene_count": 1,
        "max_orphan_percent": 10.0,
        "max_rejected_percent": 10.0,
        "max_unknown_contig_feature_percent": 10.0,
        "max_duplicate_gene_id_events": 0
    });
    let err = validate_qc_thresholds(&qc, &t).expect_err("orphan gate must fail");
    assert!(err.contains("orphan_percent"));
}

#[test]
fn qc_edgecase_fixture_orphan_rate_regression_is_rejected() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let qc = serde_json::from_slice::<serde_json::Value>(
        &std::fs::read(root.join("tests/fixtures/qc_edgecases/qc_orphan_high.json"))
            .expect("read qc fixture"),
    )
    .expect("parse qc fixture");
    let t = serde_json::from_slice::<serde_json::Value>(
        &std::fs::read(root.join("tests/fixtures/qc_edgecases/thresholds_strict.json"))
            .expect("read threshold fixture"),
    )
    .expect("parse threshold fixture");
    assert!(validate_qc_thresholds(&qc, &t).is_err());
}

#[test]
fn diff_build_is_deterministic_for_same_inputs() {
    let tmp = tempdir().expect("tmp");
    let root = tmp.path();
    let from = root.join("release=110/species=homo_sapiens/assembly=GRCh38/derived");
    let to = root.join("release=111/species=homo_sapiens/assembly=GRCh38/derived");
    fs::create_dir_all(&from).expect("create from");
    fs::create_dir_all(&to).expect("create to");

    fs::write(
            from.join("release_gene_index.json"),
            r#"{"schema_version":"1","dataset":{"release":"110","species":"homo_sapiens","assembly":"GRCh38"},"entries":[{"gene_id":"ENSG1","seqid":"chr1","start":10,"end":20,"signature_sha256":"a"},{"gene_id":"ENSG2","seqid":"chr1","start":30,"end":40,"signature_sha256":"b"}]}"#,
        )
        .expect("write from index");
    fs::write(
            to.join("release_gene_index.json"),
            r#"{"schema_version":"1","dataset":{"release":"111","species":"homo_sapiens","assembly":"GRCh38"},"entries":[{"gene_id":"ENSG1","seqid":"chr1","start":10,"end":25,"signature_sha256":"z"},{"gene_id":"ENSG3","seqid":"chr2","start":5,"end":8,"signature_sha256":"c"}]}"#,
        )
        .expect("write to index");

    write_sqlite(
        &from.join("gene_summary.sqlite"),
        &[("ENSG1", "protein_coding"), ("ENSG2", "lncRNA")],
    );
    write_sqlite(
        &to.join("gene_summary.sqlite"),
        &[("ENSG1", "protein_coding"), ("ENSG3", "miRNA")],
    );

    let out1 = root.join("diff-out-1");
    let out2 = root.join("diff-out-2");
    build_release_diff(
        BuildReleaseDiffArgs {
            root: root.to_path_buf(),
            from_release: "110".to_string(),
            to_release: "111".to_string(),
            species: "homo_sapiens".to_string(),
            assembly: "GRCh38".to_string(),
            out_dir: out1.clone(),
            max_inline_items: 100,
        },
        OutputMode { json: true },
    )
    .expect("build diff #1");
    build_release_diff(
        BuildReleaseDiffArgs {
            root: root.to_path_buf(),
            from_release: "110".to_string(),
            to_release: "111".to_string(),
            species: "homo_sapiens".to_string(),
            assembly: "GRCh38".to_string(),
            out_dir: out2.clone(),
            max_inline_items: 100,
        },
        OutputMode { json: true },
    )
    .expect("build diff #2");
    let d1 = fs::read(out1.join("diff.json")).expect("read diff1");
    let d2 = fs::read(out2.join("diff.json")).expect("read diff2");
    assert_eq!(d1, d2, "diff output must be byte-identical");
    assert!(!sha256_hex(&d1).is_empty());
}

fn write_sqlite(path: &std::path::Path, rows: &[(&str, &str)]) {
    let conn = rusqlite::Connection::open(path).expect("open sqlite");
    conn.execute(
        "CREATE TABLE gene_summary (gene_id TEXT NOT NULL, biotype TEXT NOT NULL)",
        [],
    )
    .expect("create table");
    for (gene_id, biotype) in rows {
        conn.execute(
            "INSERT INTO gene_summary(gene_id, biotype) VALUES (?1, ?2)",
            [gene_id, biotype],
        )
        .expect("insert");
    }
}

#[test]
fn gc_plan_respects_catalog_and_dataset_pins() {
    let tmp = tempdir().expect("tmp");
    let root = tmp.path().join("store");
    fs::create_dir_all(&root).expect("mkdir");

    let reachable = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let pinned = DatasetId::new("111", "homo_sapiens", "GRCh38").expect("dataset");
    let candidate = DatasetId::new("112", "homo_sapiens", "GRCh38").expect("dataset");
    for d in [&reachable, &pinned, &candidate] {
        let p = bijux_atlas_model::artifact_paths(&root, d);
        fs::create_dir_all(p.sqlite.parent().expect("parent")).expect("mkdir derived");
        fs::write(&p.sqlite, b"sqlite").expect("write sqlite");
    }

    let catalog = Catalog::new(vec![CatalogEntry::new(
        reachable.clone(),
        bijux_atlas_model::artifact_paths(&root, &reachable)
            .manifest
            .strip_prefix(&root)
            .expect("strip")
            .display()
            .to_string(),
        bijux_atlas_model::artifact_paths(&root, &reachable)
            .sqlite
            .strip_prefix(&root)
            .expect("strip")
            .display()
            .to_string(),
    )]);
    let catalog_path = root.join("catalog.json");
    fs::write(
        &catalog_path,
        bijux_atlas_store::canonical_catalog_json(&catalog).expect("catalog json"),
    )
    .expect("write catalog");
    let pins_path = tmp.path().join("pins.json");
    fs::write(
        &pins_path,
        serde_json::to_vec(&json!({
            "dataset_ids":[format!(
                "release={}&species={}&assembly={}",
                pinned.release, pinned.species, pinned.assembly
            )],
            "artifact_hashes":[]
        }))
        .expect("pins json"),
    )
    .expect("write pins");

    let report = compute_gc_plan(&root, &[catalog_path], &pins_path).expect("gc plan");
    assert_eq!(report.candidates.dataset_roots.len(), 1);
    let expected_paths = bijux_atlas_model::artifact_paths(&root, &candidate);
    let expected_root = expected_paths
        .manifest
        .parent()
        .and_then(|p| p.parent())
        .expect("dataset root");
    let actual_root = PathBuf::from(&report.candidates.dataset_roots[0])
        .canonicalize()
        .expect("canonical candidate");
    let expected_root = expected_root.canonicalize().expect("canonical expected");
    assert_eq!(actual_root, expected_root);
}

#[test]
fn gc_plan_multiple_catalog_paths_are_deterministic() {
    let tmp = tempdir().expect("tmp");
    let root = tmp.path().join("store");
    fs::create_dir_all(&root).expect("mkdir");

    let d1 = DatasetId::new("200", "homo_sapiens", "GRCh38").expect("dataset");
    let d2 = DatasetId::new("201", "homo_sapiens", "GRCh38").expect("dataset");
    for d in [&d1, &d2] {
        let p = bijux_atlas_model::artifact_paths(&root, d);
        fs::create_dir_all(p.sqlite.parent().expect("parent")).expect("mkdir derived");
        fs::write(&p.sqlite, b"sqlite").expect("write sqlite");
    }

    let write_catalog = |name: &str, dataset: &DatasetId| -> PathBuf {
        let c = Catalog::new(vec![CatalogEntry::new(
            dataset.clone(),
            bijux_atlas_model::artifact_paths(&root, dataset)
                .manifest
                .strip_prefix(&root)
                .expect("strip")
                .display()
                .to_string(),
            bijux_atlas_model::artifact_paths(&root, dataset)
                .sqlite
                .strip_prefix(&root)
                .expect("strip")
                .display()
                .to_string(),
        )]);
        let path = root.join(name);
        fs::write(
            &path,
            bijux_atlas_store::canonical_catalog_json(&c).expect("catalog json"),
        )
        .expect("write catalog");
        path
    };
    let c1 = write_catalog("catalog-a.json", &d1);
    let c2 = write_catalog("catalog-b.json", &d2);
    let pins_path = tmp.path().join("pins.json");
    fs::write(&pins_path, br#"{"dataset_ids":[],"artifact_hashes":[]}"#).expect("pins");

    let r1 = compute_gc_plan(&root, &[c2.clone(), c1.clone(), c2.clone()], &pins_path).expect("gc");
    let r2 = compute_gc_plan(&root, &[c1, c2], &pins_path).expect("gc");
    assert_eq!(r1.catalogs, r2.catalogs);
    assert_eq!(r1.candidates.dataset_roots, r2.candidates.dataset_roots);
}
