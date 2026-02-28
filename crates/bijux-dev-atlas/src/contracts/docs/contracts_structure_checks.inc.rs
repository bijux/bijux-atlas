fn test_docs_003_depth_budget(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let docs_root = docs_root_path(ctx);
    let mut stack = vec![docs_root.clone()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(err) => {
                let rel = dir
                    .strip_prefix(&ctx.repo_root)
                    .map(|value| value.display().to_string())
                    .unwrap_or_else(|_| dir.display().to_string());
                push_docs_violation(
                    &mut violations,
                    "DOC-003",
                    "docs.structure.depth_budget",
                    Some(rel),
                    format!("read dir failed: {err}"),
                );
                continue;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let rel = match path.strip_prefix(&docs_root) {
                Ok(value) => value,
                Err(_) => continue,
            };
            let depth = rel.components().count();
            if depth > DOCS_MAX_DEPTH {
                let rel_path = path
                    .strip_prefix(&ctx.repo_root)
                    .map(|value| value.display().to_string())
                    .unwrap_or_else(|_| path.display().to_string());
                push_docs_violation(
                    &mut violations,
                    "DOC-003",
                    "docs.structure.depth_budget",
                    Some(rel_path),
                    format!("docs depth {depth} exceeds budget {DOCS_MAX_DEPTH}"),
                );
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_docs_004_sibling_budget(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let docs_root = docs_root_path(ctx);
    let mut stack = vec![docs_root.clone()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(err) => {
                let rel = dir
                    .strip_prefix(&ctx.repo_root)
                    .map(|value| value.display().to_string())
                    .unwrap_or_else(|_| dir.display().to_string());
                push_docs_violation(
                    &mut violations,
                    "DOC-004",
                    "docs.structure.sibling_budget",
                    Some(rel),
                    format!("read dir failed: {err}"),
                );
                continue;
            }
        };
        let mut count = 0usize;
        for entry in entries.flatten() {
            count += 1;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            }
        }
        if count > DOCS_MAX_SIBLINGS {
            let rel = dir
                .strip_prefix(&ctx.repo_root)
                .map(|value| value.display().to_string())
                .unwrap_or_else(|_| dir.display().to_string());
            push_docs_violation(
                &mut violations,
                "DOC-004",
                "docs.structure.sibling_budget",
                Some(rel),
                format!("docs sibling count {count} exceeds budget {DOCS_MAX_SIBLINGS}"),
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}
