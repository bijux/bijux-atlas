use bijux_atlas_query::{decode_cursor, encode_cursor, CursorErrorCode, CursorPayload, OrderMode};

#[test]
fn cursor_rejects_oversized_token() {
    let token = "a".repeat(2048);
    let err = decode_cursor(&token, b"s", "h", OrderMode::GeneId).expect_err("oversized");
    assert_eq!(err.code, CursorErrorCode::InvalidFormat);
}

#[test]
fn cursor_bounds_accept_normal_and_reject_tamper_same_length() {
    let payload = CursorPayload {
        order: "gene_id".to_string(),
        last_seqid: None,
        last_start: None,
        last_gene_id: "g1".to_string(),
        query_hash: "h".to_string(),
    };
    let token = encode_cursor(&payload, b"secret").expect("encode");
    assert!(token.starts_with("v1."));
    let ok = decode_cursor(&token, b"secret", "h", OrderMode::GeneId).expect("decode");
    assert_eq!(ok.last_gene_id, "g1");

    let mut tampered = token.clone().into_bytes();
    let idx = tampered.len() / 2;
    tampered[idx] = if tampered[idx] == b'a' { b'b' } else { b'a' };
    let tampered = String::from_utf8(tampered).expect("utf8");
    let err = decode_cursor(&tampered, b"secret", "h", OrderMode::GeneId).expect_err("tamper");
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

    let decoded =
        decode_cursor(&legacy_token, b"secret", "h", OrderMode::GeneId).expect("legacy decode");
    assert_eq!(decoded.last_gene_id, "g1");
}
