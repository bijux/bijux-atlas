// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ApiContentType {
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentNegotiation {
    pub pretty: bool,
}

impl ContentNegotiation {
    #[must_use]
    pub fn for_v1(pretty: bool, requested: Option<&str>) -> Option<Self> {
        match requested {
            None | Some("application/json") => Some(Self { pretty }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiResponseEnvelope<T> {
    pub data: T,
}
