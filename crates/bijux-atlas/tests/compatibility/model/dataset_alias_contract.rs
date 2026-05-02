// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::dataset::{DatasetId, LatestAliasRecord};

#[test]
fn latest_alias_record_validates_canonical_trace_fields() {
    let dataset = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let record = LatestAliasRecord::new(
        dataset,
        "promotion-gated".to_string(),
        "1714592400".to_string(),
        "atlas-cli".to_string(),
        "a".repeat(64),
    );
    record.validate().expect("valid alias record");
}

#[test]
fn latest_alias_record_rejects_non_latest_alias_or_non_hex_catalog_hash() {
    let dataset = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let mut record = LatestAliasRecord::new(
        dataset,
        "promotion-gated".to_string(),
        "1714592400".to_string(),
        "atlas-cli".to_string(),
        "A".repeat(64),
    );
    assert!(record.validate().is_err(), "uppercase hex should be rejected");

    record.catalog_sha256 = "b".repeat(64);
    record.alias = "most_recent".to_string();
    assert!(record.validate().is_err(), "alias must stay exactly latest");
}
