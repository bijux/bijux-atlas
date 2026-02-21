from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
import time
from pathlib import Path

from ..core.context import RunContext


def _requirements_paths(repo_root: Path) -> tuple[Path, Path]:
    base = repo_root / "packages/atlasctl"
    return base / "requirements.in", base / "requirements.lock.txt"


def _normalize_requirements(req_in: Path, req_lock: Path) -> None:
    lines = [
        ln.strip()
        for ln in req_in.read_text(encoding="utf-8").splitlines()
        if ln.strip() and not ln.strip().startswith("#")
    ]
    req_lock.write_text("\n".join(sorted(set(lines))) + "\n", encoding="utf-8")


def _run(cmd: list[str], cwd: Path) -> tuple[int, str]:
    proc = subprocess.run(cmd, cwd=cwd, text=True, capture_output=True, check=False)
    return proc.returncode, (proc.stdout + proc.stderr).strip()


def run_deps_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    req_in, req_lock = _requirements_paths(ctx.repo_root)

    if ns.deps_cmd == "lock":
        _normalize_requirements(req_in, req_lock)
        print(req_lock.relative_to(ctx.repo_root).as_posix())
        return 0

    if ns.deps_cmd == "export-requirements":
        out = Path(getattr(ns, "out", "")).expanduser() if getattr(ns, "out", None) else req_lock
        _normalize_requirements(req_in, out)
        print(out.relative_to(ctx.repo_root).as_posix() if out.is_absolute() and out.is_relative_to(ctx.repo_root) else str(out))
        return 0

    if ns.deps_cmd == "sync":
        code, out = _run([sys.executable, "-m", "pip", "install", "--requirement", str(req_lock)], cwd=ctx.repo_root)
        if out:
            print(out)
        return code

    if ns.deps_cmd == "check-venv":
        with tempfile.TemporaryDirectory(prefix="atlasctl-deps-") as td:
            venv_dir = Path(td) / "venv"
            code, out = _run([sys.executable, "-m", "venv", str(venv_dir)], cwd=ctx.repo_root)
            if code != 0:
                print(out)
                return code
            py = venv_dir / ("Scripts/python.exe" if sys.platform.startswith("win") else "bin/python")
            code, out = _run([str(py), "-m", "pip", "install", "--requirement", str(req_lock)], cwd=ctx.repo_root)
            if code != 0:
                print(out)
                return code
            env = {"PYTHONPATH": str(ctx.repo_root / "packages/atlasctl/src")}
            proc = subprocess.run([str(py), "-m", "atlasctl", "--help"], cwd=ctx.repo_root, text=True, capture_output=True, env=env, check=False)
            if proc.returncode != 0:
                print((proc.stdout + proc.stderr).strip())
            return proc.returncode

    if ns.deps_cmd == "cold-start":
        runs = max(1, int(getattr(ns, "runs", 3)))
        budget_ms = int(getattr(ns, "max_ms", 500))
        samples: list[float] = []
        for _ in range(runs):
            t0 = time.perf_counter()
            proc = subprocess.run(
                [sys.executable, "-c", "import atlasctl.cli.main"],
                cwd=ctx.repo_root,
                text=True,
                capture_output=True,
                env={"PYTHONPATH": str(ctx.repo_root / "packages/atlasctl/src")},
                check=False,
            )
            elapsed_ms = (time.perf_counter() - t0) * 1000.0
            if proc.returncode != 0:
                print((proc.stdout + proc.stderr).strip())
                return proc.returncode
            samples.append(elapsed_ms)
        avg = sum(samples) / len(samples)
        print(f"cold-start-ms avg={avg:.1f} max={max(samples):.1f} runs={runs} budget={budget_ms}")
        return 0 if avg <= budget_ms else 1

    return 2


def configure_deps_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("deps", help="dependency workflow commands (pip-tools route)")
    ps = p.add_subparsers(dest="deps_cmd", required=True)

    ps.add_parser("lock", help="refresh requirements.lock.txt deterministically from requirements.in")

    export = ps.add_parser("export-requirements", help="export normalized requirements lock from requirements.in")
    export.add_argument("--out", help="output path", default="packages/atlasctl/requirements.lock.txt")

    ps.add_parser("sync", help="install lockfile dependencies into current interpreter env")
    ps.add_parser("check-venv", help="validate deps install/import in a clean temporary venv")

    cold = ps.add_parser("cold-start", help="measure atlasctl import cold-start time")
    cold.add_argument("--runs", type=int, default=3)
    cold.add_argument("--max-ms", type=int, default=500)
