// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

// Compatibility re-export while `core` call sites finish migrating to `crate::ports::*`.
pub use crate::ports::*;
