// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_model::{Catalog, CatalogEntry};
use std::collections::BTreeMap;

#[must_use]
pub fn sorted_catalog_entries(mut entries: Vec<CatalogEntry>) -> Vec<CatalogEntry> {
    entries.sort();
    entries
}

#[must_use]
pub fn merge_catalogs(catalogs: &[Catalog]) -> Catalog {
    let mut merged = BTreeMap::new();
    for catalog in catalogs {
        for entry in &catalog.datasets {
            merged
                .entry(entry.dataset.canonical_string())
                .or_insert_with(|| entry.clone());
        }
    }
    let mut datasets: Vec<CatalogEntry> = merged.into_values().collect();
    datasets.sort();
    Catalog::new(datasets)
}
