// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetKeyDto {
    pub release: String,
    pub species: String,
    pub assembly: String,
}

impl DatasetKeyDto {
    pub fn new(release: String, species: String, assembly: String) -> Result<Self, &'static str> {
        if release.trim().is_empty() || species.trim().is_empty() || assembly.trim().is_empty() {
            return Err("dataset dimensions must be non-empty");
        }
        Ok(Self {
            release,
            species,
            assembly,
        })
    }

    #[must_use]
    pub fn route_key(&self) -> String {
        format!(
            "release={}/species={}/assembly={}",
            self.release, self.species, self.assembly
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageCursorDto {
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkCursorDto {
    pub next_cursor: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneRowsDto {
    pub rows: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListGenesResponseDto {
    pub api_version: String,
    pub contract_version: String,
    pub dataset: DatasetKeyDto,
    pub page: PageCursorDto,
    pub data: GeneRowsDto,
    pub links: Option<LinkCursorDto>,
}
