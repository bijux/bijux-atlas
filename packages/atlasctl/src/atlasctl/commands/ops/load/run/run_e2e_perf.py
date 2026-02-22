#!/usr/bin/env python3
from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _run(cmd: list[str], cwd: Path, env: dict[str, str] | None = None, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=str(cwd), env=env, check=check, text=True, capture_output=False)


def main() -> int:
    root = _repo_root()
    art = root / "artifacts/ops/e2e/k6"
    art.mkdir(parents=True, exist_ok=True)
    base_url = os.environ.get("ATLAS_BASE_URL", os.environ.get("BASE_URL", "http://127.0.0.1:18080"))
    pr_mode = os.environ.get("PR_MODE", "0")
    profile = "pr" if pr_mode == "1" else "full"

    subprocess.run(
        [str(root / "bin/atlasctl"), "run", "./packages/atlasctl/src/atlasctl/commands/ops/load/contracts/validate_suite_manifest.py"],
        check=True,
        cwd=str(root),
    )
    env = os.environ.copy()
    env["ATLAS_BASE_URL"] = base_url
    subprocess.run(
        [
            str(root / "bin/atlasctl"),
            "run",
            "./packages/atlasctl/src/atlasctl/commands/ops/load/run/run_suites_from_manifest.py",
            "--profile",
            profile,
            "--out",
            str(art),
        ],
        check=True,
        cwd=str(root),
        env=env,
    )

    if pr_mode != "1":
        env2 = os.environ.copy()
        env2["OUT_DIR"] = str(art)
        subprocess.run(
            [str(root / "bin/atlasctl"), "run", "./packages/atlasctl/src/atlasctl/commands/ops/load/run/cold_start_benchmark.py"],
            check=True,
            cwd=str(root),
            env=env2,
            stdout=subprocess.DEVNULL,
        )
        if (art / "result.json").exists():
            shutil.copyfile(art / "result.json", art / "cold_start.result.json")

    metrics_path = art / "metrics.prom"
    curl = subprocess.run(
        ["curl", "-fsS", f"{base_url}/metrics"],
        check=False,
        cwd=str(root),
        capture_output=True,
        text=True,
    )
    metrics_path.write_text(curl.stdout or "", encoding="utf-8")
    if not metrics_path.exists() or metrics_path.stat().st_size == 0:
        raise SystemExit(f"runtime metrics snapshot missing: {metrics_path}")

    for rel in (
        "packages/atlasctl/src/atlasctl/commands/ops/load/reports/score_k6.py",
        "packages/atlasctl/src/atlasctl/commands/ops/load/reports/validate_results.py",
    ):
        cmd = [str(root / "bin/atlasctl"), "run", f"./{rel}"]
        if rel.endswith("validate_results.py"):
            cmd.append(str(art))
        subprocess.run(cmd, check=True, cwd=str(root))
    subprocess.run(["python3", "ops/load/reports/generate.py"], check=True, cwd=str(root))
    print(f"e2e perf complete: {art}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
