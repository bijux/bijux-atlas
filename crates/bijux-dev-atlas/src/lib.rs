// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![deny(clippy::todo)]

use clap as _;
use regex as _;

pub mod adapters;
pub(crate) mod commands;
pub mod contracts;
pub mod core;
pub mod docs;
pub mod engine;
pub mod model;
pub mod ops;
pub mod policies;
pub mod prelude;
pub(crate) mod ports;
pub mod registry;
pub mod runtime;
pub(crate) mod schema_support;
pub mod ui;

pub use crate::model::governance as governance_objects;
