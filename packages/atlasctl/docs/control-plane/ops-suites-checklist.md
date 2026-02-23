# Add A New Ops Suite

Use this checklist when adding or changing an ops suite.

1. Add/update the suite in `configs/ops/suites.json`.
2. Set `speed` to `fast` or `slow`.
3. If `slow`, add `slow_reason` and `reduction_plan`.
4. Map the suite to an `atlasctl_suite` and declare `markers`.
5. Declare `actions` coverage and the correct `evidence_area`.
6. Update/load baseline manifests in `configs/ops/load/` if the suite is load-related.
7. Regenerate and verify suite membership goldens.
8. Run `check_ops_suites_contracts.py` and `suite check`.

