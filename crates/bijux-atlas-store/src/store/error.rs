// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_core::ErrorCode;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StoreErrorCode {
    NotFound,
    Validation,
    Conflict,
    Network,
    Io,
    CachedOnly,
    Unsupported,
    Internal,
}

impl StoreErrorCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::Validation => "validation_error",
            Self::Conflict => "conflict",
            Self::Network => "network_error",
            Self::Io => "io_error",
            Self::CachedOnly => "cached_only_mode",
            Self::Unsupported => "unsupported",
            Self::Internal => "internal_error",
        }
    }

    #[must_use]
    pub const fn as_error_code(self) -> ErrorCode {
        match self {
            Self::NotFound => ErrorCode::QueryRejectedByPolicy,
            Self::Validation => ErrorCode::InvalidQueryParameter,
            Self::Conflict => ErrorCode::QueryRejectedByPolicy,
            Self::Network => ErrorCode::NotReady,
            Self::Io => ErrorCode::Internal,
            Self::CachedOnly => ErrorCode::NotReady,
            Self::Unsupported => ErrorCode::QueryRejectedByPolicy,
            Self::Internal => ErrorCode::Internal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreError {
    pub code: StoreErrorCode,
    pub message: String,
}

impl StoreError {
    #[must_use]
    pub fn new(code: StoreErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl Display for StoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for StoreError {}
