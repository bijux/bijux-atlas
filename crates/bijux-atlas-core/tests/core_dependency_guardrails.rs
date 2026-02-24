// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

#[test]
fn core_crate_has_no_runtime_io_or_async_deps() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = manifest_dir.join("Cargo.toml");
    let text = fs::read_to_string(cargo_toml).expect("read Cargo.toml");

    for forbidden in ["reqwest", "tokio", "std::fs", "hyper", "axum"] {
        assert!(
            !text.contains(forbidden),
            "forbidden dependency/token in core Cargo.toml: {forbidden}"
        );
    }
}

#[test]
fn core_crate_rand_dependency_is_forbidden() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml =
        std::fs::read_to_string(manifest_dir.join("Cargo.toml")).expect("read Cargo.toml");
    assert!(
        !cargo_toml.contains("rand"),
        "bijux-atlas-core must not depend on rand"
    );
}

#[test]
fn core_crate_serde_json_is_feature_gated() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml =
        std::fs::read_to_string(manifest_dir.join("Cargo.toml")).expect("read Cargo.toml");
    assert!(
        cargo_toml.contains("serde = [\"dep:serde_json\"]"),
        "serde_json must be enabled only through core feature `serde`"
    );
    assert!(
        cargo_toml.contains("serde_json = { version = \"1\", optional = true }"),
        "serde_json dependency must be optional"
    );
}

#[test]
fn purity_doc_exists_and_declares_ban_list() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let purity =
        std::fs::read_to_string(manifest_dir.join("docs/PURITY.md")).expect("read docs/PURITY.md");
    for token in [
        "No network I/O",
        "No filesystem I/O",
        "No process spawning",
        "No randomness",
        "serde_json",
    ] {
        assert!(
            purity.contains(token),
            "docs/PURITY.md missing token `{token}`"
        );
    }
}

#[test]
fn serde_json_usage_is_limited_to_canonical_module() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    for entry in std::fs::read_dir(&src_dir).expect("read src") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        if path.file_name().and_then(|s| s.to_str()) == Some("canonical.rs") {
            continue;
        }
        let content = std::fs::read_to_string(&path).expect("read source");
        assert!(
            !content.contains("serde_json"),
            "serde_json usage must be limited to canonical.rs: {}",
            path.display()
        );
    }
}
