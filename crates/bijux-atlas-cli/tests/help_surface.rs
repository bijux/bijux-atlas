// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

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
    for reserved in [" plugin", " plugins", " dev"] {
        assert!(
            !rendered.contains(reserved),
            "reserved verb exposed: {reserved}"
        );
    }
}
