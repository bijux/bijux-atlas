// SPDX-License-Identifier: Apache-2.0

#![deny(clippy::redundant_clone)]

pub(crate) mod admission;
mod handler;
pub(crate) mod response;

pub(crate) use self::handler::genes_handler;
