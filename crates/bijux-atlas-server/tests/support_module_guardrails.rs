#[test]
fn support_modules_remain_non_entrypoint() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let support_files = ["src/http/genes_support.rs"];
    let forbidden_tokens = [
        "pub async fn",
        "route(",
        "Router::new",
        "serve(",
        "main_entry",
    ];

    for rel in support_files {
        let path = root.join(rel);
        let text = std::fs::read_to_string(&path).expect("read support file");
        for token in forbidden_tokens {
            assert!(
                !text.contains(token),
                "support module {} contains forbidden token {}",
                rel,
                token
            );
        }
    }
}
