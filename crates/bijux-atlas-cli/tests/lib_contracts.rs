use std::process::Command;

#[test]
fn command_surface_ssot_matches_doc() {
    let expected = [
        "atlas catalog validate",
        "atlas completion",
        "atlas doctor",
        "atlas dataset validate",
        "atlas explain-query",
        "atlas ingest",
        "atlas inspect-db",
        "atlas openapi generate",
        "atlas print-config",
        "atlas serve",
        "atlas smoke",
        "atlas version",
        "completion",
        "version",
    ]
    .join("\n");

    let path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/CLI_COMMAND_LIST.md");
    let current = std::fs::read_to_string(path).expect("read CLI command list");
    assert_eq!(current.trim(), expected.trim());
}

#[test]
fn help_output_command_surface_matches_doc_exactly() {
    fn parse_commands_from_help(text: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut in_commands = false;
        for line in text.lines() {
            if line.trim() == "Commands:" {
                in_commands = true;
                continue;
            }
            if in_commands {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("Environment:") {
                    break;
                }
                let cmd = trimmed
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string();
                if !cmd.is_empty() && cmd != "help" {
                    out.push(cmd);
                }
            }
        }
        out
    }

    let top = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--help")
        .output()
        .expect("top help");
    assert!(top.status.success());
    let top_help = String::from_utf8(top.stdout).expect("utf8 top help");
    let top_cmds = parse_commands_from_help(&top_help);

    let atlas = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["atlas", "--help"])
        .output()
        .expect("atlas help");
    assert!(atlas.status.success());
    let atlas_help = String::from_utf8(atlas.stdout).expect("utf8 atlas help");
    let atlas_cmds = parse_commands_from_help(&atlas_help);

    let mut observed = Vec::new();
    for c in top_cmds {
        if c == "atlas" {
            for sub in &atlas_cmds {
                if matches!(sub.as_str(), "catalog" | "dataset" | "openapi") {
                    let nested = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
                        .args(["atlas", sub, "--help"])
                        .output()
                        .expect("nested atlas help");
                    assert!(nested.status.success());
                    let nested_help = String::from_utf8(nested.stdout).expect("utf8 nested help");
                    for subsub in parse_commands_from_help(&nested_help) {
                        observed.push(format!("atlas {sub} {subsub}"));
                    }
                } else {
                    observed.push(format!("atlas {sub}"));
                }
            }
        } else {
            observed.push(c);
        }
    }
    observed.sort();

    let path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/CLI_COMMAND_LIST.md");
    let mut expected = std::fs::read_to_string(path)
        .expect("read command list")
        .lines()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(observed, expected);
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
    for reserved in [" plugin", " plugins"] {
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
fn completion_generation_is_deterministic() {
    let one = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["completion", "bash"])
        .output()
        .expect("run completion #1");
    let two = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["completion", "bash"])
        .output()
        .expect("run completion #2");
    assert!(one.status.success());
    assert!(two.status.success());
    assert_eq!(one.stdout, two.stdout);
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
    for field in [
        "schema_version",
        "name",
        "version",
        "compatible_umbrella",
        "compatible_umbrella_min",
        "compatible_umbrella_max_exclusive",
        "build_hash",
    ] {
        assert!(payload.get(field).is_some(), "missing field `{field}`");
    }
}

#[test]
fn atlas_repo_builds_only_bijux_atlas_plugin_binary() {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = std::fs::read_to_string(manifest).expect("read Cargo.toml");
    assert!(text.contains("name = \"bijux-atlas\""));
    assert!(!text.contains("name = \"bijux\""));
}
