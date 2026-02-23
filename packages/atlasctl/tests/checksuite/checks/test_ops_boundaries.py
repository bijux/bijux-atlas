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
