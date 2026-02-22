#!/usr/bin/env python3
from __future__ import annotations

import atexit
import os
import shutil
import subprocess
import time
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _run(cmd: list[str], cwd: Path, check: bool = True, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=str(cwd), check=check, env=env, text=True, capture_output=False)


def main() -> int:
    root = _repo_root()
    art = root / "artifacts/perf"
    res = art / "results"
    (art / "cache").mkdir(parents=True, exist_ok=True)
    res.mkdir(parents=True, exist_ok=True)
    base_url = os.environ.get("ATLAS_BASE_URL", "http://127.0.0.1:18080")
    compose = root / "ops/load/compose/docker-compose.perf.yml"

    def cleanup() -> None:
        subprocess.run(["docker", "compose", "-f", str(compose), "down", "--remove-orphans"], cwd=str(root), check=False)

    atexit.register(cleanup)

    subprocess.run(
        [str(root / "bin/atlasctl"), "run", "./packages/atlasctl/src/atlasctl/commands/ops/load/run/prepare_perf_store.py", str(art / "store")],
        check=True,
        cwd=str(root),
    )
    subprocess.run(["docker", "compose", "-f", str(compose), "up", "-d", "--build"], check=True, cwd=str(root))

    for _ in range(60):
        if subprocess.run(["curl", "-fsS", f"{base_url}/readyz"], check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0:
            break
        time.sleep(1)

    ns = os.environ.get("ATLAS_E2E_NAMESPACE", "atlas-e2e")
    if shutil.which("kubectl"):
        subprocess.run(["kubectl", "-n", ns, "top", "pods"], cwd=str(root), check=False, stdout=open(art / "kubectl_top_pods_start.txt", "w", encoding="utf-8"), stderr=subprocess.DEVNULL)
    env = os.environ.copy()
    env["OUT_DIR"] = str(art / "cold-start")
    subprocess.run(
        [str(root / "bin/atlasctl"), "run", "./packages/atlasctl/src/atlasctl/commands/ops/load/run/cold_start_benchmark.py"],
        check=True,
        cwd=str(root),
        env=env,
    )

    for rel in (
        "packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_prereqs.py",
        "packages/atlasctl/src/atlasctl/commands/ops/load/contracts/validate_suite_manifest.py",
    ):
        subprocess.run([str(root / "bin/atlasctl"), "run", f"./{rel}"], check=True, cwd=str(root))

    with open(art / "docker_stats_soak_start.json", "w", encoding="utf-8") as fh:
        subprocess.run(["docker", "stats", "--no-stream", "--format", "{{json .}}"], cwd=str(root), check=False, stdout=fh)
    subprocess.run(
        [
            str(root / "bin/atlasctl"),
            "run",
            "./packages/atlasctl/src/atlasctl/commands/ops/load/run/run_suites_from_manifest.py",
            "--profile",
            "nightly",
            "--out",
            str(res),
        ],
        check=True,
        cwd=str(root),
    )
    with open(art / "docker_stats_soak_end.json", "w", encoding="utf-8") as fh:
        subprocess.run(["docker", "stats", "--no-stream", "--format", "{{json .}}"], cwd=str(root), check=False, stdout=fh)
    with open(art / "docker_stats.json", "w", encoding="utf-8") as fh:
        subprocess.run(["docker", "stats", "--no-stream", "--format", "{{json .}}"], cwd=str(root), check=False, stdout=fh)

    metrics = subprocess.run(["curl", "-fsS", f"{base_url}/metrics"], cwd=str(root), check=False, capture_output=True, text=True)
    (art / "metrics.prom").write_text(metrics.stdout or "", encoding="utf-8")
    if shutil.which("kubectl"):
        with open(art / "kubectl_top_pods_end.txt", "w", encoding="utf-8") as fh:
            subprocess.run(["kubectl", "-n", ns, "top", "pods"], cwd=str(root), check=False, stdout=fh, stderr=subprocess.DEVNULL)

    for cmd in (
        "./packages/atlasctl/src/atlasctl/commands/ops/load/reports/generate_report.py",
        "./packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_regression.py",
    ):
        subprocess.run([str(root / "bin/atlasctl"), "run", cmd], check=True, cwd=str(root))
    subprocess.run(
        [str(root / "bin/atlasctl"), "run", "./packages/atlasctl/src/atlasctl/commands/ops/load/reports/validate_results.py", str(res)],
        check=True,
        cwd=str(root),
    )
    subprocess.run(["python3", "ops/load/reports/generate.py"], check=True, cwd=str(root))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
