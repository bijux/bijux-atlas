// SPDX-License-Identifier: Apache-2.0
//! Stable documentation references used by registries and list surfaces.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocRef {
    pub path: &'static str,
    pub anchor: Option<&'static str>,
    pub title: &'static str,
}

impl DocRef {
    pub const fn new(
        path: &'static str,
        anchor: Option<&'static str>,
        title: &'static str,
    ) -> Self {
        Self { path, anchor, title }
    }
}
