# Automation Boundary Gap Review

## Outdated checks that assumed deprecated structure

The following checks were written before the delegation doctrine was fully restored and relied on assumptions that no longer hold:

1. `workflow_and_make_surfaces_do_not_call_legacy_script_paths` in `tests/automation_surface_contracts.rs`
2. `required_make_wrappers_must_delegate_to_dev_atlas` in `tests/repo_automation_doctrine_contracts.rs`
3. `workflows_must_not_execute_repo_bash_scripts` in `tests/repo_automation_doctrine_contracts.rs`

Observed gaps before this update:

1. No single command produced a unified automation boundary report.
2. No dedicated `checks` or `contract` command exposed boundary validation directly.
3. Some assertions were split across multiple test files without a consolidated evidence output.

## Outdated contracts that assumed legacy script locations

The following contract coverage had drift risk because historical exceptions and script-era paths were spread across separate checks:

1. `tools/` and `scripts/` root bans in `tests/repo_automation_doctrine_contracts.rs`
2. workflow shell policy checks in `tests/repo_automation_doctrine_contracts.rs`
3. make-wrapper purity checks in `tests/repo_automation_doctrine_contracts.rs`

This review keeps those controls and adds a single automation boundary scan/report command surface.
