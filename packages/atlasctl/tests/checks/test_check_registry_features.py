from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]


def _run(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    extra: list[str] = []
    if os.environ.get("BIJUX_SCRIPTS_TEST_NO_NETWORK") == "1":
        extra.append("--no-network")
    return subprocess.run([sys.executable, "-m", "atlasctl.cli", *extra, *args], cwd=ROOT, env=env, text=True, capture_output=True, check=False)


def test_check_list_json_inventory() -> None:
    proc = _run("--json", "check", "list")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["status"] == "ok"
    assert any(c["id"] == "repo.no_xtask_refs" for c in payload["checks"])
    assert any(c["id"] == "repo.no_direct_python_invocations" for c in payload["checks"])
    assert any(c["id"] == "repo.public_api_exports" for c in payload["checks"])
    assert any(c["id"] == "license.file_mit" for c in payload["checks"])


def test_check_explain_json() -> None:
    proc = _run("--json", "check", "explain", "repo.no_xtask_refs")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["id"] == "repo.no_xtask_refs"
    assert "how_to_fix" in payload


def test_check_repo_module_size_alias() -> None:
    proc = _run("check", "repo", "module-size")
    assert proc.returncode in (0, 1), proc.stderr


def test_check_license_alias() -> None:
    proc = _run("--json", "check", "license")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["domain"] == "license"
    assert payload["status"] == "pass"
