from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]


def _run(rel: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["python3", rel], cwd=ROOT, text=True, capture_output=True, check=False)


def test_product_artifact_manifest_contract_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/product/validation/check_product_artifact_manifest_contract.py")
    assert proc.returncode == 0, proc.stderr


def test_product_build_write_roots_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/product/validation/check_product_build_write_roots.py")
    assert proc.returncode == 0, proc.stderr


def test_product_pinned_tools_policy_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/product/validation/check_product_pinned_tools_policy.py")
    assert proc.returncode == 0, proc.stderr


def test_product_provenance_and_tmp_policy_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/product/validation/check_product_provenance_and_artifact_roots_policy.py")
    assert proc.returncode == 0, proc.stderr


def test_product_external_tools_manifest_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/product/validation/check_product_external_tools_manifest.py")
    assert proc.returncode == 0, proc.stderr


def test_product_release_tag_contract_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/product/validation/check_product_release_tag_contract.py")
    assert proc.returncode == 0, proc.stderr


def test_schema_version_bump_playbook_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/product/validation/check_schema_version_bump_playbook.py")
    assert proc.returncode == 0, proc.stderr


def test_support_policy_docs_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/product/validation/check_support_policy_docs.py")
    assert proc.returncode == 0, proc.stderr


def test_release_notes_generator_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/product/validation/check_product_release_notes_generator.py")
    assert proc.returncode == 0, proc.stderr


def test_goldens_update_workflow_command_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/product/validation/check_goldens_update_workflow_command.py")
    assert proc.returncode == 0, proc.stderr
