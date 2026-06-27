// SPDX-License-Identifier: Apache-2.0

#![deny(clippy::redundant_clone)]

pub(crate) mod admission;
mod main_handler;
pub(crate) mod response;
mod response_finalize;
mod routes;
pub(crate) mod support;

pub(crate) use self::main_handler::genes_handler;
pub(crate) use self::routes::genes_count_handler;
