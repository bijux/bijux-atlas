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

#[test]
fn runtime_layer_does_not_import_http_protocol_modules() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/runtime");
    let forbidden = ["crate::http::", "super::http::", "hyper::"];

    for path in rust_files_under(&root) {
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read runtime file");
        for token in forbidden {
            assert!(
                !text.contains(token),
                "runtime file {} contains forbidden token: {}",
                path.display(),
                token
            );
        }
    }
}

#[test]
fn effects_layer_avoids_http_server_framework_deps() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/runtime/effects");
    let forbidden = ["crate::http::", "axum::", "hyper::"];

    for path in rust_files_under(&root) {
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read effects file");
        for token in forbidden {
            assert!(
                !text.contains(token),
                "effects file {} contains forbidden token: {}",
                path.display(),
                token
            );
        }
    }
}

fn rust_files_under(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
    out
}
