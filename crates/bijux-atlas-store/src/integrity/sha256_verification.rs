// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_core::sha256_hex;

pub fn verify_expected_sha256(bytes: &[u8], expected: &str) -> Result<(), String> {
    let actual = sha256_hex(bytes);
    if actual != expected {
        return Err(format!(
            "sha256 mismatch expected={expected} actual={actual}"
        ));
    }
    Ok(())
}
