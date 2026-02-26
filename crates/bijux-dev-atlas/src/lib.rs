// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]

pub(crate) mod adapters;
pub(crate) mod commands;
pub(crate) mod core;
pub(crate) mod model;
pub(crate) mod policies;
pub(crate) mod ports;
