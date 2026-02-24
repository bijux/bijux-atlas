// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_query::{decode_cursor, encode_cursor, CursorErrorCode, CursorPayload, OrderMode};

#[test]
fn cursor_rejects_oversized_token() {
    let token = "a".repeat(2048);
    let err = decode_cursor(&token, b"s", "h", OrderMode::GeneId, None).expect_err("oversized");
    assert_eq!(err.code, CursorErrorCode::InvalidFormat);
}

#[test]
fn cursor_bounds_accept_normal_and_reject_tamper_same_length() {
    let payload = CursorPayload {
        cursor_version: "v1".to_string(),
        dataset_id: Some("110/homo_sapiens/GRCh38".to_string()),
        sort_key: Some("gene_id".to_string()),
        last_seen: None,
        order: "gene_id".to_string(),
        last_seqid: None,
        last_start: None,
        last_gene_id: "g1".to_string(),
        query_hash: "h".to_string(),
        depth: 0,
    };
    let token = encode_cursor(&payload, b"secret").expect("encode");
    assert!(token.starts_with("v1."));
    let ok = decode_cursor(
        &token,
        b"secret",
        "h",
        OrderMode::GeneId,
        Some("110/homo_sapiens/GRCh38"),
    )
    .expect("decode");
    assert_eq!(ok.last_gene_id, "g1");

    let mut tampered = token.clone().into_bytes();
    let idx = tampered.len() / 2;
    tampered[idx] = if tampered[idx] == b'a' { b'b' } else { b'a' };
    let tampered = String::from_utf8(tampered).expect("utf8");
    let err = decode_cursor(
        &tampered,
        b"secret",
        "h",
        OrderMode::GeneId,
        Some("110/homo_sapiens/GRCh38"),
    )
    .expect_err("tamper");
    assert!(matches!(
        err.code,
        CursorErrorCode::InvalidSignature | CursorErrorCode::InvalidFormat
    ));
    assert_eq!(
        tampered.len(),
        token.len(),
        "tampered token keeps same length"
    );
}

#[test]
fn cursor_decode_accepts_legacy_unversioned_format() {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;
    let payload_json = r#"{"order":"gene_id","last_seqid":null,"last_start":null,"last_gene_id":"g1","query_hash":"h"}"#;
    let payload_part = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());
    let mut mac = HmacSha256::new_from_slice(b"secret").expect("hmac key");
    mac.update(payload_part.as_bytes());
    let sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    let legacy_token = format!("{payload_part}.{sig}");

    let decoded = decode_cursor(&legacy_token, b"secret", "h", OrderMode::GeneId, None)
        .expect("legacy decode");
    assert_eq!(decoded.last_gene_id, "g1");
}

#[test]
fn cursor_rejects_excessive_depth() {
    let payload = CursorPayload {
        cursor_version: "v1".to_string(),
        dataset_id: Some("110/homo_sapiens/GRCh38".to_string()),
        sort_key: Some("gene_id".to_string()),
        last_seen: None,
        order: "gene_id".to_string(),
        last_seqid: None,
        last_start: None,
        last_gene_id: "g1".to_string(),
        query_hash: "h".to_string(),
        depth: 100_001,
    };
    let token = encode_cursor(&payload, b"secret").expect("encode");
    let err = decode_cursor(
        &token,
        b"secret",
        "h",
        OrderMode::GeneId,
        Some("110/homo_sapiens/GRCh38"),
    )
    .expect_err("reject");
    assert_eq!(err.code, CursorErrorCode::InvalidPayload);
}

#[test]
fn cursor_rejects_dataset_mismatch() {
    let payload = CursorPayload {
        cursor_version: "v1".to_string(),
        dataset_id: Some("110/homo_sapiens/GRCh38".to_string()),
        sort_key: Some("gene_id".to_string()),
        last_seen: None,
        order: "gene_id".to_string(),
        last_seqid: None,
        last_start: None,
        last_gene_id: "g1".to_string(),
        query_hash: "h".to_string(),
        depth: 0,
    };
    let token = encode_cursor(&payload, b"secret").expect("encode");
    let err = decode_cursor(
        &token,
        b"secret",
        "h",
        OrderMode::GeneId,
        Some("111/homo_sapiens/GRCh38"),
    )
    .expect_err("dataset mismatch");
    assert_eq!(err.code, CursorErrorCode::DatasetMismatch);
}

#[test]
fn cursor_rejects_unsupported_version_prefix() {
    let err = decode_cursor("v2.payload.sig", b"secret", "h", OrderMode::GeneId, None)
        .expect_err("unsupported");
    assert_eq!(err.code, CursorErrorCode::UnsupportedVersion);
}
