// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_query::{decode_cursor, encode_cursor, CursorErrorCode, CursorPayload, OrderMode};

#[test]
fn signed_cursor_preserves_the_sha256_wire_vector() {
    let payload = CursorPayload {
        cursor_version: "v1".to_string(),
        dataset_id: Some("dataset".to_string()),
        sort_key: None,
        last_seen: None,
        order: "gene_id".to_string(),
        last_seqid: None,
        last_start: None,
        last_gene_id: "gene:1".to_string(),
        query_hash: "query-hash".to_string(),
        depth: 0,
    };
    // Freeze independently computed HMAC-SHA-256 bytes, including the legacy envelope.
    let expected = concat!(
        "v1.eyJjdXJzb3JfdmVyc2lvbiI6InYxIiwiZGF0YXNldF9pZCI6ImRhdGFzZXQi",
        "LCJzb3J0X2tleSI6bnVsbCwibGFzdF9zZWVuIjpudWxsLCJvcmRlciI6ImdlbmVf",
        "aWQiLCJsYXN0X3NlcWlkIjpudWxsLCJsYXN0X3N0YXJ0IjpudWxsLCJsYXN0X2dl",
        "bmVfaWQiOiJnZW5lOjEiLCJxdWVyeV9oYXNoIjoicXVlcnktaGFzaCIsImRlcHRoIjowfQ.",
        "VZ3WRdlidBq-0k1oytmqdzpYsqxkJeHnJkbMFp_dW1k"
    );
    let key = b"cursor-compatibility-key";
    assert_eq!(encode_cursor(&payload, key).expect("encode"), expected);
    for token in [
        expected,
        expected.strip_prefix("v1.").expect("version prefix"),
    ] {
        assert_eq!(
            decode_cursor(token, key, "query-hash", OrderMode::GeneId, Some("dataset"))
                .expect("verify fixed cursor"),
            payload
        );
        assert_eq!(
            decode_cursor(
                token,
                b"wrong-key",
                "query-hash",
                OrderMode::GeneId,
                Some("dataset")
            )
            .expect_err("wrong keys must be rejected")
            .code,
            CursorErrorCode::InvalidSignature
        );
    }
}
