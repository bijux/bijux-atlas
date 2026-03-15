// SPDX-License-Identifier: Apache-2.0

#![deny(clippy::redundant_clone)]

pub(crate) mod admission;
pub(crate) mod response;
mod handler;

pub(crate) use self::handler::genes_handler;
