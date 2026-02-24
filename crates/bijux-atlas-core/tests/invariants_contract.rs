use bijux_atlas_core::{DatasetId, RunId, ShardId};

#[test]
fn dataset_id_rejects_invalid_characters() {
    let err = DatasetId::new("Human Genome").expect_err("must reject spaces and uppercase");
    assert!(err.to_string().contains("dataset_id"));
}

#[test]
fn shard_id_rejects_empty() {
    let err = ShardId::new("").expect_err("must reject empty");
    assert!(err.to_string().contains("must not be empty"));
}

#[test]
fn run_id_accepts_valid_ascii_slug() {
    let run_id = RunId::new("run_2026-02-24").expect("valid run id");
    assert_eq!(run_id.as_str(), "run_2026-02-24");
}

#[test]
fn dataset_id_serde_roundtrip_contract() {
    let dataset = DatasetId::new("hsapiens_grch38").expect("valid dataset id");
    let serialized = serde_json::to_string(&dataset).expect("serialize");
    assert_eq!(serialized, "\"hsapiens_grch38\"");

    let restored: DatasetId = serde_json::from_str(&serialized).expect("deserialize");
    assert_eq!(restored, dataset);
}

#[test]
fn shard_id_serde_rejects_invalid_value() {
    let parsed: Result<ShardId, _> = serde_json::from_str("\"Shard-01\"");
    assert!(parsed.is_err(), "uppercase should fail invariant");
}
