// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
