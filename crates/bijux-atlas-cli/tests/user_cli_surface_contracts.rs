// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

#[test]
fn user_cli_does_not_expose_dev_runtime_diagnostics() {
    let out = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(!text.contains("self-check"));
    assert!(!text.contains("print-config-schema"));
    assert!(!text.contains("runtime"));
}
