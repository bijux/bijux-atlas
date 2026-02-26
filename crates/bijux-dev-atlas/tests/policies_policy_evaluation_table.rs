// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::policies::{
    evaluate_policy_set_pure, DevAtlasPolicySet, DevAtlasPolicySetDocument, PolicyCategory,
    PolicyInputSnapshot, PolicySchemaVersion,
};

fn policy_doc() -> DevAtlasPolicySetDocument {
    serde_json::from_str(
        r#"{
  "schema_version":"1",
  "mode":"strict",
  "repo":{
    "max_loc_warn":10,
    "max_loc_hard":20,
    "max_depth_hard":7,
    "max_rs_files_per_dir_hard":10,
    "max_modules_per_dir_hard":16,
    "loc_allowlist":[],
    "rs_files_per_dir_allowlist":[]
  },
  "ops":{"registry_relpath":"ops/inventory/registry.toml"},
  "compatibility":[],
  "documented_defaults":[
    {"field":"repo.max_loc_hard","reason":"limit complexity"},
    {"field":"ops.registry_relpath","reason":"ssot"}
  ]
}"#,
    )
    .expect("policy")
}

#[test]
fn policy_table_good_and_bad_cases() {
    let set = DevAtlasPolicySet {
        schema_version: PolicySchemaVersion::V1,
        mode: policy_doc().mode,
        repo_policy: policy_doc().repo,
        ops_policy: policy_doc().ops,
        compatibility: Vec::new(),
        documented_defaults: policy_doc().documented_defaults,
        ratchets: Vec::new(),
        relaxations: Vec::new(),
    };

    let cases = vec![
        (
            "good",
            PolicyInputSnapshot {
                rust_file_line_counts: vec![("crates/x/src/lib.rs".to_string(), 12)],
                registry_relpath_exists: true,
            },
            0,
        ),
        (
            "repo_violation",
            PolicyInputSnapshot {
                rust_file_line_counts: vec![("crates/x/src/lib.rs".to_string(), 50)],
                registry_relpath_exists: true,
            },
            1,
        ),
        (
            "ops_violation",
            PolicyInputSnapshot {
                rust_file_line_counts: vec![("crates/x/src/lib.rs".to_string(), 10)],
                registry_relpath_exists: false,
            },
            1,
        ),
    ];

    for (_name, input, expected) in cases {
        let violations = evaluate_policy_set_pure(&set, &input);
        assert_eq!(violations.len(), expected);
    }
}

#[test]
fn violation_category_is_stable() {
    let set = DevAtlasPolicySet {
        schema_version: PolicySchemaVersion::V1,
        mode: policy_doc().mode,
        repo_policy: policy_doc().repo,
        ops_policy: policy_doc().ops,
        compatibility: Vec::new(),
        documented_defaults: policy_doc().documented_defaults,
        ratchets: Vec::new(),
        relaxations: Vec::new(),
    };
    let violations = evaluate_policy_set_pure(
        &set,
        &PolicyInputSnapshot {
            rust_file_line_counts: vec![("crates/x/src/lib.rs".to_string(), 99)],
            registry_relpath_exists: true,
        },
    );
    assert_eq!(violations[0].category, PolicyCategory::Repo);
}
