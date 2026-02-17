#[test]
fn planner_modules_do_not_pull_io_or_db_deps() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let pure_modules = [
        "src/planner/mod.rs",
        "src/filters.rs",
        "src/cost.rs",
        "src/limits.rs",
        "src/normalize.rs",
    ];
    let forbidden = [
        "rusqlite",
        "reqwest",
        "std::fs",
        "tokio::net",
        "std::process",
    ];

    for module in pure_modules {
        let path = root.join(module);
        let text = std::fs::read_to_string(&path).expect("read pure module");
        for needle in forbidden {
            assert!(
                !text.contains(needle),
                "forbidden import `{}` in {}",
                needle,
                module
            );
        }
    }
}
