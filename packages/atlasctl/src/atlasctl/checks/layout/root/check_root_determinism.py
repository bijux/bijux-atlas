from __future__ import annotations

import shutil
import tempfile
from pathlib import Path

from ....core.exec import run as run_cmd

CHECK_ID = "repo.root_determinism"
DESCRIPTION = "verify deterministic root output across two make root runs"

_OUTPUTS = (
    "docs/_generated/repo-surface.md",
    "docs/_generated/naming-inventory.md",
    "docs/_generated/make-targets.md",
)

_GEN_CMDS = (
    ["./bin/atlasctl", "docs", "generate-repo-surface"],
    ["./bin/atlasctl", "docs", "naming-inventory"],
    ["./bin/atlasctl", "docs", "generate-make-targets-catalog"],
)


def _capture_outputs(repo_root: Path, run_id: str, out_dir: Path) -> tuple[int, list[str]]:
    for cmd in _GEN_CMDS:
        proc = run_cmd(cmd, cwd=repo_root, text=True, capture_output=True)
        if proc.returncode != 0:
            message = (proc.stderr or proc.stdout or "root determinism generation failed").strip()
            return 1, [f"{run_id}: {' '.join(cmd)} failed: {message}"]

    errors: list[str] = []
    for rel in _OUTPUTS:
        source = repo_root / rel
        if not source.exists():
            errors.append(f"{run_id}: expected output missing: {rel}")
            continue
        target = out_dir / rel.replace("/", "__")
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
    return (0 if not errors else 1), errors


def run(repo_root: Path) -> tuple[int, list[str]]:
    with tempfile.TemporaryDirectory(prefix="atlasctl-root-det-") as tmp:
        tmpdir = Path(tmp)
        out_a = tmpdir / "a"
        out_b = tmpdir / "b"
        out_a.mkdir(parents=True, exist_ok=True)
        out_b.mkdir(parents=True, exist_ok=True)

        code_a, errors_a = _capture_outputs(repo_root, "det-a", out_a)
        code_b, errors_b = _capture_outputs(repo_root, "det-b", out_b)
        if code_a != 0 or code_b != 0:
            return 1, [*errors_a, *errors_b]

        diffs: list[str] = []
        for rel in _OUTPUTS:
            name = rel.replace("/", "__")
            a = (out_a / name).read_text(encoding="utf-8")
            b = (out_b / name).read_text(encoding="utf-8")
            if a != b:
                diffs.append(f"non-deterministic output: {rel}")
        return (0 if not diffs else 1), diffs
