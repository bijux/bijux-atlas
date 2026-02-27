// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("create dst");
    for entry in fs::read_dir(src).expect("read src") {
        let entry = entry.expect("entry");
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_tree(&path, &target);
        } else {
            fs::copy(&path, &target).expect("copy file");
        }
    }
}

fn fixture_repo(name: &str) -> tempfile::TempDir {
    let src = workspace_root().join("crates/bijux-dev-atlas/tests/fixtures/docker_contracts").join(name);
    let tmp = tempfile::tempdir().expect("tempdir");
    copy_tree(&src, tmp.path());
    #[cfg(unix)]
    std::os::unix::fs::symlink("docker/images/runtime/Dockerfile", tmp.path().join("Dockerfile"))
        .expect("symlink");
    bijux_dev_atlas::contracts::docker::sync_contract_markdown(tmp.path()).expect("sync contract doc");
    tmp
}

fn run_contracts(repo: &Path, contract_filter: Option<&str>) -> bijux_dev_atlas::contracts::RunReport {
    bijux_dev_atlas::contracts::run(
        "docker",
        bijux_dev_atlas::contracts::docker::contracts,
        repo,
        &bijux_dev_atlas::contracts::RunOptions {
            mode: bijux_dev_atlas::contracts::Mode::Static,
            allow_subprocess: false,
            allow_network: false,
            fail_fast: false,
            contract_filter: contract_filter.map(str::to_string),
            test_filter: None,
            list_only: false,
            artifacts_root: None,
        },
    )
    .expect("run contracts")
}

#[test]
fn pass_fixture_satisfies_all_static_contracts() {
    let tmp = fixture_repo("pass");
    let report = run_contracts(tmp.path(), None);
    assert_eq!(report.fail_count(), 0, "unexpected failures in pass fixture");
}

#[test]
fn fail_latest_fixture_hits_forbidden_tag_contract() {
    let tmp = fixture_repo("fail_latest");
    let report = run_contracts(tmp.path(), Some("DOCKER-006"));
    assert!(report.fail_count() > 0, "expected DOCKER-006 failures");
}

#[test]
fn fail_patterns_fixture_hits_forbidden_pattern_contract() {
    let tmp = fixture_repo("fail_patterns");
    let report = run_contracts(tmp.path(), Some("DOCKER-010"));
    assert!(report.fail_count() > 0, "expected DOCKER-010 failures");
}
