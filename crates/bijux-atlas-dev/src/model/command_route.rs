// SPDX-License-Identifier: Apache-2.0
//! Stable command-route metadata used by CLI docs and route registries.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandRoute {
    pub id: &'static str,
    pub name: &'static str,
    pub domain: &'static str,
    pub purpose: &'static str,
}

impl CommandRoute {
    pub const fn new(
        id: &'static str,
        name: &'static str,
        domain: &'static str,
        purpose: &'static str,
    ) -> Self {
        Self {
            id,
            name,
            domain,
            purpose,
        }
    }
}
