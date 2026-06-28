// SPDX-License-Identifier: Apache-2.0

mod merge;
mod serialization;

pub use merge::{merge_catalogs, sorted_catalog_entries};
pub use serialization::{canonical_catalog_json, validate_catalog_strict};
