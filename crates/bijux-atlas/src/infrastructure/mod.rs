// SPDX-License-Identifier: Apache-2.0

//! Compatibility surface for legacy outbound naming.
//! Canonical ownership lives under `adapters::outbound`.

pub mod redis;
pub mod sqlite;
pub mod store;
pub mod telemetry;
