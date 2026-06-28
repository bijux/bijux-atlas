// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

fn model_crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn public_enums_are_non_exhaustive() {
    let files = [
        "src/diff/mod.rs",
        "src/gene/mod.rs",
        "src/dataset/manifest.rs",
        "src/dataset/keys.rs",
        "src/dataset/version.rs",
        "src/policy.rs",
    ];

    for file in files {
        let path = model_crate_dir().join(file);
        let text = std::fs::read_to_string(&path).expect("read source");
        for line in text.lines() {
            if !line.contains("pub enum ") {
                continue;
            }
            let needle = line.trim();
            let idx = text.find(needle).expect("enum line in source text");
            let start = idx.saturating_sub(220);
            let window = &text[start..idx];
            assert!(
                window.contains("#[non_exhaustive]"),
                "public enum without #[non_exhaustive] in {}: {}",
                path.display(),
                needle
            );
        }
    }
}
