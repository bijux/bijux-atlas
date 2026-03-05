// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_core::{
    detect_tampering, verify_artifact_checksum, verify_artifact_signature, EncryptionAtRest,
    XorEncryption,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn security_data_protection_benchmarks(c: &mut Criterion) {
    let payload = vec![42_u8; 64 * 1024];
    let checksum = bijux_atlas_core::sha256_hex(&payload);
    let signing_key = "atlas-signing-key";
    let signature = bijux_atlas_core::sha256_hex(format!("{checksum}:{signing_key}").as_bytes());
    let encryption = XorEncryption::new(b"benchmark-key");

    c.bench_function("security_encrypt_decrypt_roundtrip_64kb", |b| {
        b.iter(|| {
            let ciphertext = encryption.encrypt(black_box(&payload));
            let plaintext = encryption.decrypt(black_box(&ciphertext));
            black_box(plaintext.len());
        })
    });

    c.bench_function("security_artifact_checksum_verify_64kb", |b| {
        b.iter(|| {
            let ok = verify_artifact_checksum(black_box(&payload), black_box(&checksum));
            black_box(ok);
        })
    });

    c.bench_function("security_artifact_signature_verify", |b| {
        b.iter(|| {
            let ok = verify_artifact_signature(
                black_box(&checksum),
                black_box(signing_key),
                black_box(&signature),
            );
            black_box(ok);
        })
    });

    c.bench_function("security_tamper_detection_check", |b| {
        b.iter(|| {
            let tampered = detect_tampering(
                black_box(&checksum),
                black_box("mismatch"),
                black_box(Some(signature.as_str())),
                black_box(Some("bad-signature")),
            );
            black_box(tampered);
        })
    });
}

criterion_group!(
    security_data_protection,
    security_data_protection_benchmarks
);
criterion_main!(security_data_protection);
