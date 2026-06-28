// SPDX-License-Identifier: Apache-2.0

pub(crate) fn bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, String> {
    bijux_atlas_core::canonical::stable_json_bytes(value).map_err(|err| err.to_string())
}

pub(crate) fn text<T: serde::Serialize>(value: &T) -> Result<String, String> {
    String::from_utf8(bytes(value)?).map_err(|err| err.to_string())
}
