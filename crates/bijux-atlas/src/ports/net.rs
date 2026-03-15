// SPDX-License-Identifier: Apache-2.0

use crate::errors::Result;

pub trait NetPort {
    fn get_bytes(&self, url: &str) -> Result<Vec<u8>>;
    fn post_json(&self, url: &str, body: &[u8]) -> Result<Vec<u8>>;
}
