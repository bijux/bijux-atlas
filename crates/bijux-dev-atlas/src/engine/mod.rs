// SPDX-License-Identifier: Apache-2.0
//! Shared execution engine for registry-driven runnables.
//!
//! Boundary: engine may depend on `model` and `runtime`; command parsing stays outside.

use std::collections::BTreeSet;
use std::path::Path;

use crate::model::engine::*;

mod executor;
mod report_codec;
mod reporting;

pub use executor::*;
pub use report_codec::*;
pub use reporting::*;

include!("selection.rs");
include!("runner.rs");
include!("rendering.rs");
#[cfg(test)]
include!("tests.rs");
