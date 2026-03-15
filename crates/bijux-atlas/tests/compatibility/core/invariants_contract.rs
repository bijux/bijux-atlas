// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::dataset::{DatasetId, ShardId};

#[test]
fn dataset_id_rejects_invalid_characters() {
    let err = DatasetId::new("110", "Human Genome", "GRCh38")
        .expect_err("must reject spaces and uppercase");
    assert!(err.to_string().contains("species"));
}

#[test]
fn shard_id_rejects_empty() {
    let err = ShardId::parse("").expect_err("must reject empty");
    assert!(err.to_string().contains("must not be empty"));
}

#[test]
fn dataset_id_accepts_valid_dimensions() {
    let dataset = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("valid dataset");
    assert_eq!(dataset.canonical_string(), "110/homo_sapiens/GRCh38");
}

#[test]
fn dataset_id_serde_roundtrip_contract() {
    let dataset = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("valid dataset id");
    let serialized = serde_json::to_string(&dataset).expect("serialize");
    assert_eq!(
        serialized,
        "{\"release\":\"110\",\"species\":\"homo_sapiens\",\"assembly\":\"GRCh38\"}"
    );

    let restored: DatasetId = serde_json::from_str(&serialized).expect("deserialize");
    assert_eq!(restored, dataset);
}

#[test]
fn shard_id_parser_rejects_invalid_value() {
    let parsed = ShardId::parse("Shard-01");
    assert!(parsed.is_err(), "uppercase should fail invariant");
}
