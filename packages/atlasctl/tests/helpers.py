from __future__ import annotations

import os
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
GOLDENS_ROOT = ROOT / "packages/atlasctl/tests/goldens"
GOLDENS_MANIFEST = GOLDENS_ROOT / "MANIFEST.json"


def run_atlasctl(*args: str, cwd: Path | None = None, evidence_root: Path | None = None) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env["PYTHONPATH"] = str(ROOT / "packages/atlasctl/src")
    if evidence_root is not None:
        env["EVIDENCE_ROOT"] = str(evidence_root)
        env.setdefault("RUN_ID", "pytest-run")
        env.setdefault("PROFILE", "test")
        return subprocess.run(
            [sys.executable, "-m", "atlasctl.cli", *args],
            cwd=(cwd or ROOT),
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )
    with tempfile.TemporaryDirectory(prefix="atlasctl-evidence-") as td:
        env["EVIDENCE_ROOT"] = td
        env.setdefault("RUN_ID", "pytest-run")
        env.setdefault("PROFILE", "test")
        return subprocess.run(
            [sys.executable, "-m", "atlasctl.cli", *args],
            cwd=(cwd or ROOT),
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )


def run_atlasctl_isolated(tmp_path: Path, *args: str, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    evidence_root = tmp_path / "evidence"
    evidence_root.mkdir(parents=True, exist_ok=True)
    return run_atlasctl(*args, cwd=cwd, evidence_root=evidence_root)


def _manifest_entries() -> list[dict[str, str]]:
    payload = json.loads(GOLDENS_MANIFEST.read_text(encoding="utf-8"))
    entries = payload.get("entries", [])
    return [row for row in entries if isinstance(row, dict)]


def golden_path(name_or_relpath: str) -> Path:
    direct = GOLDENS_ROOT / name_or_relpath
    if direct.exists():
        return direct
    matches = [row for row in _manifest_entries() if row.get("name") == name_or_relpath]
    if len(matches) != 1:
        raise FileNotFoundError(f"golden reference `{name_or_relpath}` not uniquely mapped in MANIFEST.json")
    rel = str(matches[0]["path"])
    path = GOLDENS_ROOT / rel
    if not path.exists():
        raise FileNotFoundError(f"golden manifest points to missing file: {rel}")
    return path


def golden_text(name_or_relpath: str) -> str:
    return golden_path(name_or_relpath).read_text(encoding="utf-8").strip()
