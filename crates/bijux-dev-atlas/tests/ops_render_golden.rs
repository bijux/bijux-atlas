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
fn ops_render_kind_matches_golden_fixture() {
    let run_id = "ops_render_kind_golden";
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "render",
            "--target",
            "kind",
            "--run-id",
            run_id,
            "--allow-write",
            "--format",
            "json",
        ])
        .output()
        .expect("ops render kind");
    assert!(output.status.success());
    let rendered_path = repo_root()
        .join("artifacts/ops")
        .join(run_id)
        .join("render/developer/kind/render.yaml");
    let rendered = fs::read_to_string(rendered_path).expect("rendered yaml");
    let golden = fs::read_to_string(
        repo_root().join("crates/bijux-dev-atlas/tests/goldens/ops_render_kind_minimal.yaml"),
    )
    .expect("golden");
    assert_eq!(rendered, golden);
}
