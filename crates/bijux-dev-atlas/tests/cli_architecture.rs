use std::fs;
use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
    let mut modules = fs::read_dir(&src)
        .expect("read src")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("rs"))
        .collect::<Vec<_>>();
    modules.sort();
    let count = modules.len();
    let budget = 14usize;
    assert!(
        count <= budget,
        "cli crate module count exceeds budget: {count} > {budget} (src/*.rs)"
    );
}

#[test]
fn cli_source_files_stay_within_loc_budget() {
    let src = crate_root().join("src");
    let mut files = fs::read_dir(&src)
        .expect("read src")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("rs"))
        .collect::<Vec<_>>();
    files.sort();
    let warning_budget = 800usize;
    let error_budget = 1000usize;
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
    }
}
