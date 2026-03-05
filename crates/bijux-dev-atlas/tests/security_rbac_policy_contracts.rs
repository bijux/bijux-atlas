// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates root")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn rbac_assignments_reference_defined_roles() {
    let root = workspace_root();

    let roles_yaml =
        fs::read_to_string(root.join("configs/security/roles.yaml")).expect("read roles");
    let roles_value: serde_yaml::Value = serde_yaml::from_str(&roles_yaml).expect("parse roles");
    let role_ids: BTreeSet<String> = roles_value["roles"]
        .as_sequence()
        .expect("roles sequence")
        .iter()
        .filter_map(|entry| entry["id"].as_str().map(ToOwned::to_owned))
        .collect();

    let assignments_yaml = fs::read_to_string(root.join("configs/security/role-assignments.yaml"))
        .expect("read assignments");
    let assignments_value: serde_yaml::Value =
        serde_yaml::from_str(&assignments_yaml).expect("parse assignments");

    for assignment in assignments_value["assignments"]
        .as_sequence()
        .expect("assignments sequence")
    {
        let role_id = assignment["role_id"].as_str().expect("assignment role id");
        assert!(
            role_ids.contains(role_id),
            "assignment references undefined role: {role_id}"
        );
    }
}

#[test]
fn rbac_roles_reference_defined_permissions_and_inheritance() {
    let root = workspace_root();

    let permissions_yaml = fs::read_to_string(root.join("configs/security/permissions.yaml"))
        .expect("read permissions");
    let permissions_value: serde_yaml::Value =
        serde_yaml::from_str(&permissions_yaml).expect("parse permissions");
    let permission_ids: BTreeSet<String> = permissions_value["permissions"]
        .as_sequence()
        .expect("permissions sequence")
        .iter()
        .filter_map(|entry| entry["id"].as_str().map(ToOwned::to_owned))
        .collect();

    let roles_yaml =
        fs::read_to_string(root.join("configs/security/roles.yaml")).expect("read roles");
    let roles_value: serde_yaml::Value = serde_yaml::from_str(&roles_yaml).expect("parse roles");
    let role_ids: BTreeSet<String> = roles_value["roles"]
        .as_sequence()
        .expect("roles sequence")
        .iter()
        .filter_map(|entry| entry["id"].as_str().map(ToOwned::to_owned))
        .collect();

    for role in roles_value["roles"].as_sequence().expect("roles sequence") {
        let current_role_id = role["id"].as_str().expect("role id");

        for permission in role["permissions"].as_sequence().expect("permissions") {
            let permission_id = permission.as_str().expect("permission id");
            assert!(
                permission_ids.contains(permission_id),
                "role {current_role_id} references undefined permission {permission_id}"
            );
        }

        for inherited in role["inherits"].as_sequence().expect("inherits") {
            let inherited_role = inherited.as_str().expect("inherited role");
            assert!(
                role_ids.contains(inherited_role),
                "role {current_role_id} inherits undefined role {inherited_role}"
            );
        }
    }
}
