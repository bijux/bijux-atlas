use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const ID_MAX_LEN: usize = 128;
pub const SEQID_MAX_LEN: usize = 128;
pub const NAME_MAX_LEN: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
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
        if input.len() > ID_MAX_LEN {
            return Err(format!("gene_id exceeds max length {ID_MAX_LEN}"));
        }
        Ok(Self(input.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
#[non_exhaustive]
pub struct TranscriptId(String);

impl TranscriptId {
    pub fn parse(input: &str) -> Result<Self, String> {
        if input.is_empty() {
            return Err("transcript_id must not be empty".to_string());
        }
        if input.trim() != input {
            return Err("transcript_id must not contain leading/trailing whitespace".to_string());
        }
        if input.len() > ID_MAX_LEN {
            return Err(format!("transcript_id exceeds max length {ID_MAX_LEN}"));
        }
        Ok(Self(input.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
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
        if input.len() > SEQID_MAX_LEN {
            return Err(format!("seqid exceeds max length {SEQID_MAX_LEN}"));
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Strand {
    Plus,
    Minus,
    Unknown,
}

impl Strand {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "+" => Ok(Self::Plus),
            "-" => Ok(Self::Minus),
            "." => Ok(Self::Unknown),
            _ => Err("strand must be one of '+', '-', '.'".to_string()),
        }
    }

    #[must_use]
    pub const fn as_symbol(self) -> &'static str {
        match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Unknown => ".",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct Region {
    pub seqid: SeqId,
    pub start: u64,
    pub end: u64,
}

impl Region {
    pub fn new(seqid: SeqId, start: u64, end: u64) -> Result<Self, String> {
        if start == 0 || end == 0 {
            return Err("region start/end must be >= 1".to_string());
        }
        if start > end {
            return Err("region start must be <= end".to_string());
        }
        Ok(Self { seqid, start, end })
    }

    pub fn parse(input: &str) -> Result<Self, String> {
        let (seqid_raw, rest) = input
            .split_once(':')
            .ok_or_else(|| "region must be in seqid:start-end format".to_string())?;
        let (start_raw, end_raw) = rest
            .split_once('-')
            .ok_or_else(|| "region must be in seqid:start-end format".to_string())?;
        let seqid = SeqId::parse(seqid_raw)?;
        let start = start_raw
            .parse::<u64>()
            .map_err(|_| "region start must be integer".to_string())?;
        let end = end_raw
            .parse::<u64>()
            .map_err(|_| "region end must be integer".to_string())?;
        Self::new(seqid, start, end)
    }

    #[must_use]
    pub fn canonical_string(&self) -> String {
        format!("{}:{}-{}", self.seqid.as_str(), self.start, self.end)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct GeneOrderKey {
    pub seqid: SeqId,
    pub start: u64,
    pub gene_id: GeneId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct TranscriptOrderKey {
    pub seqid: SeqId,
    pub start: u64,
    pub transcript_id: TranscriptId,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
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
