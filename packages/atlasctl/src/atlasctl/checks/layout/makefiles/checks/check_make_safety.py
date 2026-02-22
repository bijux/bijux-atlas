"""Compatibility shim; canonical implementation moved to `atlasctl.checks.domains.policies.make.impl.check_make_safety`."""

from importlib import import_module

_IMPL = import_module("atlasctl.checks.domains.policies.make.impl.check_make_safety")
check_make_no_direct_scripts_only_atlasctl = _IMPL.check_make_no_direct_scripts_only_atlasctl
check_make_no_direct_python_only_atlasctl = _IMPL.check_make_no_direct_python_only_atlasctl
check_make_no_direct_bash_ops = _IMPL.check_make_no_direct_bash_ops
check_make_no_direct_artifact_writes = _IMPL.check_make_no_direct_artifact_writes
check_make_no_direct_script_exec_drift = _IMPL.check_make_no_direct_script_exec_drift
check_make_no_bypass_atlasctl_without_allowlist = _IMPL.check_make_no_bypass_atlasctl_without_allowlist
check_ci_workflows_call_make_and_make_calls_atlasctl = _IMPL.check_ci_workflows_call_make_and_make_calls_atlasctl
check_public_make_targets_map_to_atlasctl = _IMPL.check_public_make_targets_map_to_atlasctl
check_make_no_direct_scripts_legacy = _IMPL.check_make_no_direct_scripts_legacy

__all__ = [
    "check_make_no_direct_scripts_only_atlasctl",
    "check_make_no_direct_python_only_atlasctl",
    "check_make_no_direct_bash_ops",
    "check_make_no_direct_artifact_writes",
    "check_make_no_direct_script_exec_drift",
    "check_make_no_bypass_atlasctl_without_allowlist",
    "check_ci_workflows_call_make_and_make_calls_atlasctl",
    "check_public_make_targets_map_to_atlasctl",
    "check_make_no_direct_scripts_legacy",
]
