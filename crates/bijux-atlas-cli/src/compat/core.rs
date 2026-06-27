// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_runtime::contracts::errors::Error;
use bijux_atlas_runtime::contracts::errors::Result;

pub fn stable_json_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
    bijux_atlas_core::canonical::stable_json_bytes(value)
        .map_err(|err| Error::json_encoding(err.to_string()))
}
