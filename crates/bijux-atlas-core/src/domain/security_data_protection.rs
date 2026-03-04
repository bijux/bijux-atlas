// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::domain::canonical::sha256_hex;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataProtectionPolicy {
    pub strategy: String,
    pub encryption_required: bool,
    pub transport_tls_required: bool,
    pub retention_days: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    pub min_version: String,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub ca_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedCertificate {
    pub cert_pem: String,
    pub key_pem: String,
    pub ca_pem: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertificateValidationError {
    MissingFile(String),
    InvalidFormat(String),
    EmptyValue(String),
}

pub fn load_certificate_bundle(
    config: &TlsConfig,
) -> Result<LoadedCertificate, CertificateValidationError> {
    let cert_pem = read_pem_file(&config.cert_path, "certificate")?;
    let key_pem = read_pem_file(&config.key_path, "private_key")?;
    let ca_pem = if let Some(path) = &config.ca_path {
        Some(read_pem_file(path, "ca")?)
    } else {
        None
    };
    Ok(LoadedCertificate {
        cert_pem,
        key_pem,
        ca_pem,
    })
}

fn read_pem_file(path: &Path, kind: &str) -> Result<String, CertificateValidationError> {
    if !path.exists() {
        return Err(CertificateValidationError::MissingFile(format!(
            "{}:{}",
            kind,
            path.display()
        )));
    }
    let value = fs::read_to_string(path).map_err(|_| {
        CertificateValidationError::MissingFile(format!("{}:{}", kind, path.display()))
    })?;
    if value.trim().is_empty() {
        return Err(CertificateValidationError::EmptyValue(kind.to_string()));
    }
    Ok(value)
}

pub fn validate_certificate_bundle(
    bundle: &LoadedCertificate,
) -> Result<(), CertificateValidationError> {
    if !bundle.cert_pem.contains("BEGIN CERTIFICATE") {
        return Err(CertificateValidationError::InvalidFormat(
            "certificate".to_string(),
        ));
    }
    if !bundle.key_pem.contains("BEGIN") || !bundle.key_pem.contains("PRIVATE KEY") {
        return Err(CertificateValidationError::InvalidFormat(
            "private_key".to_string(),
        ));
    }
    if let Some(ca) = &bundle.ca_pem {
        if !ca.contains("BEGIN CERTIFICATE") {
            return Err(CertificateValidationError::InvalidFormat("ca".to_string()));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CertificateRotationState {
    pub active_cert_fingerprint: Option<String>,
    pub next_cert_fingerprint: Option<String>,
}

impl CertificateRotationState {
    pub fn stage_next(&mut self, next_fingerprint: &str) {
        self.next_cert_fingerprint = Some(next_fingerprint.to_string());
    }

    pub fn promote_next(&mut self) -> bool {
        let Some(next) = self.next_cert_fingerprint.take() else {
            return false;
        };
        self.active_cert_fingerprint = Some(next);
        true
    }
}

pub fn https_enforced(forwarded_proto: Option<&str>, required: bool) -> bool {
    if !required {
        return true;
    }
    forwarded_proto.is_some_and(|value| value.eq_ignore_ascii_case("https"))
}

#[must_use]
pub fn tls_handshake_allowed(
    tls_enabled: bool,
    min_version: &str,
    negotiated_version: &str,
) -> bool {
    if !tls_enabled {
        return false;
    }
    match (min_version, negotiated_version) {
        ("1.3", "1.3") => true,
        ("1.2", "1.2" | "1.3") => true,
        ("1.1", "1.1" | "1.2" | "1.3") => true,
        _ => false,
    }
}

pub trait EncryptionAtRest {
    fn encrypt(&self, plaintext: &[u8]) -> Vec<u8>;
    fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8>;
}

#[derive(Debug, Clone)]
pub struct XorEncryption {
    key: Vec<u8>,
}

impl XorEncryption {
    #[must_use]
    pub fn new(key: &[u8]) -> Self {
        Self { key: key.to_vec() }
    }
}

impl EncryptionAtRest for XorEncryption {
    fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        if self.key.is_empty() {
            return plaintext.to_vec();
        }
        plaintext
            .iter()
            .enumerate()
            .map(|(idx, value)| value ^ self.key[idx % self.key.len()])
            .collect()
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8> {
        self.encrypt(ciphertext)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetManifestIntegrity {
    pub dataset_id: String,
    pub artifact_checksums: BTreeMap<String, String>,
    pub manifest_checksum_sha256: String,
    pub dataset_signature_sha256: String,
}

#[must_use]
pub fn calculate_manifest_checksum(
    dataset_id: &str,
    artifact_checksums: &BTreeMap<String, String>,
) -> String {
    let payload = serde_json::json!({
        "dataset_id": dataset_id,
        "artifact_checksums": artifact_checksums
    });
    let encoded = serde_json::to_vec(&payload).unwrap_or_default();
    sha256_hex(&encoded)
}

#[must_use]
pub fn verify_dataset_manifest_integrity(manifest: &DatasetManifestIntegrity) -> bool {
    calculate_manifest_checksum(&manifest.dataset_id, &manifest.artifact_checksums)
        == manifest.manifest_checksum_sha256
}

#[must_use]
pub fn verify_artifact_checksum(bytes: &[u8], expected_sha256: &str) -> bool {
    sha256_hex(bytes) == expected_sha256
}

#[must_use]
pub fn verify_artifact_signature(
    artifact_sha256: &str,
    signing_key: &str,
    expected_signature: &str,
) -> bool {
    let payload = format!("{artifact_sha256}:{signing_key}");
    sha256_hex(payload.as_bytes()) == expected_signature
}

#[must_use]
pub fn detect_tampering(
    expected_checksum: &str,
    actual_checksum: &str,
    expected_signature: Option<&str>,
    actual_signature: Option<&str>,
) -> bool {
    if expected_checksum != actual_checksum {
        return true;
    }
    match (expected_signature, actual_signature) {
        (Some(left), Some(right)) => left != right,
        (None, None) => false,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        calculate_manifest_checksum, detect_tampering, https_enforced, load_certificate_bundle,
        tls_handshake_allowed, validate_certificate_bundle, verify_artifact_checksum,
        verify_artifact_signature, verify_dataset_manifest_integrity, CertificateRotationState,
        DatasetManifestIntegrity, EncryptionAtRest, TlsConfig, XorEncryption,
    };
    use std::collections::BTreeMap;

    #[test]
    fn tls_bundle_load_validate_and_rotation_work() {
        let temp = tempfile::tempdir().expect("temp");
        let cert = temp.path().join("tls.crt");
        let key = temp.path().join("tls.key");
        std::fs::write(
            &cert,
            "-----BEGIN CERTIFICATE-----\nabc\n-----END CERTIFICATE-----",
        )
        .expect("write cert");
        std::fs::write(
            &key,
            "-----BEGIN PRIVATE KEY-----\nabc\n-----END PRIVATE KEY-----",
        )
        .expect("write key");

        let cfg = TlsConfig {
            enabled: true,
            min_version: "1.2".to_string(),
            cert_path: cert,
            key_path: key,
            ca_path: None,
        };
        let bundle = load_certificate_bundle(&cfg).expect("load bundle");
        validate_certificate_bundle(&bundle).expect("validate bundle");

        let mut rotation = CertificateRotationState::default();
        rotation.stage_next("fp-next");
        assert!(rotation.promote_next());
        assert_eq!(rotation.active_cert_fingerprint.as_deref(), Some("fp-next"));
    }

    #[test]
    fn https_enforcement_requires_secure_forwarded_proto() {
        assert!(https_enforced(Some("https"), true));
        assert!(!https_enforced(Some("http"), true));
        assert!(!https_enforced(None, true));
        assert!(https_enforced(None, false));
    }

    #[test]
    fn tls_handshake_policy_enforces_minimum_version() {
        assert!(tls_handshake_allowed(true, "1.2", "1.2"));
        assert!(tls_handshake_allowed(true, "1.2", "1.3"));
        assert!(!tls_handshake_allowed(true, "1.3", "1.2"));
        assert!(!tls_handshake_allowed(false, "1.2", "1.3"));
    }

    #[test]
    fn encryption_integrity_signature_and_tamper_detection_work() {
        let cipher = XorEncryption::new(b"atlas-key");
        let plaintext = b"dataset bytes";
        let encrypted = cipher.encrypt(plaintext);
        assert_ne!(encrypted, plaintext);
        let decrypted = cipher.decrypt(&encrypted);
        assert_eq!(decrypted, plaintext);

        let checksum = super::sha256_hex(plaintext);
        assert!(verify_artifact_checksum(plaintext, &checksum));

        let signature = super::sha256_hex(format!("{checksum}:signing-key").as_bytes());
        assert!(verify_artifact_signature(
            &checksum,
            "signing-key",
            &signature
        ));
        assert!(detect_tampering(
            &checksum,
            "different",
            Some("a"),
            Some("a")
        ));
    }

    #[test]
    fn dataset_manifest_integrity_and_corruption_detection_work() {
        let mut checksums = BTreeMap::new();
        checksums.insert("sqlite".to_string(), "abc123".to_string());
        checksums.insert("fasta".to_string(), "fff111".to_string());
        let manifest_checksum = calculate_manifest_checksum("release/species/assembly", &checksums);
        let manifest = DatasetManifestIntegrity {
            dataset_id: "release/species/assembly".to_string(),
            artifact_checksums: checksums,
            manifest_checksum_sha256: manifest_checksum,
            dataset_signature_sha256: "sig".to_string(),
        };
        assert!(verify_dataset_manifest_integrity(&manifest));

        let mut corrupted = manifest.clone();
        corrupted.manifest_checksum_sha256 = "corrupt".to_string();
        assert!(!verify_dataset_manifest_integrity(&corrupted));
    }

    #[test]
    fn encryption_operations_have_bounded_runtime() {
        let cipher = XorEncryption::new(b"atlas-performance-key");
        let payload = vec![0x55_u8; 2 * 1024 * 1024];
        let started = std::time::Instant::now();
        let encrypted = cipher.encrypt(&payload);
        let decrypted = cipher.decrypt(&encrypted);
        assert_eq!(decrypted, payload);
        assert!(started.elapsed() < std::time::Duration::from_millis(250));
    }
}
