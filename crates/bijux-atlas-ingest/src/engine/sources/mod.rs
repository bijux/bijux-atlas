// SPDX-License-Identifier: Apache-2.0

pub(crate) mod decode;
pub(crate) mod fai;
pub(crate) mod gff3;

pub(crate) use decode::{decode_ingest_inputs, DecodedIngest};
pub(crate) use fai::ContigStats;
