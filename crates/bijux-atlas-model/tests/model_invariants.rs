use bijux_atlas_model::{
    normalize_assembly, normalize_release, normalize_species, parse_species_normalized,
    BiotypePolicy, DatasetId, DatasetSelector, GeneId, GeneNamePolicy, Region, SeqId, Strand,
    TranscriptId, TranscriptTypePolicy, ID_MAX_LEN, SEQID_MAX_LEN,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn dataset_id_canonical_string_is_stable() {
    let id = DatasetId::from_normalized("110", "Homo-Sapiens", "GRCh38").expect("dataset id");
    assert_eq!(id.canonical_string(), "110/homo_sapiens/GRCh38");
}

#[test]
fn dataset_new_does_not_apply_implicit_normalization() {
    assert!(DatasetId::new("110", "Homo-sapiens", "GRCh38").is_err());
    assert!(DatasetId::new("110", "homo_sapiens", "GRCh38").is_ok());
}

#[test]
fn release_species_assembly_parsing_is_strict() {
    assert_eq!(normalize_release("110").expect("release"), "110");
    assert!(normalize_species("Homo-sapiens").is_ok());
    assert_eq!(normalize_assembly("GRCh38").expect("assembly"), "GRCh38");

    assert!(normalize_release("11a").is_err());
    assert!(normalize_species("homo sapiens").is_err());
    assert!(normalize_assembly("GRCh38!").is_err());
    assert_eq!(
        parse_species_normalized("Homo-sapiens")
            .expect("normalized parse")
            .as_str(),
        "homo_sapiens"
    );
}

#[test]
fn gene_id_rejects_hidden_trimming() {
    assert!(GeneId::parse("ENSG000001").is_ok());
    assert!(GeneId::parse(" ENSG000001").is_err());
    assert!(GeneId::parse("ENSG000001 ").is_err());
}

#[test]
fn seqid_rejects_hidden_trimming() {
    assert!(SeqId::parse("chr1").is_ok());
    assert!(SeqId::parse(" chr1").is_err());
    assert!(SeqId::parse("chr1 ").is_err());
}

#[test]
fn transcript_id_rejects_hidden_trimming() {
    assert!(TranscriptId::parse("ENST000001").is_ok());
    assert!(TranscriptId::parse(" ENST000001").is_err());
    assert!(TranscriptId::parse("ENST000001 ").is_err());
}

#[test]
fn region_and_strand_invariants_hold() {
    let region = Region::parse("chr1:10-20").expect("region parse");
    assert_eq!(region.canonical_string(), "chr1:10-20");
    assert!(Region::parse("chr1:20-10").is_err());
    assert!(Strand::parse("+").is_ok());
    assert!(Strand::parse("-").is_ok());
    assert!(Strand::parse(".").is_ok());
    assert!(Strand::parse("x").is_err());
}

#[test]
fn max_size_limits_are_enforced() {
    let too_long_gene_id = "g".repeat(ID_MAX_LEN + 1);
    assert!(GeneId::parse(&too_long_gene_id).is_err());
    let too_long_seqid = "c".repeat(SEQID_MAX_LEN + 1);
    assert!(SeqId::parse(&too_long_seqid).is_err());
}

#[test]
fn no_implicit_latest_selector_contract_is_explicit_only() {
    let id = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let selector = DatasetSelector::Explicit(id.clone());
    assert_eq!(selector, DatasetSelector::Explicit(id));
}

#[test]
fn gene_name_biotype_and_transcript_policies_remain_deterministic() {
    let mut attrs = BTreeMap::new();
    attrs.insert("Name".to_string(), " BRCA1 ".to_string());
    attrs.insert("gene_biotype".to_string(), "protein_coding".to_string());

    assert_eq!(
        GeneNamePolicy::default().resolve(&attrs, "fallback"),
        "BRCA1"
    );
    assert_eq!(BiotypePolicy::default().resolve(&attrs), "protein_coding");

    let accepted = BTreeSet::from(["mRNA".to_string(), "transcript".to_string()]);
    let transcript_policy = TranscriptTypePolicy::from_types(accepted);
    assert!(transcript_policy.accepts("mRNA"));
    assert!(!transcript_policy.accepts("gene"));
}
