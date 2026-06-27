// SPDX-License-Identifier: Apache-2.0

mod render;
pub(crate) use self::render::*;

include!("k8s.rs");
mod load;
pub(crate) use self::load::*;

include!("lifecycle/mod.rs");

#[cfg(test)]
mod contracts;
