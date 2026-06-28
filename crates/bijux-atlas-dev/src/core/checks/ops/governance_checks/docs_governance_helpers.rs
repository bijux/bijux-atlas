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

fn extract_ops_command_refs(content: &str) -> BTreeSet<String> {
    let mut commands = BTreeSet::new();
    for line in content.lines() {
        let mut cursor = line;
        while let Some(pos) = cursor.find("bijux dev atlas ops ") {
            let after = &cursor[pos + "bijux dev atlas ops ".len()..];
            let mut tokens = Vec::new();
            for token in after.split_whitespace() {
                if token.starts_with("--")
                    || token.starts_with('`')
                    || token.starts_with('|')
                    || token.starts_with('(')
                {
                    break;
                }
                let clean = token
                    .trim_matches(|ch: char| ",.;:()[]`".contains(ch))
                    .to_string();
                if clean.is_empty() {
                    break;
                }
                if !clean
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
                {
                    break;
                }
                tokens.push(clean);
                if tokens.len() >= 3 {
                    break;
                }
            }
            if !tokens.is_empty() {
                commands.insert(format!("bijux dev atlas ops {}", tokens.join(" ")));
            }
            cursor = after;
        }
    }
    commands
}

