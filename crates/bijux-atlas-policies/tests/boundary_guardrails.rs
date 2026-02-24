// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

#[test]
fn policies_crate_must_not_import_ops_or_runtime_crates() {
    let src_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let forbidden = [
        "bijux_atlas_ops",
        "bijux_atlas_server",
        "bijux_atlas_cli",
        "tokio",
        "axum",
        "reqwest",
    ];

    let mut stack = vec![src_root];
    while let Some(path) = stack.pop() {
        for entry in std::fs::read_dir(path).expect("read_dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) != Some("rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("read file");
            for token in forbidden {
                assert!(
                    !text.contains(token),
                    "forbidden import token `{token}` in {}",
                    path.display()
                );
            }
        }
    }
}
