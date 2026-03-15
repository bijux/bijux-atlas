// SPDX-License-Identifier: Apache-2.0

#![deny(clippy::redundant_clone)]

#[path = "genes/admission.rs"]
mod genes_admission;
#[path = "genes/response.rs"]
mod genes_response;
mod handler;

pub(crate) use self::handler::genes_handler;
