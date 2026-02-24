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
    let budget = 700usize;
    assert!(
        loc <= budget,
        "cli crate main.rs exceeds LOC budget: {loc} > {budget} ({})",
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
    let budget = 10usize;
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
    let budget = 700usize;
    for path in files {
        let text = fs::read_to_string(&path).expect("read source file");
        let loc = text.lines().count();
        assert!(
            loc <= budget,
            "cli source file exceeds LOC budget: {loc} > {budget} ({})",
            path.display()
        );
    }
}
