#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import subprocess
import time
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _run(cmd: list[str], cwd: Path | None = None, env: dict[str, str] | None = None) -> None:
    subprocess.run(cmd, check=True, cwd=str(cwd) if cwd else None, env=env)


def _wait_ready(port: int) -> None:
    for _ in range(60):
        proc = subprocess.run(
            ["curl", "-fsS", f"http://127.0.0.1:{port}/readyz"],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        if proc.returncode == 0:
            return
        time.sleep(1)


def _read_summary(path: Path) -> dict[str, float]:
    data = json.loads(path.read_text(encoding="utf-8"))
    values = data.get("metrics", {}).get("http_req_duration", {}).get("values", {})
    failed = data.get("metrics", {}).get("http_req_failed", {}).get("values", {})
    return {
        "p50": float(values.get("p(50)", 0.0)),
        "p95": float(values.get("p(95)", 0.0)),
        "p99": float(values.get("p(99)", 0.0)),
        "fail": float(failed.get("rate", 0.0)),
    }


def main() -> int:
    root = _repo_root()
    out = root / "artifacts/perf/redis-compare"
    (out / "no-redis").mkdir(parents=True, exist_ok=True)
    (out / "with-redis").mkdir(parents=True, exist_ok=True)
    (root / "artifacts/perf/cache").mkdir(parents=True, exist_ok=True)

    _run(
        [
            str(root / "bin/atlasctl"),
            "run",
            "./packages/atlasctl/src/atlasctl/commands/ops/load/run/prepare_perf_store.py",
            str(root / "artifacts/perf/store"),
        ],
        cwd=root,
    )

    def run_stack(compose_file: Path, port: int, out_dir: Path) -> None:
        _run(["docker", "compose", "-f", str(compose_file), "down", "--remove-orphans"], cwd=root)
        _run(["docker", "compose", "-f", str(compose_file), "up", "-d", "--build"], cwd=root)
        try:
            _wait_ready(port)
            env = os.environ.copy()
            env.update({"ATLAS_BASE_URL": f"http://127.0.0.1:{port}", "RATE": "2000", "DURATION": "90s"})
            _run(
                [
                    str(root / "bin/atlasctl"),
                    "ops",
                    "load",
                    "--report",
                    "text",
                    "run",
                    "--suite",
                    "mixed.json",
                    "--out",
                    str(out_dir),
                ],
                cwd=root,
                env=env,
            )
        finally:
            subprocess.run(
                ["docker", "compose", "-f", str(compose_file), "down", "--remove-orphans"],
                check=False,
                cwd=str(root),
            )

    run_stack(root / "ops/load/compose/docker-compose.perf.yml", 18080, out / "no-redis")
    run_stack(root / "ops/load/compose/docker-compose.perf.redis.yml", 18081, out / "with-redis")

    n = _read_summary(out / "no-redis/mixed.summary.json")
    r = _read_summary(out / "with-redis/mixed.summary.json")
    lines = [
        "# Redis Perf Comparison (10x mixed load)",
        "",
        "| mode | p50 ms | p95 ms | p99 ms | fail rate |",
        "|---|---:|---:|---:|---:|",
        f"| no redis | {n['p50']:.2f} | {n['p95']:.2f} | {n['p99']:.2f} | {n['fail']:.4f} |",
        f"| redis enabled | {r['p50']:.2f} | {r['p95']:.2f} | {r['p99']:.2f} | {r['fail']:.4f} |",
        "",
        f"p95 improvement (ms): {n['p95'] - r['p95']:.2f}",
    ]
    out_file = out / "comparison.md"
    out_file.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(out_file)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
