// SPDX-License-Identifier: Apache-2.0

use bijux_cli::api::install::{install_target_aliases, resolve_install_target};
use bijux_cli::contracts::known_bijux_tool;

#[test]
fn atlas_control_plane_binary_stays_registered_with_bijux_cli() {
    let tool = known_bijux_tool("atlas").expect("atlas namespace");
    assert_eq!(tool.runtime_binary(), "bijux-atlas");
    assert_eq!(tool.control_binary(), "bijux-dev-atlas");
    assert_eq!(tool.runtime_package(), "bijux-atlas");
    assert_eq!(tool.control_package(), "bijux-dev-atlas");
}

#[test]
fn dev_atlas_install_target_resolves_to_canonical_binary() {
    let aliases = install_target_aliases();
    assert!(aliases.contains(&"atlas".to_string()));
    assert!(aliases.contains(&"dev-atlas".to_string()));

    let target = resolve_install_target("dev-atlas").expect("dev-atlas install target");
    assert_eq!(target.target_name, "dev-atlas");
    assert_eq!(target.strategy.package_name, "bijux-dev-atlas");
    assert_eq!(target.strategy.executable_name, "bijux-dev-atlas");

    let binary_target =
        resolve_install_target("bijux-dev-atlas").expect("bijux-dev-atlas install target");
    assert_eq!(binary_target.target_name, "dev-atlas");
    assert_eq!(binary_target.strategy.package_name, "bijux-dev-atlas");
    assert_eq!(binary_target.strategy.executable_name, "bijux-dev-atlas");
}
