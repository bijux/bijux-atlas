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
fn doctor_smoke() {
    let status = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .arg("doctor")
        .status()
        .expect("doctor");
    assert!(status.success());
}

#[test]
fn run_smoke() {
    let status = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["run", "--format", "text"])
        .status()
        .expect("run");
    assert!(status.success());
}

#[test]
fn help_snapshot_stable() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .arg("--help")
        .output()
        .expect("help");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("utf8");

    let golden_path = repo_root().join("crates/bijux-dev-atlas/tests/goldens/help.txt");
    let golden = fs::read_to_string(&golden_path).expect("golden");
    assert_eq!(text, golden);
}
