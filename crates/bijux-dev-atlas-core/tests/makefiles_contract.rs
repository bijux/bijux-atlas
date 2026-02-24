// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn make_help_output_matches_golden() {
    let repo = repo_root();
    let output = Command::new("make")
        .args(["-s", "-f", "makefiles/root.mk", "help"])
        .current_dir(&repo)
        .output()
        .expect("run make help");
    assert!(output.status.success(), "make help must succeed");
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    let golden =
        fs::read_to_string(repo.join("crates/bijux-dev-atlas-core/tests/goldens/make_help.txt"))
            .expect("read golden");
    assert_eq!(
        stdout.trim_end(),
        golden.trim_end(),
        "make help output drift"
    );
}

#[test]
fn root_curated_targets_are_documented_once() {
    let repo = repo_root();
    let text = fs::read_to_string(repo.join("makefiles/root.mk")).expect("read root makefile");
    let curated_block = text
        .split("CURATED_TARGETS := \\")
        .nth(1)
        .and_then(|rest| rest.split("\n\nhelp:").next())
        .expect("curated targets block");
    let mut targets = Vec::new();
    for line in curated_block.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let trimmed = trimmed.trim_end_matches('\\').trim();
        if trimmed.is_empty() {
            continue;
        }
        for part in trimmed.split_whitespace() {
            targets.push(part.to_string());
        }
    }
    let mut sorted = targets.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        targets.len(),
        sorted.len(),
        "CURATED_TARGETS must not contain duplicates"
    );
    assert!(targets.contains(&"ci-pr".to_string()));
    assert!(
        !targets.contains(&"clean".to_string()),
        "clean is intentionally local alias not published yet"
    );
}
