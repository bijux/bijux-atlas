use std::collections::BTreeMap;

pub(super) fn parse_mkdocs_yaml(ctx: &CheckContext<'_>) -> Result<YamlValue, CheckError> {
    let rel = Path::new("mkdocs.yml");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(format!("failed to read {}: {err}", rel.display())))?;
    serde_yaml::from_str(&text)
        .map_err(|err| CheckError::Failed(format!("failed to parse {}: {err}", rel.display())))
}

fn collect_mkdocs_nav_refs(node: &YamlValue, out: &mut Vec<(String, String)>) {
    match node {
        YamlValue::Sequence(seq) => {
            for item in seq {
                collect_mkdocs_nav_refs(item, out);
            }
        }
        YamlValue::Mapping(map) => {
            for (k, v) in map {
                let title = k.as_str().unwrap_or_default().to_string();
                if let Some(path) = v.as_str() {
                    out.push((title, path.to_string()));
                } else {
                    collect_mkdocs_nav_refs(v, out);
                }
            }
        }
        _ => {}
    }
}

pub(super) fn mkdocs_nav_refs(ctx: &CheckContext<'_>) -> Result<Vec<(String, String)>, CheckError> {
    let yaml = parse_mkdocs_yaml(ctx)?;
    let nav = yaml
        .get("nav")
        .ok_or_else(|| CheckError::Failed("mkdocs.yml missing `nav`".to_string()))?;
    let mut refs = Vec::new();
    collect_mkdocs_nav_refs(nav, &mut refs);
    refs.sort();
    Ok(refs)
}

fn docs_markdown_paths(ctx: &CheckContext<'_>) -> Vec<PathBuf> {
    let docs = ctx.repo_root.join("docs");
    if !docs.exists() {
        return Vec::new();
    }
    walk_files(&docs)
        .into_iter()
        .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("md"))
        .collect()
}

fn markdown_h1_title(text: &str) -> Option<String> {
    text.lines()
        .find_map(|line| line.strip_prefix("# ").map(|v| v.trim().to_string()))
}

fn markdown_link_targets(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let mut cursor = line;
        while let Some(start) = cursor.find('(') {
            let after_start = &cursor[start + 1..];
            let Some(end) = after_start.find(')') else {
                break;
            };
            let target = &after_start[..end];
            if target.ends_with(".md") && !target.contains("://") {
                out.push(target.to_string());
            }
            cursor = &after_start[end + 1..];
        }
    }
    out
}

