#!/usr/bin/env python3
import argparse
import json
import os
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = json.loads((ROOT / "ops/load/suites/suites.json").read_text())


def run(cmd, env=None):
    subprocess.run(cmd, check=True, env=env)


def in_profile(suite, profile):
    return profile in suite.get("run_in", [])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--profile",
        choices=["smoke", "full", "nightly", "pr", "load-ci", "load-nightly"],
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
            run([str(ROOT / "ops/load/scripts/run_suite.sh"), scenario, str(out)])
        elif suite.get("kind") == "script":
            script = ROOT / suite["script"]
            env = os.environ.copy()
            env["OUT_DIR"] = str(out)
            run([str(script)], env=env)
            # Normalize script output into a summary-like file for downstream reports.
            result = out / "result.json"
            if result.exists():
                payload = json.loads(result.read_text())
                if isinstance(payload, list) and payload:
                    vals = sorted(float(x.get("cold_start_ms", 0.0)) for x in payload)
                    n = len(vals)
                    p95 = vals[min(n - 1, int(0.95 * (n - 1)))]
                    p99 = vals[min(n - 1, int(0.99 * (n - 1)))]
                    mx = max(vals)
                    summary = {
                        "metrics": {
                            "http_req_duration": {"values": {"p(95)": p95, "p(99)": p99}},
                            "pod_cold_start_ms": {"values": {"max": mx}},
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


if __name__ == "__main__":
    main()
