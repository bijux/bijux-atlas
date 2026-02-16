use bijux_atlas_core::{canonical, sha256_hex};
use serde_json::json;

#[test]
fn stable_json_bytes_are_key_order_deterministic() {
    let a = json!({"z": 2, "a": 1});
    let b = json!({"a": 1, "z": 2});
    let ba = canonical::stable_json_bytes(&a).expect("stable json a");
    let bb = canonical::stable_json_bytes(&b).expect("stable json b");
    assert_eq!(ba, bb);
}

#[test]
fn sha256_is_repeatable_for_same_bytes() {
    let bytes = b"atlas-core-determinism";
    let h1 = sha256_hex(bytes);
    let h2 = sha256_hex(bytes);
    assert_eq!(h1, h2);
}

#[test]
fn stable_json_hash_repeatable_across_invocations() {
    let value = json!({"k2": 2, "k1": 1, "nested": {"b": 2, "a": 1}});
    let h1 = canonical::stable_json_hash_hex(&value).expect("hash1");
    let h2 = canonical::stable_json_hash_hex(&value).expect("hash2");
    assert_eq!(h1, h2);
}
