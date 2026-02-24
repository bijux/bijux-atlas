// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_model::{parse_dataset_key, DatasetId};

#[test]
fn dataset_key_roundtrip_is_canonical() {
    let ds = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let key = ds.key_string();
    let parsed = parse_dataset_key(&key).expect("parse key");
    assert_eq!(parsed, ds);
    assert_eq!(
        parsed.key_string(),
        "release=110&species=homo_sapiens&assembly=GRCh38"
    );
}

#[test]
fn dataset_key_rejects_missing_or_unknown_segments() {
    assert!(parse_dataset_key("release=110&species=homo_sapiens").is_err());
    assert!(parse_dataset_key("release=110&species=homo_sapiens&assembly=GRCh38&x=y").is_err());
    assert!(parse_dataset_key("release=latest&species=homo_sapiens&assembly=GRCh38").is_err());
    assert!(parse_dataset_key("release=110&species=Homo-sapiens&assembly=GRCh38").is_err());
}

#[test]
fn canonical_string_roundtrip_is_strict() {
    let ds = DatasetId::from_canonical_string("110/homo_sapiens/GRCh38").expect("canonical parse");
    assert_eq!(ds.canonical_string(), "110/homo_sapiens/GRCh38");
    assert!(DatasetId::from_canonical_string("110/homo_sapiens").is_err());
}
