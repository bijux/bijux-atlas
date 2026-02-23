from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[5]


def _run(rel: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["python3", rel], cwd=ROOT, text=True, capture_output=True, check=False)


def test_ops_import_boundary_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_ops_command_import_boundary.py")
    assert proc.returncode == 0, proc.stderr


def test_ops_internal_public_boundary_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_ops_internal_not_public.py")
    assert proc.returncode == 0, proc.stderr


def test_ops_subprocess_boundary_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_ops_subprocess_boundary.py")
    assert proc.returncode == 0, proc.stderr


def test_ops_scenarios_atlasctl_only_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_ops_scenarios_atlasctl_only.py")
    assert proc.returncode == 0, proc.stderr


def test_ops_forbidden_runtime_tokens_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_ops_forbidden_runtime_tokens.py")
    assert proc.returncode == 0, proc.stderr


def test_ops_temporary_shims_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_ops_temporary_shims.py")
    assert proc.returncode == 0, proc.stderr


def test_ops_report_contract_fields_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_ops_report_contract_fields.py")
    assert proc.returncode == 0, proc.stderr


def test_ops_actions_docs_generated_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_ops_actions_docs_generated.py")
    assert proc.returncode == 0, proc.stderr


def test_ops_product_task_scripts_zero_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_ops_product_task_scripts_zero.py")
    assert proc.returncode == 0, proc.stderr


def test_no_core_models_duplicate_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/architecture/check_no_core_models_duplicate.py")
    assert proc.returncode == 0, proc.stderr


def test_commands_no_ops_tests_fixtures_imports_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/architecture/check_commands_no_ops_tests_fixtures_imports.py")
    assert proc.returncode == 0, proc.stderr


def test_ops_commands_import_policy_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/architecture/check_ops_commands_import_policy.py")
    assert proc.returncode == 0, proc.stderr


def test_ops_command_group_entrypoints_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/architecture/check_ops_command_group_entrypoints.py")
    assert proc.returncode == 0, proc.stderr


def test_registry_reads_centralized_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/architecture/check_registry_reads_centralized.py")
    assert proc.returncode == 0, proc.stderr


def test_ops_runtime_modules_naming_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/architecture/check_ops_runtime_modules_naming.py")
    assert proc.returncode == 0, proc.stderr


def test_no_floating_ops_modules_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/architecture/check_no_floating_ops_modules.py")
    assert proc.returncode == 0, proc.stderr


def test_no_print_in_ops_command_entrypoints_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/architecture/check_no_print_in_ops_command_entrypoints.py")
    assert proc.returncode == 0, proc.stderr


def test_no_stdout_writes_in_ops_command_entrypoints_check_passes() -> None:
    proc = _run(
        "packages/atlasctl/src/atlasctl/checks/layout/architecture/check_no_stdout_writes_in_ops_command_entrypoints.py"
    )
    assert proc.returncode == 0, proc.stderr


def test_ops_cli_profile_network_flags_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/architecture/check_ops_cli_profile_network_flags.py")
    assert proc.returncode == 0, proc.stderr


def test_ops_no_path_dot_in_runtime_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/architecture/check_ops_no_path_dot_in_runtime.py")
    assert proc.returncode == 0, proc.stderr


def test_ops_no_cwd_reliance_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/architecture/check_ops_no_cwd_reliance.py")
    assert proc.returncode == 0, proc.stderr


def test_ops_command_capabilities_manifest_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/architecture/check_ops_command_capabilities_manifest.py")
    assert proc.returncode == 0, proc.stderr
