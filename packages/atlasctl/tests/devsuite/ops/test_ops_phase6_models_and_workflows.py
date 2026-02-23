from __future__ import annotations

from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from atlasctl.ops.models.toolchain import ToolchainInventory
from atlasctl.ops.workflows.doctor import env_doctor
from atlasctl.ops.workflows.guards import require_destructive_ops_allowed


def test_toolchain_inventory_loads_yaml():
    repo_root = Path.cwd()
    inv = ToolchainInventory.load(repo_root)
    assert inv.source_path == "ops/inventory/toolchain.yaml"
    assert inv.schema_version >= 1
    assert isinstance(inv.images, dict)


def test_env_doctor_reads_toolchain_yaml():
    repo_root = Path.cwd()
    ctx = SimpleNamespace(repo_root=repo_root, run_id="run-1")
    payload = env_doctor(ctx)
    assert payload["toolchain_source"] == "ops/inventory/toolchain.yaml"
    assert payload["schema_name"] == "atlasctl.ops-env-doctor.v1"
    assert isinstance(payload["images"], list)


def test_destructive_guard_requires_explicit_opt_in():
    with patch("atlasctl.ops.workflows.guards.os.environ", {}, create=True):
        ok, message = require_destructive_ops_allowed("ops platform down")
    assert ok is False
    assert "ATLASCTL_OPS_ALLOW_DESTRUCTIVE" in message
