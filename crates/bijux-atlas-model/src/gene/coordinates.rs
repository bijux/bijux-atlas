// SPDX-License-Identifier: Apache-2.0

use super::error::ParseError;
use super::identifiers::{GeneId, SeqId, TranscriptId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct GeneSummary {
    pub gene_id: GeneId,
    pub name: Option<String>,
    pub seqid: SeqId,
    pub start: u64,
    pub end: u64,
    pub biotype: Option<String>,
    pub transcript_count: u64,
    pub sequence_length: u64,
}

impl GeneSummary {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        gene_id: GeneId,
        name: Option<String>,
        seqid: SeqId,
        start: u64,
        end: u64,
        biotype: Option<String>,
        transcript_count: u64,
        sequence_length: u64,
    ) -> Self {
        Self {
            gene_id,
            name,
            seqid,
            start,
            end,
            biotype,
            transcript_count,
            sequence_length,
        }
    }

    pub fn validate(&self) -> Result<(), ParseError> {
        if self.start == 0 || self.end == 0 {
            return Err(ParseError::InvalidFormat(
                "gene summary start/end must be >= 1",
            ));
        }
        if self.start > self.end {
            return Err(ParseError::InvalidFormat(
                "gene summary start must be <= end",
            ));
        }
        if self.sequence_length != (self.end - self.start + 1) {
            return Err(ParseError::InvalidFormat(
                "gene summary sequence_length must equal end-start+1",
            ));
        }
        Ok(())
    }
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
    pub fn parse(raw: &str) -> Result<Self, ParseError> {
        match raw {
            "+" => Ok(Self::Plus),
            "-" => Ok(Self::Minus),
            "." => Ok(Self::Unknown),
            _ => Err(ParseError::InvalidFormat(
                "strand must be one of '+', '-', '.'",
            )),
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
    pub fn new(seqid: SeqId, start: u64, end: u64) -> Result<Self, ParseError> {
        if start == 0 || end == 0 {
            return Err(ParseError::InvalidFormat("region start/end must be >= 1"));
        }
        if start > end {
            return Err(ParseError::InvalidFormat("region start must be <= end"));
        }
        Ok(Self { seqid, start, end })
    }

    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let (seqid_raw, rest) = input.split_once(':').ok_or(ParseError::InvalidFormat(
            "region must be in seqid:start-end format",
        ))?;
        let (start_raw, end_raw) = rest.split_once('-').ok_or(ParseError::InvalidFormat(
            "region must be in seqid:start-end format",
        ))?;
        let seqid = SeqId::parse(seqid_raw)?;
        let start = start_raw
            .parse::<u64>()
            .map_err(|_| ParseError::InvalidFormat("region start must be integer"))?;
        let end = end_raw
            .parse::<u64>()
            .map_err(|_| ParseError::InvalidFormat("region end must be integer"))?;
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
