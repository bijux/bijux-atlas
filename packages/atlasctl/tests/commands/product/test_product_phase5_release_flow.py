from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[5]


def test_product_build_plan_prints_manifest_target() -> None:
    proc = subprocess.run(
        ["./bin/atlasctl", "product", "build", "--plan"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr
    out = proc.stdout
    assert "product build plan:" in out
    assert "artifact manifest" in out


def test_product_verify_alias_available() -> None:
    proc = subprocess.run(
        ["./bin/atlasctl", "product", "verify"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    # allow failure due to missing manifest; this asserts parser/command wiring exists
    combined = (proc.stdout or "") + (proc.stderr or "")
    assert (
        "missing product artifact manifest" in combined
        or "product validation requires pinned tool versions to pass" in combined
        or proc.returncode == 0
    )
