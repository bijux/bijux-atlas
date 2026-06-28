// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_model::DatasetId;
use bijux_atlas_store::{
    dataset_key_prefix, dataset_manifest_key, dataset_manifest_lock_key, dataset_sqlite_key,
    StorePath,
};

#[test]
fn dataset_layout_keys_are_stable() {
    let dataset = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");

    assert_eq!(dataset_key_prefix(&dataset), "110/homo_sapiens/GRCh38");
    assert_eq!(
        dataset_manifest_key(&dataset),
        "110/homo_sapiens/GRCh38/manifest.json"
    );
    assert_eq!(
        dataset_sqlite_key(&dataset),
        "110/homo_sapiens/GRCh38/gene_summary.sqlite"
    );
    assert_eq!(
        dataset_manifest_lock_key(&dataset),
        "110/homo_sapiens/GRCh38/manifest.lock"
    );
}

#[test]
fn store_path_rejects_absolute_and_parent_segments() {
    assert!(StorePath::parse("/abs/path").is_err());
    assert!(StorePath::parse("a/../b").is_err());
    assert!(StorePath::parse("catalog.json").is_ok());
}
