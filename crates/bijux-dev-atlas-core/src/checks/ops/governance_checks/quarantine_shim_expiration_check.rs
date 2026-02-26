pub(super) fn check_ops_quarantine_shim_expiration_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let contract_rel = Path::new("ops/CONTRACT.md");
    if !ctx.adapters.fs.exists(ctx.repo_root, contract_rel) {
        return Ok(vec![violation(
            "OPS_SHIM_QUARANTINE_README_MISSING",
            format!(
                "missing shim expiration contract file `{}`",
                contract_rel.display()
            ),
            "declare shim expiration deadline in ops contract",
            Some(contract_rel),
        )]);
    }
    let text = fs::read_to_string(ctx.repo_root.join(contract_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let deadline_line = text.lines().find(|line| {
        line.trim_start()
            .starts_with("- Legacy shell compatibility deadline: ")
    });
    let Some(deadline_line) = deadline_line else {
        return Ok(vec![violation(
            "OPS_SHIM_EXPIRATION_MISSING",
            "ops contract must declare an explicit shim expiration deadline".to_string(),
            "add a deadline line in the form `- Legacy shell compatibility deadline: YYYY-MM-DD.`",
            Some(contract_rel),
        )]);
    };
    let deadline = deadline_line
        .trim_start()
        .trim_start_matches("- Legacy shell compatibility deadline: ")
        .trim_end_matches('.')
        .trim();
    let valid_deadline = deadline.len() == 10
        && deadline.chars().enumerate().all(|(idx, ch)| match idx {
            4 | 7 => ch == '-',
            _ => ch.is_ascii_digit(),
        });
    if !valid_deadline {
        return Ok(vec![violation(
            "OPS_SHIM_EXPIRATION_FORMAT_INVALID",
            format!("shim quarantine deadline has invalid format: `{deadline}`"),
            "use ISO date format YYYY-MM-DD in shim quarantine deadline",
            Some(contract_rel),
        )]);
    }
    Ok(Vec::new())
}
