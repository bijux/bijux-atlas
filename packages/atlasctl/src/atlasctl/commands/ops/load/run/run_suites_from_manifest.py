#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()
MANIFEST = json.loads((ROOT / "ops/load/suites/suites.json").read_text())


def run(cmd: list[str], env: dict[str, str] | None = None) -> None:
    subprocess.run(cmd, check=True, env=env)


def in_profile(suite: dict[str, object], profile: str) -> bool:
    if profile == "all":
        return True
    return profile in suite.get("run_in", [])


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--profile",
        choices=["smoke", "full", "all", "nightly", "pr", "load-ci", "load-nightly"],
        required=True,
    )
    ap.add_argument("--out", default="artifacts/perf/results")
    args = ap.parse_args()

    out = (ROOT / args.out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    redis_enabled = os.environ.get("ATLAS_ENABLE_REDIS_EXPERIMENT", "0") == "1"

    for suite in MANIFEST.get("suites", []):
        if not in_profile(suite, args.profile):
            continue
        if suite.get("requires", {}).get("redis_experiment", False) and not redis_enabled:
            print(f"skip redis experiment suite: {suite['name']}")
            continue
        if suite.get("kind") == "k6":
            scenario = suite["scenario"]
            run(
                [
                    str(ROOT / "bin/atlasctl"),
                    "ops",
                    "load",
                    "--report",
                    "text",
                    "run",
                    "--suite",
                    scenario,
                    "--out",
                    str(out),
                ]
            )
        elif suite.get("kind") == "script":
            script = ROOT / suite["script"]
            env = os.environ.copy()
            env["OUT_DIR"] = str(out)
            run([str(script)], env=env)
            result = out / "result.json"
            if result.exists():
                payload = json.loads(result.read_text())
                if isinstance(payload, list) and payload:
                    vals = sorted(float(x.get("cold_start_ms", 0.0)) for x in payload)
                    n = len(vals)
                    p95 = vals[min(n - 1, int(0.95 * (n - 1)))]
                    p99 = vals[min(n - 1, int(0.99 * (n - 1)))]
                    max_v = max(vals)
                    summary = {
                        "metrics": {
                            "http_req_duration": {"values": {"p(95)": p95, "p(99)": p99}},
                            "pod_cold_start_ms": {"values": {"max": max_v}},
                        }
                    }
                    (out / f"{suite['name']}.summary.json").write_text(json.dumps(summary, indent=2) + "\n")
                    meta = {
                        "suite": suite["name"],
                        "resolved_suite": suite["script"],
                        "git_sha": os.environ.get("GITHUB_SHA", "local"),
                        "image_digest": os.environ.get("ATLAS_IMAGE_DIGEST", "unknown"),
                        "dataset_hash": os.environ.get("ATLAS_DATASET_HASH", "unknown"),
                        "dataset_release": os.environ.get("ATLAS_DATASET_RELEASE", "unknown"),
                        "policy_hash": os.environ.get("ATLAS_POLICY_HASH", "unknown"),
                    }
                    (out / f"{suite['name']}.meta.json").write_text(json.dumps(meta, indent=2) + "\n")
                result.unlink(missing_ok=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
