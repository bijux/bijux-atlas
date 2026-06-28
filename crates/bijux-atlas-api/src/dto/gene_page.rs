// SPDX-License-Identifier: Apache-2.0

use super::DatasetKeyDto;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
