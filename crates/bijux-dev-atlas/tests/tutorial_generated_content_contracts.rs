// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn collect_markdown_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).expect("read dir");
    for entry in entries {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            collect_markdown_files(&path, files);
            continue;
        }
        if path.extension().and_then(|v| v.to_str()) == Some("md") {
            files.push(path);
        }
    }
}

#[test]
fn tutorials_should_not_embed_long_raw_output_blocks() {
    const MAX_INLINE_LINES: usize = 80;

    let root = repo_root();
    let tutorials = root.join("docs/tutorials");
    let mut files = Vec::new();
    collect_markdown_files(&tutorials, &mut files);

    let mut violations = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file).expect("read markdown");
        let mut in_block = false;
        let mut block_lines = 0usize;
        let mut block_lang = String::new();

        for line in text.lines() {
            if line.starts_with("```") {
                if in_block {
                    if (block_lang == "text" || block_lang == "json" || block_lang == "bash")
                        && block_lines > MAX_INLINE_LINES
                    {
                        violations.push(format!(
                            "{} has inline `{}` block with {} lines (max {})",
                            file.strip_prefix(&root).unwrap_or(&file).display(),
                            block_lang,
                            block_lines,
                            MAX_INLINE_LINES
                        ));
                    }
                    in_block = false;
                    block_lines = 0;
                    block_lang.clear();
                } else {
                    in_block = true;
                    block_lang = line.trim_start_matches("```").trim().to_string();
                    block_lines = 0;
                }
                continue;
            }
            if in_block {
                block_lines += 1;
            }
        }
    }

    assert!(
        violations.is_empty(),
        "tutorials must reference generated snippets instead of long pasted command output:\n{}",
        violations.join("\n")
    );
}

#[test]
fn generated_docs_directory_contains_required_example_snippets() {
    let root = repo_root();
    let required = [
        "docs/_generated/examples.md",
        "docs/_generated/command-lists.md",
        "docs/_generated/schema-snippets.md",
        "docs/_generated/openapi-snippets.md",
        "docs/_generated/ops-snippets.md",
    ];
    let missing = required
        .iter()
        .filter(|rel| !root.join(rel).exists())
        .map(|rel| (*rel).to_string())
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "required generated tutorial snippets are missing: {}",
        missing.join(", ")
    );
}
