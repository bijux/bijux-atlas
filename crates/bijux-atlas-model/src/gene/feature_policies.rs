// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct TranscriptTypePolicy {
    pub accepted_types: BTreeSet<String>,
}

impl Default for TranscriptTypePolicy {
    fn default() -> Self {
        Self {
            accepted_types: BTreeSet::from([
                "transcript".to_string(),
                "mRNA".to_string(),
                "mrna".to_string(),
            ]),
        }
    }
}

impl TranscriptTypePolicy {
    #[must_use]
    pub fn from_types(accepted_types: BTreeSet<String>) -> Self {
        Self { accepted_types }
    }

    #[must_use]
    pub fn accepts(&self, feature_type: &str) -> bool {
        self.accepted_types.contains(feature_type)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct TranscriptIdPolicy {
    pub attribute_keys: Vec<String>,
}

impl Default for TranscriptIdPolicy {
    fn default() -> Self {
        Self {
            attribute_keys: vec![
                "ID".to_string(),
                "transcript_id".to_string(),
                "transcriptId".to_string(),
            ],
        }
    }
}

impl TranscriptIdPolicy {
    #[must_use]
    pub fn from_keys(attribute_keys: Vec<String>) -> Self {
        Self { attribute_keys }
    }

    #[must_use]
    pub fn resolve(&self, attrs: &BTreeMap<String, String>) -> Option<String> {
        for key in &self.attribute_keys {
            if let Some(value) = attrs.get(key) {
                let v = value.trim();
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnknownFeaturePolicy {
    Reject,
    IgnoreWithWarning,
}

impl Default for UnknownFeaturePolicy {
    fn default() -> Self {
        Self::IgnoreWithWarning
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum FeatureIdUniquenessPolicy {
    Reject,
    NamespaceByFeatureType,
    NormalizeAsciiLowercaseReject,
}

impl Default for FeatureIdUniquenessPolicy {
    fn default() -> Self {
        Self::Reject
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum DuplicateGeneIdPolicy {
    Fail,
    DedupeKeepLexicographicallySmallest,
}

impl Default for DuplicateGeneIdPolicy {
    fn default() -> Self {
        Self::Fail
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum DuplicateTranscriptIdPolicy {
    Reject,
    DedupeKeepLexicographicallySmallest,
}

impl Default for DuplicateTranscriptIdPolicy {
    fn default() -> Self {
        Self::Reject
    }
}
