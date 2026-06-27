// SPDX-License-Identifier: Apache-2.0
//! Shared execution engine for registry-driven runnables.
//!
//! Boundary: engine may depend on `model` and `runtime`; command parsing stays outside.

use std::collections::BTreeSet;
use std::path::Path;

use crate::model::engine::*;

mod executor;
mod rendering;
mod report_codec;
mod reporting;
mod runner;
mod selection;

pub use executor::*;
pub use rendering::*;
pub use report_codec::*;
pub use reporting::*;
pub use runner::*;
pub use selection::*;
#[cfg(test)]
include!("tests.rs");
