// SPDX-License-Identifier: Apache-2.0

use crate::contracts::errors::Result;

pub trait AuthPort {
    fn authenticate_api_key(&self, api_key: &str) -> Result<()>;
    fn authenticate_bearer_token(&self, token: &str) -> Result<()>;
}
