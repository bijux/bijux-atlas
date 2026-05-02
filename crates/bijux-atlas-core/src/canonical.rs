// SPDX-License-Identifier: Apache-2.0

use sha2::{Digest, Sha256};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash256([u8; 32]);

impl Hash256 {
    #[must_use]
    pub(crate) const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    #[must_use]
    pub fn to_hex(self) -> String {
        let mut out = String::with_capacity(64);
        for b in self.0 {
            use std::fmt::Write as _;
            let _ = write!(&mut out, "{b:02x}");
        }
        out
    }
}

impl core::fmt::Debug for Hash256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Hash256({})", self.to_hex())
    }
}

impl core::fmt::Display for Hash256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

#[must_use]
pub fn stable_sort_by_key<T, K: Ord, F: FnMut(&T) -> K>(mut values: Vec<T>, mut key: F) -> Vec<T> {
    values.sort_by_key(|v| key(v));
    values
}

#[must_use]
pub fn stable_hash_bytes(bytes: &[u8]) -> Hash256 {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = [0_u8; 32];
    out.copy_from_slice(&digest);
    Hash256::from_bytes(out)
}

#[must_use]
pub fn stable_hash_hex(bytes: &[u8]) -> String {
    stable_hash_bytes(bytes).to_hex()
}

#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    stable_hash_hex(bytes)
}

#[must_use]
pub fn sha256(bytes: &[u8]) -> Hash256 {
    stable_hash_bytes(bytes)
}
