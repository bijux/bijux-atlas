// SPDX-License-Identifier: Apache-2.0

pub const ALLOWED_INCLUDE: [&str; 4] = ["coords", "biotype", "counts", "length"];
pub const MAX_CURSOR_BYTES: usize = 4096;
pub const MAX_FILTER_COUNT: usize = 6;
pub const MAX_RANGE_SPAN: u64 = 5_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IncludeField {
    Coords,
    Biotype,
    Counts,
    Length,
}

impl IncludeField {
    pub(super) fn parse(raw: &str) -> Option<Self> {
        match raw {
            "coords" => Some(Self::Coords),
            "biotype" => Some(Self::Biotype),
            "counts" => Some(Self::Counts),
            "length" => Some(Self::Length),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    GeneIdAsc,
    RegionAsc,
}

impl SortKey {
    pub(super) fn parse(raw: &str) -> Option<Self> {
        match raw {
            "gene_id:asc" => Some(Self::GeneIdAsc),
            "region:asc" => Some(Self::RegionAsc),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntervalMode {
    Overlap,
    Containment,
    BoundaryTouch,
}

impl IntervalMode {
    pub(super) fn parse(raw: &str) -> Option<Self> {
        match raw {
            "overlap" => Some(Self::Overlap),
            "containment" => Some(Self::Containment),
            "boundary_touch" => Some(Self::BoundaryTouch),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrandMode {
    Any,
    Plus,
    Minus,
    Unknown,
}

impl StrandMode {
    pub(super) fn parse(raw: &str) -> Option<Self> {
        match raw {
            "any" => Some(Self::Any),
            "plus" => Some(Self::Plus),
            "minus" => Some(Self::Minus),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListGenesParams {
    pub dataset: Option<String>,
    pub release: String,
    pub species: String,
    pub assembly: String,
    pub limit: usize,
    pub cursor: Option<String>,
    pub gene_id: Option<String>,
    pub name: Option<String>,
    pub name_like: Option<String>,
    pub biotype: Option<String>,
    pub contig: Option<String>,
    pub range: Option<String>,
    pub min_transcripts: Option<u64>,
    pub max_transcripts: Option<u64>,
    pub sort: Option<SortKey>,
    pub include: Option<Vec<IncludeField>>,
    pub interval_mode: Option<IntervalMode>,
    pub strand: Option<StrandMode>,
    pub pretty: bool,
}
