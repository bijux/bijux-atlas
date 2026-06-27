// SPDX-License-Identifier: Apache-2.0

#[path = "main_handler.rs"]
mod main_handler;
#[path = "response_finalize.rs"]
mod response_finalize;

pub(crate) use self::main_handler::genes_handler;
