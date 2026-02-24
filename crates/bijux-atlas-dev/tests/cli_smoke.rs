use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn doctor_smoke() {
    Command::cargo_bin("bijux-atlas-dev")
        .expect("bin")
        .current_dir(repo_root())
        .arg("doctor")
        .assert()
        .success();
}

#[test]
fn run_smoke() {
    Command::cargo_bin("bijux-atlas-dev")
        .expect("bin")
        .current_dir(repo_root())
        .args(["run", "--format", "text"])
        .assert()
        .success();
}

#[test]
fn help_snapshot_stable() {
    let output = Command::cargo_bin("bijux-atlas-dev")
        .expect("bin")
        .current_dir(repo_root())
        .arg("--help")
        .output()
        .expect("help");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("utf8");

    let golden_path = repo_root().join("crates/bijux-atlas-dev/tests/goldens/help.txt");
    let golden = fs::read_to_string(&golden_path).expect("golden");
    assert_eq!(text, golden);
}
