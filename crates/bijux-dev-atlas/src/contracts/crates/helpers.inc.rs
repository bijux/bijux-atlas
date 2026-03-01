fn violation(
    contract_id: &str,
    test_id: &str,
    file: Option<String>,
    message: impl Into<String>,
) -> Violation {
    Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file,
        line: None,
        message: message.into(),
        evidence: None,
    }
}

fn collect_crate_dirs(repo_root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let root = repo_root.join("crates");
    let Ok(entries) = fs::read_dir(root) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.push(path);
        }
    }
    out.sort();
    out
}

fn markdown_links(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut links = Vec::new();
    let mut idx = 0usize;
    while idx + 3 < bytes.len() {
        if bytes[idx] == b'[' {
            if let Some(close_bracket) = text[idx..].find("](") {
                let open_paren = idx + close_bracket + 1;
                if let Some(close_paren_rel) = text[open_paren + 1..].find(')') {
                    let target = &text[open_paren + 1..open_paren + 1 + close_paren_rel];
                    links.push(target.to_string());
                    idx = open_paren + 1 + close_paren_rel + 1;
                    continue;
                }
            }
        }
        idx += 1;
    }
    links
}

fn is_kebab_case_markdown_filename(name: &str) -> bool {
    if !name.ends_with(".md") {
        return false;
    }
    let stem = &name[..name.len() - 3];
    !stem.is_empty()
        && stem
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn crate_docs_markdown_files(crate_dir: &Path) -> Vec<PathBuf> {
    let docs_dir = crate_dir.join("docs");
    let Ok(entries) = fs::read_dir(docs_dir) else {
        return Vec::new();
    };
    let mut files = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn load_allowlist(repo_root: &Path, rel: &str) -> std::collections::BTreeSet<String> {
    let path = repo_root.join(rel);
    let Ok(text) = fs::read_to_string(path) else {
        return std::collections::BTreeSet::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(&text) else {
        return std::collections::BTreeSet::new();
    };
    let mut out = std::collections::BTreeSet::new();
    if let Some(items) = value.get("allowlist").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(entry) = item.as_str() {
                out.insert(entry.to_string());
            }
        }
    }
    out
}
