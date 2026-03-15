// SPDX-License-Identifier: Apache-2.0

use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    Transport,
    Timeout,
    RateLimited,
    Server,
    Client,
    Decode,
    InvalidConfig,
}

#[derive(Debug)]
pub struct ClientError {
    pub class: ErrorClass,
    pub message: String,
}

impl ClientError {
    pub fn new(class: ErrorClass, message: impl Into<String>) -> Self {
        Self {
            class,
            message: message.into(),
        }
    }
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.class, self.message)
    }
}

impl Error for ClientError {}
