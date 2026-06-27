// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct RequestBuilder {
    path: String,
    query: BTreeMap<String, String>,
}

impl RequestBuilder {
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            query: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.insert(key.into(), value.into());
        self
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn query(&self) -> &BTreeMap<String, String> {
        &self.query
    }
}
