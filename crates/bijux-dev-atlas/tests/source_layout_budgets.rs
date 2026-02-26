// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|v| v.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn commands_and_core_source_files_follow_loc_budgets() {
    let warning_budget = 800usize;
    let error_budget = 1000usize;
    let strict_error_budget = 1000usize;
    let strict_paths = [
        "src/commands/docs_runtime/command_dispatch.rs",
        "src/commands/docs_runtime/docs_command_router.rs",
        "src/core/checks/ops/governance_checks/inventory_contract_integrity_check/main_check.rs",
        "src/core/checks/ops/governance_checks/foundation_and_tooling_checks.rs",
    ];

    let mut files = Vec::new();
    collect_rs_files(&crate_root().join("src/commands"), &mut files);
    collect_rs_files(&crate_root().join("src/core"), &mut files);
    files.sort();

    for path in files {
        let text = fs::read_to_string(&path).expect("read source file");
        let loc = text.lines().count();
        if loc > warning_budget {
            eprintln!(
                "warning: commands/core source file exceeds warning LOC budget: {loc} > {warning_budget} ({})",
                path.display()
            );
        }
        assert!(
            loc <= error_budget,
            "commands/core source file exceeds hard LOC budget: {loc} > {error_budget} ({})",
            path.display()
        );

        let rel = path
            .strip_prefix(crate_root())
            .unwrap_or(path.as_path())
            .display()
            .to_string();
        if strict_paths.contains(&rel.as_str()) {
            assert!(
                loc <= strict_error_budget,
                "strict commands/core LOC budget exceeded: {loc} > {strict_error_budget} ({})",
                path.display()
            );
        }
    }
}
