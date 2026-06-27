// SPDX-License-Identifier: Apache-2.0

mod contigs;
mod coordinates;
mod error;
mod feature_policies;
mod identifiers;
mod naming;

pub use contigs::{
    canonical_contig_label, classify_contig, ContigClass, SeqidNormalizationPolicy,
    SeqidNormalizationTrace,
};
pub use coordinates::{GeneOrderKey, GeneSummary, Region, Strand, TranscriptOrderKey};
pub use error::ParseError;
pub use feature_policies::{
    DuplicateGeneIdPolicy, DuplicateTranscriptIdPolicy, FeatureIdUniquenessPolicy,
    TranscriptIdPolicy, TranscriptTypePolicy, UnknownFeaturePolicy,
};
pub use identifiers::{GeneId, SeqId, TranscriptId, ID_MAX_LEN, NAME_MAX_LEN, SEQID_MAX_LEN};
pub use naming::{BiotypePolicy, GeneNamePolicy};
