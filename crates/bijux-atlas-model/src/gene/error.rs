// SPDX-License-Identifier: Apache-2.0

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseError {
    Empty(&'static str),
    Trimmed(&'static str),
    TooLong(&'static str, usize),
    InvalidFormat(&'static str),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty(name) => write!(f, "{name} must not be empty"),
            Self::Trimmed(name) => {
                write!(f, "{name} must not contain leading/trailing whitespace")
            }
            Self::TooLong(name, max) => write!(f, "{name} exceeds max length {max}"),
            Self::InvalidFormat(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for ParseError {}
