// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct GeneNamePolicy {
    pub attribute_keys: Vec<String>,
}

impl Default for GeneNamePolicy {
    fn default() -> Self {
        Self {
            attribute_keys: vec![
                "gene_name".to_string(),
                "Name".to_string(),
                "gene".to_string(),
                "description".to_string(),
            ],
        }
    }
}

impl GeneNamePolicy {
    #[must_use]
    pub fn from_keys(attribute_keys: Vec<String>) -> Self {
        Self { attribute_keys }
    }

    #[must_use]
    pub fn resolve(&self, attrs: &BTreeMap<String, String>, fallback: &str) -> String {
        for key in &self.attribute_keys {
            if let Some(value) = attrs.get(key) {
                let v = value.split_whitespace().collect::<Vec<_>>().join(" ");
                if !v.is_empty() {
                    return v;
                }
            }
        }
        fallback.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct BiotypePolicy {
    pub attribute_keys: Vec<String>,
    pub unknown_value: String,
}

impl Default for BiotypePolicy {
    fn default() -> Self {
        Self {
            attribute_keys: vec![
                "gene_biotype".to_string(),
                "biotype".to_string(),
                "gene_type".to_string(),
            ],
            unknown_value: "unknown".to_string(),
        }
    }
}

impl BiotypePolicy {
    #[must_use]
    pub fn from_keys_and_unknown(attribute_keys: Vec<String>, unknown_value: String) -> Self {
        Self {
            attribute_keys,
            unknown_value,
        }
    }

    #[must_use]
    pub fn resolve(&self, attrs: &BTreeMap<String, String>) -> String {
        for key in &self.attribute_keys {
            if let Some(value) = attrs.get(key) {
                let v = value.split_whitespace().collect::<Vec<_>>().join(" ");
                if !v.is_empty() {
                    return v;
                }
            }
        }
        self.unknown_value.clone()
    }
}
