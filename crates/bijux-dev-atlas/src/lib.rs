// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![deny(clippy::todo)]

use clap as _;
use regex as _;

pub mod adapters;
pub(crate) mod commands;
pub mod core;
pub mod model;
pub mod policies;
pub(crate) mod ports;
