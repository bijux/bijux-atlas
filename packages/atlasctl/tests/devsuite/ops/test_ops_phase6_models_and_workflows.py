from __future__ import annotations

from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from atlasctl.core.process import CommandResult
from atlasctl.ops.models.toolchain import ToolchainInventory
from atlasctl.ops.workflows import deploy as deploy_workflows
from atlasctl.ops.workflows.doctor import env_doctor
from atlasctl.ops.workflows import tests as test_workflows
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


def test_deploy_atlas_uses_helm_adapter_preflight(monkeypatch):
    ctx = SimpleNamespace(repo_root=Path.cwd(), run_id="r1")
    monkeypatch.setattr(deploy_workflows, "ensure_local_kind_context", lambda _ctx: (True, "kind-test"))
    monkeypatch.setattr(
        deploy_workflows,
        "helm",
        SimpleNamespace(run=lambda _ctx, *args: CommandResult(0, "v3", "", 1)),
    )
    monkeypatch.setattr(deploy_workflows.legacy, "_ops_deploy_native", lambda _ctx, _report: 0)
    assert deploy_workflows.deploy_atlas(ctx, "text") == 0


def test_test_load_includes_threshold_evidence(monkeypatch):
    ctx = SimpleNamespace(repo_root=Path.cwd(), run_id="r1")
    monkeypatch.setattr(test_workflows, "ensure_local_kind_context", lambda _ctx: (True, "kind-test"))
    monkeypatch.setattr(
        test_workflows,
        "k6",
        SimpleNamespace(run=lambda _ctx, *args: CommandResult(0, "k6 v0", "", 1)),
    )
    monkeypatch.setattr(test_workflows.legacy, "_ops_load_run_native", lambda *_a, **_k: 0)
    monkeypatch.setattr(test_workflows, "ops_run_area_dir", lambda *_a, **_k: Path.cwd() / "artifacts" / "runs" / "r1" / "ops" / "ops-load")
    with patch("builtins.print"):
        code = test_workflows.test_load(ctx, "text")
    assert code == 0
