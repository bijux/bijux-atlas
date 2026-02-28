fn test_root_001_surface_allowlist(ctx: &RunContext) -> TestResult {
    let mut allowed = ROOT_ALLOWED_VISIBLE
        .iter()
        .chain(ROOT_ALLOWED_VISIBLE_TAIL.iter())
        .map(|value| (*value).to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let mut violations = Vec::new();
    let entries = match std::fs::read_dir(&ctx.repo_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-001".to_string(),
                test_id: "root.surface.allowlist".to_string(),
                file: None,
                line: None,
                message: format!("read repo root failed: {err}"),
                evidence: None,
            }])
        }
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if ROOT_IGNORED_LOCAL.iter().any(|ignored| *ignored == name) {
            continue;
        }
        if !allowed.remove(&name) {
            violations.push(Violation {
                contract_id: "ROOT-001".to_string(),
                test_id: "root.surface.allowlist".to_string(),
                file: Some(name),
                line: None,
                message: "unexpected repo root entry".to_string(),
                evidence: None,
            });
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}
