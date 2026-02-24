use std::collections::BTreeMap;
use std::fmt;

pub use crate::generated::error_codes::{ErrorCode, ERROR_CODES};

pub type Result<T> = std::result::Result<T, Error>;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExitCode {
    Success = 0,
    Usage = 2,
    Validation = 3,
    DependencyFailure = 4,
    Internal = 10,
}

impl ExitCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Usage => "usage",
            Self::Validation => "validation",
            Self::DependencyFailure => "dependency_failure",
            Self::Internal => "internal",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConfigPathScope {
    User,
    Workspace,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    #[cfg(feature = "serde")]
    SerdeJson(serde_json::Error),
    #[cfg(feature = "serde")]
    DecodeCursorBase64(String),
    #[cfg(feature = "serde")]
    DecodeCursorJson(String),
    InvalidIdentifier {
        kind: &'static str,
        value: String,
        reason: &'static str,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "serde")]
            Self::SerdeJson(err) => write!(f, "serde json error: {err}"),
            #[cfg(feature = "serde")]
            Self::DecodeCursorBase64(message) => write!(f, "cursor base64 decode failed: {message}"),
            #[cfg(feature = "serde")]
            Self::DecodeCursorJson(message) => write!(f, "cursor json decode failed: {message}"),
            Self::InvalidIdentifier {
                kind,
                value,
                reason,
            } => write!(f, "invalid {kind} `{value}`: {reason}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(feature = "serde")]
            Self::SerdeJson(err) => Some(err),
            #[cfg(feature = "serde")]
            Self::DecodeCursorBase64(_) | Self::DecodeCursorJson(_) | Self::InvalidIdentifier { .. } => None,
            #[cfg(not(feature = "serde"))]
            Self::InvalidIdentifier { .. } => None,
        }
    }
}

#[cfg(feature = "serde")]
impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJson(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MachineError {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub details: BTreeMap<String, String>,
}

impl MachineError {
    #[must_use]
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_detail(mut self, key: &str, value: &str) -> Self {
        self.details.insert(key.to_string(), value.to_string());
        self
    }
}

impl fmt::Display for MachineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for MachineError {}
