// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

#[test]
fn hash256_public_api_avoids_raw_fixed_array_types() {
    let src = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/domain/canonical.rs"),
    )
    .expect("read domain/canonical.rs");

    // Public API should not expose raw fixed hash arrays; use Hash256 newtype.
    assert!(
        !src.contains("pub const fn as_bytes(&self) -> &[u8; 32]"),
        "Hash256 public API must not expose raw [u8; 32]"
    );
    assert!(
        !src.contains("pub const fn from_bytes(bytes: [u8; 32])"),
        "Hash256 constructor from raw [u8; 32] must not be public"
    );
}

#[test]
fn error_code_enum_is_defined_only_in_core_generated_module() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();

    let mut definitions = Vec::new();
    for entry in walkdir::WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("rs"))
    {
        let path = entry.path();
        let path_str = path.to_string_lossy();
        if !path_str.contains("/crates/bijux-atlas-") {
            continue;
        }
        if path.components().any(|c| c.as_os_str() == "tests") {
            continue;
        }
        let text = std::fs::read_to_string(path).expect("read rust file");
        if text.contains("enum ErrorCode") {
            definitions.push(path_str.to_string());
        }
    }

    let expected = root
        .join("crates/bijux-atlas-core/src/generated/error_codes.rs")
        .to_string_lossy()
        .to_string();
    assert_eq!(definitions, vec![expected]);
}
