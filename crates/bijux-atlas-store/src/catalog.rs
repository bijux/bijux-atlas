use bijux_atlas_core::canonical;
use bijux_atlas_model::{Catalog, CatalogEntry};

pub fn validate_catalog_strict(catalog: &Catalog) -> Result<(), String> {
    catalog.validate_sorted().map_err(|e| e.to_string())
}

pub fn canonical_catalog_json(catalog: &Catalog) -> Result<String, String> {
    validate_catalog_strict(catalog)?;
    let bytes = canonical::stable_json_bytes(catalog).map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

#[must_use]
pub fn sorted_catalog_entries(mut entries: Vec<CatalogEntry>) -> Vec<CatalogEntry> {
    entries.sort();
    entries
}
