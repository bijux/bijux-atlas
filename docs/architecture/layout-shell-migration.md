# Layout shell-to-python migration checklist

This checklist tracks migration of legacy shell checks under
`ops/vendor/layout-checks/`.

## Converted to Python (`checks/layout/root`)
- [x] `check_root_shape.sh` -> `root/check_root_shape.py` (`repo.root_shape`)
- [x] `check_no_forbidden_paths.sh` -> `root/check_forbidden_paths.py` (`repo.no_forbidden_paths`)
- [x] `check_no_direct_script_runs.sh` -> `root/check_no_direct_script_runs.py` (`repo.no_direct_script_runs`)
- [x] `check_root_determinism.sh` -> `root/check_root_determinism.py` (`repo.root_determinism`)
- [x] `check_forbidden_root_files.sh` -> `root/check_forbidden_root_files.py` (`repo.forbidden_root_files`)
- [x] `check_forbidden_root_names.sh` -> `root/check_forbidden_root_names.py` (`repo.forbidden_root_names`)

## Shell Inventory and Decision
- `check_artifacts_allowlist.sh`: keep (wrapped/transitional)
- `check_artifacts_policy.sh`: keep (wrapped/transitional)
- `check_kind_cluster_contract_drift.sh`: keep (wrapped/transitional)
- `check_no_root_dumping.sh`: keep (wrapped/transitional)
- `check_ops_canonical_shims.sh`: keep (wrapped/transitional)
- `check_ops_lib_canonical.sh`: keep (wrapped/transitional)
- `check_ops_script_targets.sh`: keep (wrapped/transitional)
- `check_ops_stack_order.sh`: keep (wrapped/transitional)
- `check_ops_workspace.sh`: keep (wrapped/transitional)
- `check_repo_hygiene.sh`: keep (wrapped/transitional)
- `check_scripts_readme_drift.sh`: keep (wrapped/transitional)
- `check_stack_manifest_consolidation.sh`: keep (wrapped/transitional)

## Policy
- New policy-critical checks must land as Python modules under `checks/layout/*`.
- Existing shell checks remain transitional and must keep strict shell headers.

## Long-Term Plan
- Reduce shell checks to near-zero by porting remaining `shell/layout/*.sh` into `checks/layout/*` Python modules.
