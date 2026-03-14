// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::errors::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DatasetId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ShardId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RunId(String);

impl DatasetId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_id("dataset_id", &value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ShardId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_id("shard_id", &value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl RunId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_id("run_id", &value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn validate_id(kind: &'static str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(Error::InvalidIdentifier {
            kind,
            value: value.to_owned(),
            reason: "must not be empty",
        });
    }

    if value.len() > 64 {
        return Err(Error::InvalidIdentifier {
            kind,
            value: value.to_owned(),
            reason: "must be at most 64 characters",
        });
    }

    if !value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    {
        return Err(Error::InvalidIdentifier {
            kind,
            value: value.to_owned(),
            reason: "must contain only [a-z0-9_-]",
        });
    }

    Ok(())
}

macro_rules! impl_id_traits {
    ($name:ident) => {
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = Error;

            fn try_from(value: String) -> Result<Self> {
                Self::new(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = Error;

            fn try_from(value: &str) -> Result<Self> {
                Self::new(value)
            }
        }

        impl FromStr for $name {
            type Err = Error;

            fn from_str(s: &str) -> Result<Self> {
                Self::new(s)
            }
        }
    };
}

impl_id_traits!(DatasetId);
impl_id_traits!(ShardId);
impl_id_traits!(RunId);
