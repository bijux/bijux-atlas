// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DatasetTier {
    Small,
    Medium,
    Large,
    XLarge,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatasetSpec {
    pub id: String,
    pub tier: DatasetTier,
    pub row_count: u64,
    pub shard_count: u32,
    pub bytes_uncompressed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatasetRegistry {
    pub schema_version: u32,
    pub datasets: Vec<DatasetSpec>,
}

impl DatasetRegistry {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("dataset registry must use schema_version=1".to_string());
        }
        if self.datasets.is_empty() {
            return Err("dataset registry must include at least one dataset".to_string());
        }
        for spec in &self.datasets {
            if spec.id.trim().is_empty() {
                return Err("dataset id must not be empty".to_string());
            }
            if spec.row_count == 0 {
                return Err(format!(
                    "dataset `{}` row_count must be greater than zero",
                    spec.id
                ));
            }
            if spec.shard_count == 0 {
                return Err(format!(
                    "dataset `{}` shard_count must be greater than zero",
                    spec.id
                ));
            }
        }
        Ok(())
    }
}

pub fn fixture_registry() -> DatasetRegistry {
    DatasetRegistry {
        schema_version: 1,
        datasets: vec![
            DatasetSpec {
                id: "genes-mini".to_string(),
                tier: DatasetTier::Small,
                row_count: 10_000,
                shard_count: 1,
                bytes_uncompressed: 8_000_000,
            },
            DatasetSpec {
                id: "genes-medium".to_string(),
                tier: DatasetTier::Medium,
                row_count: 1_000_000,
                shard_count: 8,
                bytes_uncompressed: 720_000_000,
            },
            DatasetSpec {
                id: "genes-large".to_string(),
                tier: DatasetTier::Large,
                row_count: 25_000_000,
                shard_count: 32,
                bytes_uncompressed: 18_000_000_000,
            },
        ],
    }
}
