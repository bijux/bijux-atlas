// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_rs_files(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().and_then(|v| v.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out.sort();
    out
}

#[test]
fn cli_main_rs_loc_stays_within_budget() {
    let path = crate_root().join("src/main.rs");
    let text = fs::read_to_string(&path).expect("read main.rs");
    let loc = text.lines().count();
    let warning_budget = 800usize;
    let error_budget = 1000usize;
    if loc > warning_budget {
        eprintln!(
            "warning: cli crate main.rs exceeds warning LOC budget: {loc} > {warning_budget} ({})",
            path.display()
        );
    }
    assert!(
        loc <= error_budget,
        "cli crate main.rs exceeds hard LOC budget: {loc} > {error_budget} ({})",
        path.display()
    );
}

#[test]
fn cli_crate_module_count_stays_within_budget() {
    let src = crate_root().join("src");
    let modules = collect_rs_files(&src);
    let count = modules.len();
    // Unified dev-atlas now intentionally carries internalized core/model/policies/adapters modules.
    let budget = 120usize;
    assert!(
        count <= budget,
        "cli crate module count exceeds budget: {count} > {budget} (src/**/*.rs)"
    );
}

#[test]
fn cli_source_files_stay_within_loc_budget() {
    let src = crate_root().join("src");
    let files = collect_rs_files(&src);
    let warning_budget = 800usize;
    let error_budget = 1200usize;
    let strict_budget = 1200usize;
    let strict_paths = [
        "src/cli.rs",
        "src/commands/ops_support.rs",
        "src/commands/docs/runtime/command_dispatch.rs",
        "src/commands/ops/runtime.rs",
        "src/commands/ops/execution_runtime.rs",
    ];
    for path in files {
        let text = fs::read_to_string(&path).expect("read source file");
        let loc = text.lines().count();
        if loc > warning_budget {
            eprintln!(
                "warning: cli source file exceeds warning LOC budget: {loc} > {warning_budget} ({})",
                path.display()
            );
        }
        assert!(
            loc <= error_budget,
            "cli source file exceeds hard LOC budget: {loc} > {error_budget} ({})",
            path.display()
        );
        let rel = path
            .strip_prefix(crate_root())
            .unwrap_or(path.as_path())
            .display()
            .to_string();
        if strict_paths.contains(&rel.as_str()) {
            assert!(
                loc <= strict_budget,
                "strict LOC budget exceeded: {loc} > {strict_budget} ({})",
                path.display()
            );
        }
    }
}
