// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, out);
            } else if path.extension().and_then(|x| x.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
}

#[test]
fn server_source_forbids_reqwest_blocking_usage() {
    let mut files = Vec::new();
    collect_rs_files(Path::new("src"), &mut files);
    for path in files {
        let text = fs::read_to_string(&path).expect("read server source file");
        assert!(
            !text.contains("reqwest::blocking"),
            "blocking reqwest usage is forbidden in server source: {}",
            path.display()
        );
    }
}
