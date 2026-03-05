// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn automation_gap_review_must_record_outdated_checks_and_contracts() {
    let root = repo_root();
    let path = root.join("crates/bijux-dev-atlas/docs/automation-boundary-gap-review.md");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    for needle in [
        "Outdated checks that assumed deprecated structure",
        "Outdated contracts that assumed legacy script locations",
        "checks automation-boundaries",
        "contract automation-boundaries",
    ] {
        assert!(
            text.contains(needle),
            "gap review file must contain `{needle}`: {}",
            path.display()
        );
    }
}

#[test]
fn governance_exceptions_must_not_reintroduce_script_automation_bypasses() {
    let root = repo_root();
    let exceptions = root.join("configs/governance/exceptions.yaml");
    let archive = root.join("configs/governance/exceptions-archive.yaml");
    let exceptions_text = fs::read_to_string(&exceptions)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", exceptions.display()));
    let archive_text = fs::read_to_string(&archive)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", archive.display()));
    for needle in [
        "tools/",
        "scripts/",
        "tutorials/scripts",
        "root script",
        "bash -c",
    ] {
        assert!(
            !exceptions_text.contains(needle) && !archive_text.contains(needle),
            "governance exception registries must not include automation bypass `{needle}`"
        );
    }
}
