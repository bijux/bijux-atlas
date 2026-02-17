use std::path::PathBuf;

#[test]
fn public_enums_are_non_exhaustive() {
    let src_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let files = [
        "diff.rs",
        "gene.rs",
        "manifest.rs",
        "policy.rs",
        "dataset.rs",
    ];

    for file in files {
        let path = src_root.join(file);
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
