use std::fs;
use std::path::PathBuf;

#[test]
fn core_crate_has_no_runtime_io_or_async_deps() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = manifest_dir.join("Cargo.toml");
    let text = fs::read_to_string(cargo_toml).expect("read Cargo.toml");

    for forbidden in ["reqwest", "tokio", "std::fs", "hyper", "axum"] {
        assert!(
            !text.contains(forbidden),
            "forbidden dependency/token in core Cargo.toml: {forbidden}"
        );
    }
}
