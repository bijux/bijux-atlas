#[test]
fn http_layer_does_not_import_runtime_effect_internals() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/http");
    let forbidden = [
        "runtime::dataset_cache_manager_storage",
        "crate::runtime::dataset_cache_manager_storage",
        "runtime::dataset_cache_manager_maintenance",
        "crate::runtime::dataset_cache_manager_maintenance",
        "std::fs::",
        "tokio::fs::",
        "reqwest::",
    ];

    for entry in std::fs::read_dir(&root).expect("read src/http") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if name == "effects_adapters.rs" {
            continue;
        }

        let text = std::fs::read_to_string(&path).expect("read http file");
        for token in forbidden {
            assert!(
                !text.contains(token),
                "http file {} contains forbidden token: {}",
                path.display(),
                token
            );
        }
    }
}
