// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_model::{
    ArtifactChecksums, ArtifactManifest, BiotypePolicy, Catalog, CatalogEntry, DatasetId,
    GeneNamePolicy, ManifestStats, OptionalFieldPolicy, SeqidNormalizationPolicy,
    TranscriptTypePolicy,
};

#[test]
fn manifest_rejects_unknown_fields() {
    let raw = r#"{
      \"manifest_version\":\"1\",
      \"db_schema_version\":\"1\",
      \"dataset\":{\"release\":\"110\",\"species\":\"homo_sapiens\",\"assembly\":\"GRCh38\"},
      \"checksums\":{\"gff3_sha256\":\"a\",\"fasta_sha256\":\"b\",\"fai_sha256\":\"c\",\"sqlite_sha256\":\"d\"},
      \"stats\":{\"gene_count\":1,\"transcript_count\":1,\"contig_count\":1},
      \"extra\":\"nope\"
    }"#;
    assert!(serde_json::from_str::<ArtifactManifest>(raw).is_err());
}

#[test]
fn catalog_rejects_unknown_fields() {
    let raw = r#"{\"datasets\":[],\"extra\":1}"#;
    assert!(serde_json::from_str::<Catalog>(raw).is_err());
}

#[test]
fn round_trip_public_manifest_and_catalog_types() {
    let manifest = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset"),
        ArtifactChecksums::new(
            "a".repeat(64),
            "b".repeat(64),
            "c".repeat(64),
            "d".repeat(64),
        ),
        ManifestStats::new(1, 2, 3),
    );

    let manifest_json = serde_json::to_string(&manifest).expect("manifest encode");
    let decoded_manifest: ArtifactManifest =
        serde_json::from_str(&manifest_json).expect("manifest decode");
    assert_eq!(manifest, decoded_manifest);

    let catalog = Catalog::new(vec![CatalogEntry::new(
        DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset"),
        "x/manifest.json".to_string(),
        "x/gene_summary.sqlite".to_string(),
    )]);
    let catalog_json = serde_json::to_string(&catalog).expect("catalog encode");
    let decoded_catalog: Catalog = serde_json::from_str(&catalog_json).expect("catalog decode");
    assert_eq!(catalog, decoded_catalog);

    let policy = OptionalFieldPolicy::NullWhenMissing;
    let policy_json = serde_json::to_string(&policy).expect("policy encode");
    let decoded_policy: OptionalFieldPolicy =
        serde_json::from_str(&policy_json).expect("policy decode");
    assert_eq!(policy, decoded_policy);
}

#[test]
fn legacy_manifest_v1_without_new_fields_is_still_compatible() {
    let raw = r#"{
      "manifest_version":"1",
      "db_schema_version":"1",
      "dataset":{"release":"110","species":"homo_sapiens","assembly":"GRCh38"},
      "checksums":{"gff3_sha256":"a","fasta_sha256":"b","fai_sha256":"c","sqlite_sha256":"d"},
      "stats":{"gene_count":1,"transcript_count":1,"contig_count":1}
    }"#;
    let manifest: ArtifactManifest = serde_json::from_str(raw).expect("legacy parse");
    assert!(manifest.dataset_signature_sha256.is_empty());
    assert!(manifest.db_hash.is_empty());
    assert!(manifest.artifact_hash.is_empty());
    assert!(!manifest.derived_column_origins.is_empty());
}

#[test]
fn strict_manifest_validation_requires_schema_consistency() {
    let mut manifest = ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset"),
        ArtifactChecksums::new(
            "a".repeat(64),
            "b".repeat(64),
            "c".repeat(64),
            "d".repeat(64),
        ),
        ManifestStats::new(1, 2, 3),
    );
    manifest.input_hashes.gff3_sha256 = "a".repeat(64);
    manifest.input_hashes.fasta_sha256 = "b".repeat(64);
    manifest.input_hashes.fai_sha256 = "c".repeat(64);
    manifest.input_hashes.policy_sha256 = "d".repeat(64);
    manifest.toolchain_hash = "e".repeat(64);
    manifest.db_hash = "d".repeat(64);
    manifest.artifact_hash = "f".repeat(64);
    manifest.db_schema_version = "2".to_string();
    assert!(manifest.validate_strict().is_err());
}

#[test]
fn policy_structs_reject_unknown_fields_and_optional_policy_is_enforced() {
    assert!(
        serde_json::from_str::<GeneNamePolicy>(r#"{"attribute_keys":["Name"],"x":1}"#).is_err()
    );
    assert!(serde_json::from_str::<BiotypePolicy>(
        r#"{"attribute_keys":["gene_biotype"],"unknown_value":"unknown","x":1}"#
    )
    .is_err());
    assert!(serde_json::from_str::<TranscriptTypePolicy>(
        r#"{"accepted_types":["transcript"],"x":1}"#
    )
    .is_err());
    assert!(
        serde_json::from_str::<SeqidNormalizationPolicy>(r#"{"aliases":{"chr1":"1"},"x":1}"#)
            .is_err()
    );

    let mut map = serde_json::Map::new();
    OptionalFieldPolicy::NullWhenMissing.apply_to_json_map(&mut map, "name", None);
    assert!(matches!(map.get("name"), Some(serde_json::Value::Null)));
    let mut map = serde_json::Map::new();
    OptionalFieldPolicy::OmitWhenMissing.apply_to_json_map(&mut map, "name", None);
    assert!(!map.contains_key("name"));
}
