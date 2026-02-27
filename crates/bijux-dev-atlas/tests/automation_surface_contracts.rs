use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    let mut cursor = Path::new(env!("CARGO_MANIFEST_DIR"));
    loop {
        if cursor.join(".git").exists() {
            return cursor.to_path_buf();
        }
        cursor = cursor.parent().expect("repo root not found");
    }
}

fn collect_files(root: &Path, relative_dir: &str, extension: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let dir = root.join(relative_dir);
    if !dir.exists() {
        return files;
    }
    let mut stack = vec![dir];
    while let Some(current) = stack.pop() {
        let entries = std::fs::read_dir(&current)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", current.display()));
        for entry in entries {
            let entry = entry.unwrap_or_else(|err| panic!("failed to read dir entry: {err}"));
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|v| v.to_str()) == Some(extension) {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

#[test]
fn workflow_and_make_surfaces_do_not_call_legacy_script_paths() {
    let root = repo_root();
    let mut files = collect_files(&root, ".github/workflows", "yml");
    files.extend(collect_files(&root, "makefiles", "mk"));

    let forbidden = [
        "bash scripts/",
        "python scripts/",
        "python3 scripts/",
        "ops/_lib/",
        "xtask",
    ];

    let mut violations = Vec::new();
    for file in files {
        let content = std::fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
        for needle in forbidden {
            if content.contains(needle) {
                violations.push(format!("{} contains `{needle}`", file.display()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "automation surfaces reference forbidden script paths:\n{}",
        violations.join("\n")
    );
}
