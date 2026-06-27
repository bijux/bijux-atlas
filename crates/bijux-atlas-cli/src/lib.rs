// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod adapters;

pub use bijux_atlas_runtime::{app, contracts, domain, runtime, version};

pub mod compat {
    pub mod core;
}
