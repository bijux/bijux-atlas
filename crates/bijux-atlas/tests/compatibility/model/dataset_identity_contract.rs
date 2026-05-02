// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::dataset::{canonical_identity_hash, DatasetId, DatasetIdentity};

#[test]
fn dataset_identity_hash_is_canonical_and_repeatable() {
    let hash_a = canonical_identity_hash(
        "110/homo_sapiens/GRCh38",
        &"a".repeat(64),
        &"b".repeat(64),
        &"c".repeat(64),
    )
    .expect("hash a");
    let hash_b = canonical_identity_hash(
        "110/homo_sapiens/GRCh38",
        &"a".repeat(64),
        &"b".repeat(64),
        &"c".repeat(64),
    )
    .expect("hash b");
    assert_eq!(hash_a, hash_b);
}

#[test]
fn dataset_identity_validates_canonical_fingerprint_bundle() {
    let dataset = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset id");
    let identity = DatasetIdentity::from_components(
        &dataset,
        &serde_json::json!({"fasta": "x", "gff3": "y", "fai": "z"}),
        &serde_json::json!({"policy": "p", "toolchain": "t"}),
        &serde_json::json!({"sqlite": "s", "artifact": "a"}),
    )
    .expect("identity");
    identity.validate().expect("identity validate");
}
