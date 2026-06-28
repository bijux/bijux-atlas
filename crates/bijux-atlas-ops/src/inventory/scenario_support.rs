// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

pub const SCENARIO_SUPPORT_PATHS: [&str; 3] = [
    "ops/e2e/scenarios/version-compatibility.json",
    "ops/e2e/scenarios/required-tools.json",
    "ops/e2e/scenarios/result-schema.json",
];

pub fn validate_scenario_support_inputs(repo_root: &Path) -> Result<(), String> {
    for rel in SCENARIO_SUPPORT_PATHS {
        if !repo_root.join(rel).exists() {
            return Err(format!("missing prerequisite `{rel}` for scenario runner"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_scenario_support_inputs;

    #[test]
    fn scenario_support_validation_requires_all_owned_prerequisites() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/e2e/scenarios")).expect("mkdir scenarios");
        std::fs::write(
            root.path()
                .join("ops/e2e/scenarios/version-compatibility.json"),
            "{}",
        )
        .expect("write compatibility");
        std::fs::write(
            root.path().join("ops/e2e/scenarios/required-tools.json"),
            "{}",
        )
        .expect("write tools");

        let error = validate_scenario_support_inputs(root.path()).expect_err("missing schema");
        assert!(error.contains("ops/e2e/scenarios/result-schema.json"));
    }
}
