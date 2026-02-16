use bijux_atlas_core::sha256_hex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestLock {
    pub manifest_sha256: String,
    pub sqlite_sha256: String,
}

impl ManifestLock {
    #[must_use]
    pub fn from_bytes(manifest_bytes: &[u8], sqlite_bytes: &[u8]) -> Self {
        Self {
            manifest_sha256: sha256_hex(manifest_bytes),
            sqlite_sha256: sha256_hex(sqlite_bytes),
        }
    }

    pub fn validate(&self, manifest_bytes: &[u8], sqlite_bytes: &[u8]) -> Result<(), String> {
        let manifest_actual = sha256_hex(manifest_bytes);
        let sqlite_actual = sha256_hex(sqlite_bytes);
        if manifest_actual != self.manifest_sha256 {
            return Err("manifest.lock mismatch for manifest_sha256".to_string());
        }
        if sqlite_actual != self.sqlite_sha256 {
            return Err("manifest.lock mismatch for sqlite_sha256".to_string());
        }
        Ok(())
    }
}

pub fn verify_expected_sha256(bytes: &[u8], expected: &str) -> Result<(), String> {
    let actual = sha256_hex(bytes);
    if actual != expected {
        return Err(format!(
            "sha256 mismatch expected={expected} actual={actual}"
        ));
    }
    Ok(())
}
