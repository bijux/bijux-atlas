// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
//! Atlas model compatibility surface.
//!
//! ```compile_fail
//! use bijux_atlas::model::DiffStatus;
//!
//! fn exhaustive_match(s: DiffStatus) -> &'static str {
//!     match s {
//!         DiffStatus::Added => "a",
//!         DiffStatus::Removed => "r",
//!         DiffStatus::Changed => "c",
//!     }
//! }
//! ```

pub mod serde_helpers {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::collections::BTreeMap;

    pub mod hex_bytes {
        use super::*;

        pub fn serialize<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut out = String::with_capacity(value.len() * 2);
            for b in value {
                use std::fmt::Write as _;
                let _ = write!(&mut out, "{b:02x}");
            }
            serializer.serialize_str(&out)
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let encoded = String::deserialize(deserializer)?;
            if encoded.len() % 2 != 0 {
                return Err(serde::de::Error::custom("hex string must have even length"));
            }

            let mut out = Vec::with_capacity(encoded.len() / 2);
            let bytes = encoded.as_bytes();
            for i in (0..bytes.len()).step_by(2) {
                let pair = std::str::from_utf8(&bytes[i..i + 2])
                    .map_err(|_| serde::de::Error::custom("hex string must be valid utf-8"))?;
                let byte = u8::from_str_radix(pair, 16).map_err(|_| {
                    serde::de::Error::custom("hex string contains non-hex digits")
                })?;
                out.push(byte);
            }
            Ok(out)
        }
    }

    pub mod timestamp_string {
        use super::*;

        pub fn serialize<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(value)
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
        where
            D: Deserializer<'de>,
        {
            let value = String::deserialize(deserializer)?;
            if value.trim().is_empty() {
                return Ok(String::new());
            }
            if !value.contains('T') || !value.ends_with('Z') {
                return Err(serde::de::Error::custom(
                    "timestamp must be RFC3339-like (e.g. 2026-02-24T00:00:00Z)",
                ));
            }
            Ok(value)
        }
    }

    #[must_use]
    pub fn map_is_empty<K, V>(value: &BTreeMap<K, V>) -> bool {
        value.is_empty()
    }
}

pub use crate::domain::dataset::{
    keys::{
        normalize_assembly, normalize_release, normalize_species, parse_assembly,
        parse_dataset_key, parse_release, parse_species, parse_species_normalized, Assembly,
        DatasetId, DatasetSelector, Release, Species, ValidationError, ASSEMBLY_MAX_LEN,
        RELEASE_MAX_LEN, SPECIES_MAX_LEN,
    },
    manifest::{
        artifact_paths, ArtifactChecksums, ArtifactManifest, ArtifactPaths, Catalog, CatalogEntry,
        IngestAnomalyReport, IngestRejection, ManifestInputHashes, ManifestStats,
        OptionalFieldPolicy, QcSeverity, ShardCatalog, ShardEntry, ShardId, ShardingPlan,
        LATEST_ALIAS_POLICY, NO_IMPLICIT_DEFAULT_DATASET_POLICY,
    },
    version::ModelVersion,
};
pub use crate::domain::policy::model::{GeneIdentifierPolicy, StrictnessMode};
pub use crate::domain::query::diff::{
    DiffPage, DiffRecord, DiffScope, DiffStatus, GeneSignatureInput, ReleaseGeneIndex,
    ReleaseGeneIndexEntry,
};
pub use crate::domain::query::gene::{
    BiotypePolicy, DuplicateGeneIdPolicy, DuplicateTranscriptIdPolicy, FeatureIdUniquenessPolicy,
    GeneId, GeneNamePolicy, GeneOrderKey, GeneSummary, ParseError, Region, SeqId,
    SeqidNormalizationPolicy, Strand, TranscriptId, TranscriptIdPolicy, TranscriptOrderKey,
    TranscriptTypePolicy, UnknownFeaturePolicy, ID_MAX_LEN, NAME_MAX_LEN, SEQID_MAX_LEN,
};

pub const CRATE_NAME: &str = "bijux-atlas";
