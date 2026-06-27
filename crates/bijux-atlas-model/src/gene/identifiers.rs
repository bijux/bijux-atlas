// SPDX-License-Identifier: Apache-2.0

use super::error::ParseError;
use serde::{Deserialize, Serialize};

pub const ID_MAX_LEN: usize = 128;
pub const SEQID_MAX_LEN: usize = 128;
pub const NAME_MAX_LEN: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
#[non_exhaustive]
pub struct GeneId(String);

impl GeneId {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        if input.is_empty() {
            return Err(ParseError::Empty("gene_id"));
        }
        if input.trim() != input {
            return Err(ParseError::Trimmed("gene_id"));
        }
        if input.len() > ID_MAX_LEN {
            return Err(ParseError::TooLong("gene_id", ID_MAX_LEN));
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
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        if input.is_empty() {
            return Err(ParseError::Empty("transcript_id"));
        }
        if input.trim() != input {
            return Err(ParseError::Trimmed("transcript_id"));
        }
        if input.len() > ID_MAX_LEN {
            return Err(ParseError::TooLong("transcript_id", ID_MAX_LEN));
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
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        if input.is_empty() {
            return Err(ParseError::Empty("seqid"));
        }
        if input.trim() != input {
            return Err(ParseError::Trimmed("seqid"));
        }
        if input.len() > SEQID_MAX_LEN {
            return Err(ParseError::TooLong("seqid", SEQID_MAX_LEN));
        }
        Ok(Self(input.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
