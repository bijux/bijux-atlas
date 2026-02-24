use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn makefiles_are_free_of_legacy_atlasctl_token() {
    let root = repo_root().join("makefiles");
    let mut stack = vec![root.clone()];
    let mut violations = Vec::new();
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read_dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) != Some("mk") {
                continue;
            }
            let rel = path.strip_prefix(repo_root()).unwrap_or(&path);
            let text = fs::read_to_string(&path).expect("read makefile");
            if text.contains("atlasctl") {
                violations.push(rel.display().to_string());
            }
        }
    }
    assert!(
        violations.is_empty(),
        "legacy atlasctl token must not appear in makefiles: {violations:?}"
    );
}
