use bijux_atlas_model::{
    parse_assembly, parse_release, parse_species, ArtifactChecksums, ArtifactManifest, Catalog,
    CatalogEntry, DiffPage, DiffRecord, DiffScope, DiffStatus, GeneId, GeneSummary,
    IngestAnomalyReport, ManifestStats, ModelVersion, ReleaseGeneIndex, ReleaseGeneIndexEntry,
    SeqId, ShardCatalog, ShardEntry, ShardId,
};
use std::path::PathBuf;

fn fixture(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(root.join(path)).expect("read fixture")
}

#[test]
fn top_level_models_roundtrip_and_validate() {
    let dataset =
        bijux_atlas_model::DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");

    let mut manifest = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        dataset.clone(),
        ArtifactChecksums::new("a".into(), "b".into(), "c".into(), "d".into()),
        ManifestStats::new(10, 20, 24),
    );
    manifest.input_hashes.gff3_sha256 = "a".into();
    manifest.input_hashes.fasta_sha256 = "b".into();
    manifest.input_hashes.fai_sha256 = "c".into();
    manifest.input_hashes.policy_sha256 = "d".into();
    manifest.db_hash = "d".into();
    manifest.artifact_hash = "x".into();
    manifest.toolchain_hash = "t".into();
    manifest.created_at = "2026-02-24T00:00:00Z".into();
    assert!(manifest.validate().is_ok());
    let manifest_json = serde_json::to_string(&manifest).expect("manifest encode");
    let manifest_back: ArtifactManifest =
        serde_json::from_str(&manifest_json).expect("manifest decode");
    assert_eq!(manifest_back.model_version, ModelVersion::V1);

    let catalog = Catalog::new(vec![CatalogEntry::new(
        dataset.clone(),
        "x/manifest.json".into(),
        "x/gene_summary.sqlite".into(),
    )]);
    assert!(catalog.validate().is_ok());
    let catalog_json = serde_json::to_string(&catalog).expect("catalog encode");
    let _: Catalog = serde_json::from_str(&catalog_json).expect("catalog decode");

    let shard_catalog = ShardCatalog::new(
        dataset.clone(),
        "contig".into(),
        vec![ShardEntry::new(
            ShardId::parse("chr1_000").expect("shard id"),
            vec![SeqId::parse("chr1").expect("seqid")],
            "x/chunk.sqlite".into(),
            "hash".into(),
        )],
    );
    assert!(shard_catalog.validate().is_ok());

    let diff_page = DiffPage::new(
        parse_release("110").expect("release"),
        parse_release("111").expect("release"),
        parse_species("homo_sapiens").expect("species"),
        parse_assembly("GRCh38").expect("assembly"),
        DiffScope::Genes,
        vec![DiffRecord::new(
            GeneId::parse("ENSG000001").expect("gene id"),
            DiffStatus::Changed,
            Some(SeqId::parse("chr1").expect("seqid")),
            Some(10),
            Some(20),
        )],
        None,
    );
    assert!(diff_page.validate().is_ok());

    let release_gene_index = ReleaseGeneIndex::new(
        "1".into(),
        dataset,
        vec![ReleaseGeneIndexEntry::new(
            GeneId::parse("ENSG000001").expect("gene id"),
            SeqId::parse("chr1").expect("seqid"),
            10,
            20,
            "abc".into(),
        )],
    );
    assert!(release_gene_index.validate().is_ok());

    let gene_summary = GeneSummary::new(
        GeneId::parse("ENSG000001").expect("gene id"),
        Some("BRCA1".into()),
        SeqId::parse("chr1").expect("seqid"),
        10,
        20,
        Some("protein_coding".into()),
        1,
        11,
    );
    assert!(gene_summary.validate().is_ok());

    let anomaly = IngestAnomalyReport::new();
    assert!(anomaly.validate().is_ok());
}

#[test]
fn known_current_fixtures_parse() {
    let _: ArtifactManifest =
        serde_json::from_str(&fixture("tests/fixtures/current/artifact_manifest.json"))
            .expect("manifest fixture");
    let _: Catalog = serde_json::from_str(&fixture("tests/fixtures/current/catalog.json"))
        .expect("catalog fixture");
    let _: DiffPage = serde_json::from_str(&fixture("tests/fixtures/current/diff_page.json"))
        .expect("diff fixture");
    let _: ReleaseGeneIndex =
        serde_json::from_str(&fixture("tests/fixtures/current/release_gene_index.json"))
            .expect("index fixture");
}

#[test]
fn backward_compatibility_fixtures_from_v0_1_parse() {
    let manifest: ArtifactManifest =
        serde_json::from_str(&fixture("tests/fixtures/v0_1/artifact_manifest.json"))
            .expect("manifest fixture");
    assert_eq!(manifest.model_version, ModelVersion::V1);

    let catalog: Catalog = serde_json::from_str(&fixture("tests/fixtures/v0_1/catalog.json"))
        .expect("catalog fixture");
    assert_eq!(catalog.model_version, ModelVersion::V1);

    let diff: DiffPage =
        serde_json::from_str(&fixture("tests/fixtures/v0_1/diff_page.json")).expect("diff fixture");
    assert_eq!(diff.model_version, ModelVersion::V1);

    let index: ReleaseGeneIndex =
        serde_json::from_str(&fixture("tests/fixtures/v0_1/release_gene_index.json"))
            .expect("index fixture");
    assert_eq!(index.model_version, ModelVersion::V1);
}
