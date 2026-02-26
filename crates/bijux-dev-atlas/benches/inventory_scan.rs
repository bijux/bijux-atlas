// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_inventory_scan(c: &mut Criterion) {
    c.bench_function("inventory_scan_walk", |b| {
        b.iter(|| {
            let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(|p| p.parent())
                .expect("repo")
                .to_path_buf();
            let ops = root.join("ops");
            let docs = root.join("docs");
            let configs = root.join("configs");
            let count = walk(&ops) + walk(&docs) + walk(&configs);
            black_box(count);
        });
    });
}

fn walk(root: &std::path::Path) -> usize {
    if !root.exists() {
        return 0;
    }
    let mut count = 0;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                count += 1;
            }
        }
    }
    count
}

criterion_group!(inventory_scan, bench_inventory_scan);
criterion_main!(inventory_scan);
