pub(super) fn check_ops_fixture_governance(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations: Vec<Violation> = Vec::new();
    validate_dataset_fixture_policies_and_filesystem(ctx, &mut violations)?;
    validate_e2e_fixture_and_mapping_governance(ctx, &mut violations)?;
    validate_fixture_inventory_and_drift_reports(ctx, &mut violations)?;
    Ok(violations)
}

