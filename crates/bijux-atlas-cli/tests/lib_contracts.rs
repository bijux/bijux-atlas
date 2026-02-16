use std::process::Command;

#[test]
fn command_surface_ssot_matches_doc() {
    let expected = [
        "atlas catalog validate",
        "atlas dataset validate",
        "atlas explain-query",
        "atlas ingest",
        "atlas inspect-db",
        "atlas openapi generate",
        "atlas serve",
        "atlas smoke",
        "completion",
    ]
    .join("\n");

    let path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/CLI_COMMAND_LIST.md");
    let current = std::fs::read_to_string(path).expect("read CLI command list");
    assert_eq!(current.trim(), expected.trim());
}

#[test]
fn help_template_includes_required_sections() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(output.status.success());
    let rendered = String::from_utf8(output.stdout).expect("utf8 help");
    for section in ["Usage:", "Options:", "Commands:", "Environment:"] {
        assert!(
            rendered.contains(section),
            "help output missing section `{section}`"
        );
    }
}

#[test]
fn top_level_subcommands_avoid_reserved_umbrella_verbs() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(output.status.success());
    let rendered = String::from_utf8(output.stdout).expect("utf8 help");
    for reserved in [" plugin", " plugins", " doctor", " config"] {
        assert!(
            !rendered.contains(reserved),
            "reserved verb exposed: {reserved}"
        );
    }
}

#[test]
fn completion_generation_contains_atlas_namespace() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["completion", "bash"])
        .output()
        .expect("run completion");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("utf8 completion");
    assert!(text.contains("atlas"));
}

#[test]
fn plugin_metadata_contains_required_fields() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--bijux-plugin-metadata")
        .output()
        .expect("run metadata");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid JSON metadata");
    for field in ["name", "version", "compatible_umbrella", "build_hash"] {
        assert!(payload.get(field).is_some(), "missing field `{field}`");
    }
}
