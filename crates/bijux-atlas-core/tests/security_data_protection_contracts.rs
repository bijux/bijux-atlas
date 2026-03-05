// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_core::domain::CertificateRotationState;
use bijux_atlas_core::{
    calculate_manifest_checksum, detect_tampering, https_enforced, load_certificate_bundle,
    tls_handshake_allowed, validate_certificate_bundle, verify_artifact_checksum,
    verify_artifact_signature, verify_dataset_manifest_integrity, DatasetManifestIntegrity,
    TlsConfig,
};
use std::collections::BTreeMap;
use std::fs;

#[test]
fn tls_bundle_and_rotation_contract() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cert = temp.path().join("tls.crt");
    let key = temp.path().join("tls.key");
    let ca = temp.path().join("ca.crt");
    fs::write(
        &cert,
        "-----BEGIN CERTIFICATE-----\nABC\n-----END CERTIFICATE-----\n",
    )
    .expect("write cert");
    fs::write(
        &key,
        "-----BEGIN PRIVATE KEY-----\nABC\n-----END PRIVATE KEY-----\n",
    )
    .expect("write key");
    fs::write(
        &ca,
        "-----BEGIN CERTIFICATE-----\nABC\n-----END CERTIFICATE-----\n",
    )
    .expect("write ca");

    let bundle = load_certificate_bundle(&TlsConfig {
        enabled: true,
        min_version: "1.2".to_string(),
        cert_path: cert,
        key_path: key,
        ca_path: Some(ca),
    })
    .expect("load bundle");
    validate_certificate_bundle(&bundle).expect("validate bundle");

    let mut rotation = CertificateRotationState::default();
    rotation.stage_next("sha256:next");
    assert_eq!(
        rotation.next_cert_fingerprint.as_deref(),
        Some("sha256:next")
    );
    assert!(rotation.promote_next());
    assert_eq!(
        rotation.active_cert_fingerprint.as_deref(),
        Some("sha256:next")
    );
}

#[test]
fn encryption_integrity_and_tamper_contract() {
    let payload = b"atlas-payload";
    let checksum = {
        let mut hasher = sha2::Sha256::new();
        use sha2::Digest;
        hasher.update(payload);
        format!("{:x}", hasher.finalize())
    };

    assert!(verify_artifact_checksum(payload, &checksum));

    let signing_key = "atlas-signing-key";
    let signature_input = format!("{checksum}:{signing_key}");
    let signature = {
        let mut hasher = sha2::Sha256::new();
        use sha2::Digest;
        hasher.update(signature_input.as_bytes());
        format!("{:x}", hasher.finalize())
    };
    assert!(verify_artifact_signature(
        &checksum,
        signing_key,
        &signature
    ));

    assert!(detect_tampering(&checksum, "mismatch", None, None));
    assert!(!detect_tampering(&checksum, &checksum, None, None));
}

#[test]
fn dataset_manifest_integrity_contract() {
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
}

#[test]
fn tls_handshake_policy_contract() {
    assert!(tls_handshake_allowed(true, "1.2", "1.2"));
    assert!(tls_handshake_allowed(true, "1.2", "1.3"));
    assert!(!tls_handshake_allowed(true, "1.3", "1.2"));
    assert!(https_enforced(Some("https"), true));
    assert!(!https_enforced(Some("http"), true));
}
