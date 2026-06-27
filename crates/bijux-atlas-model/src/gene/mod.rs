// SPDX-License-Identifier: Apache-2.0

mod coordinates;
mod error;
mod identifiers;

pub use coordinates::{GeneOrderKey, GeneSummary, Region, Strand, TranscriptOrderKey};
pub use error::ParseError;
pub use identifiers::{GeneId, SeqId, TranscriptId, ID_MAX_LEN, NAME_MAX_LEN, SEQID_MAX_LEN};

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ContigClass {
    PrimaryAutosome,
    SexChromosome,
    Mitochondrial,
    Plasmid,
    Scaffold,
    Alternate,
    Unclassified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct SeqidNormalizationTrace {
    pub source_seqid: String,
    pub normalized_seqid: String,
    pub class: ContigClass,
    pub canonical_label: String,
    pub alias_applied: bool,
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

    #[must_use]
    pub fn normalize_with_trace(&self, raw: &str) -> SeqidNormalizationTrace {
        let source = raw.trim().to_string();
        let alias_mapped = self.aliases.get(source.as_str()).cloned();
        let normalized = alias_mapped.clone().unwrap_or_else(|| source.clone());
        SeqidNormalizationTrace {
            source_seqid: source,
            normalized_seqid: normalized.clone(),
            class: classify_contig(&normalized),
            canonical_label: canonical_contig_label(&normalized),
            alias_applied: alias_mapped.is_some(),
        }
    }
}

#[must_use]
pub fn canonical_contig_label(seqid: &str) -> String {
    let lowered = seqid.trim().to_ascii_lowercase();
    let without_chr = lowered.strip_prefix("chr").unwrap_or(lowered.as_str());
    if without_chr == "mt" || without_chr == "m" {
        "mitochondrial".to_string()
    } else if let Some(rest) = without_chr.strip_prefix("plasmid") {
        format!("plasmid{}", rest)
    } else {
        without_chr.to_string()
    }
}

#[must_use]
pub fn classify_contig(seqid: &str) -> ContigClass {
    let label = canonical_contig_label(seqid);
    if label.parse::<u64>().is_ok() {
        return ContigClass::PrimaryAutosome;
    }
    if matches!(label.as_str(), "x" | "y" | "w" | "z") {
        return ContigClass::SexChromosome;
    }
    if label == "mitochondrial" || label == "mitochondria" {
        return ContigClass::Mitochondrial;
    }
    if label.starts_with("plasmid") {
        return ContigClass::Plasmid;
    }
    if label.contains("scaffold") || label.starts_with("scf") {
        return ContigClass::Scaffold;
    }
    if label.contains('_')
        || label.starts_with("gl")
        || label.starts_with("ki")
        || label.contains("alt")
    {
        return ContigClass::Alternate;
    }
    ContigClass::Unclassified
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

#[cfg(test)]
mod tests {
    use super::{canonical_contig_label, classify_contig, ContigClass, SeqidNormalizationPolicy};
    use std::collections::BTreeMap;

    #[test]
    fn seqid_normalization_trace_is_reversible_and_classified() {
        let policy = SeqidNormalizationPolicy::from_aliases(BTreeMap::from([(
            "MT".to_string(),
            "chrM".to_string(),
        )]));
        let trace = policy.normalize_with_trace("MT");
        assert_eq!(trace.source_seqid, "MT");
        assert_eq!(trace.normalized_seqid, "chrM");
        assert_eq!(trace.class, ContigClass::Mitochondrial);
        assert_eq!(trace.canonical_label, "mitochondrial");
        assert!(trace.alias_applied);
    }

    #[test]
    fn contig_classification_covers_scientific_contig_families() {
        assert_eq!(classify_contig("chr1"), ContigClass::PrimaryAutosome);
        assert_eq!(classify_contig("chrX"), ContigClass::SexChromosome);
        assert_eq!(classify_contig("chrM"), ContigClass::Mitochondrial);
        assert_eq!(classify_contig("plasmidA"), ContigClass::Plasmid);
        assert_eq!(classify_contig("scaffold_42"), ContigClass::Scaffold);
        assert_eq!(
            classify_contig("chr1_GL383518v1_alt"),
            ContigClass::Alternate
        );
        assert_eq!(canonical_contig_label("chrMT"), "mitochondrial".to_string());
    }
}
