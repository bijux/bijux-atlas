// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_files_under(root: &Path) -> Vec<PathBuf> {
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

#[test]
fn runtime_source_does_not_include_criterion_or_bench_harness_logic() {
    let src = crate_root().join("src");
    for file in rust_files_under(&src) {
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
        for forbidden in ["criterion::", "criterion_group!", "criterion_main!"] {
            assert!(
                !text.contains(forbidden),
                "bench harness token `{forbidden}` must stay in benches/: {}",
                file.display()
            );
        }
    }
}

#[test]
fn benches_directory_exists_as_benchmark_owner() {
    let benches = crate_root().join("benches");
    assert!(
        benches.is_dir(),
        "runtime crate must keep benches/ as benchmark ownership root"
    );
    assert!(
        !benches.join("domain").exists(),
        "runtime benches must use durable ownership buckets, not benches/domain"
    );
    assert!(
        !benches.join("ingest").exists(),
        "runtime crate must not retain ingest-owned benchmark buckets"
    );
}

#[test]
fn cargo_manifest_bench_paths_match_runtime_bench_tree() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest_text = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", manifest_path.display()));
    let manifest: toml::Value = toml::from_str(&manifest_text)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", manifest_path.display()));
    let benches = manifest
        .get("bench")
        .and_then(toml::Value::as_array)
        .expect("bench array");

    for bench in benches {
        let path = bench
            .get("path")
            .and_then(toml::Value::as_str)
            .expect("bench path");
        assert!(
            !path.starts_with("benches/api/"),
            "runtime manifest must not claim api-owned benches: {path}"
        );
        assert!(
            !path.starts_with("benches/ingest/"),
            "runtime manifest must not claim ingest-owned benches: {path}"
        );
        assert!(
            crate_root().join(path).is_file(),
            "declared runtime bench path must exist: {path}"
        );
    }
}
