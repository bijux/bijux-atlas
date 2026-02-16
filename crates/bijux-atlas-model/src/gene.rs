use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct GeneId(String);

impl GeneId {
    pub fn parse(input: &str) -> Result<Self, String> {
        if input.is_empty() {
            return Err("gene_id must not be empty".to_string());
        }
        if input.trim() != input {
            return Err("gene_id must not contain leading/trailing whitespace".to_string());
        }
        Ok(Self(input.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct SeqId(String);

impl SeqId {
    pub fn parse(input: &str) -> Result<Self, String> {
        if input.is_empty() {
            return Err("seqid must not be empty".to_string());
        }
        if input.trim() != input {
            return Err("seqid must not contain leading/trailing whitespace".to_string());
        }
        Ok(Self(input.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct GeneSummary {
    pub gene_id: String,
    pub name: Option<String>,
    pub seqid: String,
    pub start: u64,
    pub end: u64,
    pub biotype: Option<String>,
    pub transcript_count: u64,
    pub sequence_length: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct SeqidNormalizationPolicy {
    pub aliases: BTreeMap<String, String>,
}

impl SeqidNormalizationPolicy {
    #[must_use]
    pub fn from_aliases(aliases: BTreeMap<String, String>) -> Self {
        Self { aliases }
    }

    #[must_use]
    pub fn normalize(&self, raw: &str) -> String {
        let trimmed = raw.trim();
        if let Some(mapped) = self.aliases.get(trimmed) {
            return mapped.clone();
        }
        trimmed.to_string()
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
