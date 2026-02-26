fn sha256_hex(path: &Path) -> Result<String, CheckError> {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    Ok(format!("{digest:x}"))
}

fn is_binary_like_file(path: &Path) -> Result<bool, CheckError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let known_binary_ext = [
        "gz", "zip", "zst", "tar", "sqlite", "db", "bin", "png", "jpg", "jpeg",
    ];
    if known_binary_ext.contains(&ext.as_str()) {
        return Ok(true);
    }
    let bytes = fs::read(path).map_err(|err| CheckError::Failed(err.to_string()))?;
    if bytes.contains(&0) {
        return Ok(true);
    }
    Ok(std::str::from_utf8(&bytes).is_err())
}

struct RequiredFilesContract {
    required_files: Vec<PathBuf>,
    required_dirs: Vec<PathBuf>,
    forbidden_patterns: Vec<String>,
    notes: Vec<String>,
}

fn parse_required_files_markdown_yaml(
    content: &str,
    rel: &Path,
) -> Result<RequiredFilesContract, CheckError> {
    let mut in_yaml = false;
    let mut yaml_block = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "```yaml" {
            in_yaml = true;
            continue;
        }
        if trimmed == "```" && in_yaml {
            break;
        }
        if in_yaml {
            yaml_block.push_str(line);
            yaml_block.push('\n');
        }
    }
    if yaml_block.trim().is_empty() {
        return Err(CheckError::Failed(format!(
            "{} must include a YAML contract block",
            rel.display()
        )));
    }
    let parsed: serde_yaml::Value =
        serde_yaml::from_str(&yaml_block).map_err(|err| CheckError::Failed(err.to_string()))?;
    let parsed_map = parsed.as_mapping().ok_or_else(|| {
        CheckError::Failed(format!(
            "{} YAML block must be a mapping with canonical keys",
            rel.display()
        ))
    })?;
    for key in [
        "required_files",
        "required_dirs",
        "forbidden_patterns",
        "notes",
    ] {
        if !parsed_map.contains_key(serde_yaml::Value::from(key)) {
            return Err(CheckError::Failed(format!(
                "{} must define `{key}` in REQUIRED_FILES contract YAML",
                rel.display()
            )));
        }
    }
    let required_files = parsed
        .get("required_files")
        .and_then(|v| v.as_sequence())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .map(PathBuf::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let required_dirs = parsed
        .get("required_dirs")
        .and_then(|v| v.as_sequence())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .map(PathBuf::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let forbidden_patterns = parsed
        .get("forbidden_patterns")
        .and_then(|v| v.as_sequence())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let notes = parsed
        .get("notes")
        .and_then(|v| v.as_sequence())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if required_files.is_empty() {
        return Err(CheckError::Failed(format!(
            "{} must define non-empty `required_files` YAML list",
            rel.display()
        )));
    }
    Ok(RequiredFilesContract {
        required_files,
        required_dirs,
        forbidden_patterns,
        notes,
    })
}

fn extract_ops_data_paths(text: &str) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();
    for token in text.split_whitespace() {
        let trimmed = token
            .trim_matches(|c: char| {
                c == '`'
                    || c == '('
                    || c == ')'
                    || c == '['
                    || c == ']'
                    || c == ','
                    || c == ';'
                    || c == ':'
                    || c == '"'
                    || c == '\''
            })
            .to_string();
        if !trimmed.starts_with("ops/") {
            continue;
        }
        if trimmed.ends_with(".json")
            || trimmed.ends_with(".yaml")
            || trimmed.ends_with(".yml")
            || trimmed.ends_with(".toml")
        {
            refs.insert(trimmed);
        }
    }
    refs
}

