// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![deny(clippy::todo)]

use clap as _;
use regex as _;

pub(crate) mod application;
pub mod core;
pub mod docs;
pub mod domains;
pub mod engine;
pub mod infrastructure;
pub mod model;
pub mod ops;
pub mod performance;
pub mod policies;
pub(crate) mod ports;
pub mod prelude;
pub mod reference;
pub mod registry;
pub(crate) mod schema_support;
pub mod ui;
pub mod version;

pub use self::infrastructure::adapters;
pub use self::infrastructure::runtime;
pub use crate::model::governance as governance_objects;
