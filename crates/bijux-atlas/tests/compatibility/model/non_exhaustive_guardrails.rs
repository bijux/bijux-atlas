// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

#[test]
fn public_enums_are_non_exhaustive() {
    let files = [
        "src/domain/query/diff.rs",
        "src/domain/query/gene.rs",
        "src/domain/dataset/manifest.rs",
        "src/domain/dataset/keys.rs",
        "src/domain/dataset/version.rs",
        "src/domain/policy/model.rs",
    ];

    for file in files {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(file);
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
