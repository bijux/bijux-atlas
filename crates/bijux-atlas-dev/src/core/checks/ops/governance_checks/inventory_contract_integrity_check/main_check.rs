pub(super) fn check_ops_inventory_contract_integrity(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    check_ops_inventory_contract_integrity_aggregate_runner(ctx)
}
