#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import shutil
import signal
import subprocess
import sys
import time
import urllib.request
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _readyz(port: int) -> bool:
    try:
        with urllib.request.urlopen(f"http://127.0.0.1:{port}/readyz", timeout=1) as resp:  # nosec B310
            return int(getattr(resp, "status", 500)) == 200
    except Exception:
        return False


def main() -> int:
    root = _repo_root()
    out_dir = Path(os.environ.get("OUT_DIR", str(root / "artifacts/perf/cold-start-5pods")))
    out_dir.mkdir(parents=True, exist_ok=True)

    bin_path = Path(os.environ.get("ATLAS_SERVER_BIN", str(root / "artifacts/target/debug/atlas-server")))
    if not bin_path.is_file() or not os.access(bin_path, os.X_OK):
        print(f"atlas-server binary not found at {bin_path}", file=sys.stderr)
        return 1

    store_root = os.environ.get("ATLAS_STORE_ROOT", str(root / "artifacts/server-store"))
    base_port = int(os.environ.get("ATLAS_BASE_PORT", "18080"))
    pods = 5
    procs: list[subprocess.Popen[bytes]] = []

    try:
        results = []
        for i in range(1, pods + 1):
            port = base_port + i - 1
            cache_root = root / f"artifacts/server-cache-pod-{i}"
            shutil.rmtree(cache_root, ignore_errors=True)
            cache_root.mkdir(parents=True, exist_ok=True)
            env = os.environ.copy()
            env.update(
                {
                    "ATLAS_BIND": f"127.0.0.1:{port}",
                    "ATLAS_STORE_ROOT": store_root,
                    "ATLAS_CACHE_ROOT": str(cache_root),
                    "ATLAS_STARTUP_WARMUP_JITTER_MAX_MS": "0",
                }
            )
            log_path = out_dir / f"pod-{i}.log"
            logf = open(log_path, "wb")
            proc = subprocess.Popen([str(bin_path)], stdout=logf, stderr=subprocess.STDOUT, env=env)
            procs.append(proc)

            start_ms = int(time.time() * 1000)
            for _ in range(200):
                if _readyz(port):
                    break
                time.sleep(0.1)
            end_ms = int(time.time() * 1000)
            results.append({"pod": i, "port": port, "cold_start_ms": end_ms - start_ms})

        (out_dir / "result.json").write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")
        print(f"wrote {out_dir / 'result.json'}")
        return 0
    finally:
        for proc in procs:
            if proc.poll() is None:
                proc.terminate()
        deadline = time.time() + 5
        for proc in procs:
            if proc.poll() is None:
                timeout = max(0.1, deadline - time.time())
                try:
                    proc.wait(timeout=timeout)
                except subprocess.TimeoutExpired:
                    proc.kill()
        for proc in procs:
            if proc.poll() is None:
                os.kill(proc.pid, signal.SIGKILL)


if __name__ == "__main__":
    raise SystemExit(main())
