// SPDX-License-Identifier: Apache-2.0
//! Binary composition root for `bijux-dev-atlas`.
//!
//! The CLI crate owns the concrete wiring between parsed commands and application use-cases. This
//! module keeps that composition logic under a single owner instead of scattering bootstrap code at
//! the crate root.

mod runtime_entry;

pub(crate) use self::runtime_entry::*;
