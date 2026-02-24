use bijux_atlas_model::{
    ArtifactChecksums, ArtifactManifest, DatasetId, DiffPage, DiffScope, DiffStatus, DiffRecord,
    GeneId, ManifestStats, SeqId, parse_assembly, parse_release, parse_species,
};

#[test]
fn manifest_validate_rejects_empty_hashes() {
    let manifest = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset"),
        ArtifactChecksums::new("a".into(), "b".into(), "c".into(), "d".into()),
        ManifestStats::new(1, 2, 3),
    );
    assert!(manifest.validate().is_err());
}

#[test]
fn diff_page_validate_requires_rows() {
    let diff = DiffPage::new(
        parse_release("110").expect("from"),
        parse_release("111").expect("to"),
        parse_species("homo_sapiens").expect("species"),
        parse_assembly("GRCh38").expect("assembly"),
        DiffScope::Genes,
        vec![],
        None,
    );
    assert!(diff.validate().is_err());
}

#[test]
fn diff_page_validate_accepts_non_empty_rows() {
    let diff = DiffPage::new(
        parse_release("110").expect("from"),
        parse_release("111").expect("to"),
        parse_species("homo_sapiens").expect("species"),
        parse_assembly("GRCh38").expect("assembly"),
        DiffScope::Genes,
        vec![DiffRecord::new(
            GeneId::parse("ENSG000001").expect("gene"),
            DiffStatus::Changed,
            Some(SeqId::parse("chr1").expect("seqid")),
            Some(1),
            Some(2),
        )],
        None,
    );
    assert!(diff.validate().is_ok());
}
