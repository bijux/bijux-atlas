use std::process::Command;

#[test]
fn top_level_help_exposes_only_control_plane_families() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("utf8 help");

    for required in ["ops", "docs", "configs", "policies", "check"] {
        assert!(
            text.contains(required),
            "missing command family: {required}"
        );
    }
    for forbidden in ["docker", "build", "workflows", "gates", "capabilities"] {
        assert!(
            !text.contains(forbidden),
            "forbidden top-level namespace leaked: {forbidden}"
        );
    }
}

#[test]
fn json_flag_forces_machine_output_mode() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(["--json", "check", "list"])
        .output()
        .expect("run check list json");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert!(payload.get("checks").is_some());
}

#[test]
fn no_network_default_is_enforced_for_docs_serve() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(["docs", "serve", "--allow-subprocess", "--json"])
        .output()
        .expect("docs serve without network");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr");
    assert!(stderr.contains("docs serve requires --allow-network"));
}
