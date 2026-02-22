#!/usr/bin/env python3
# Purpose: validate load result artifacts against result contract and metadata policy.
# Inputs: result summary files under an output directory and contract schema.
# Outputs: non-zero exit on contract violations.
from __future__ import annotations

import json
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()
OUT = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "artifacts/perf/results"
schema = json.loads((ROOT / "ops/load/contracts/result-schema.json").read_text())
suite_manifest = json.loads((ROOT / "ops/load/suites/suites.json").read_text())
required_metrics = set(schema["properties"]["metrics"]["required"])
errors: list[str] = []

expected_by_result = {}
for suite in suite_manifest.get("suites", []):
    if suite.get("kind") == "k6":
        scenario = suite.get("scenario", "")
        stem = Path(scenario).stem
        if stem:
            expected_by_result[stem] = set(suite.get("expected_metrics", []))
    elif suite.get("kind") == "script":
        expected_by_result[suite.get("name", "")] = set(suite.get("expected_metrics", []))

for file in sorted(OUT.glob("*.summary.json")):
    data = json.loads(file.read_text())
    metrics = data.get("metrics", {})
    missing = sorted(required_metrics - set(metrics.keys()))
    if missing:
        errors.append(f"{file}: missing metrics keys {missing}")
    expected_metrics = expected_by_result.get(file.stem.replace(".summary", ""), set())
    for key in sorted(expected_metrics):
        if key not in metrics:
            errors.append(f"{file}: missing expected metric '{key}' from suite manifest")
    meta = file.with_suffix(".meta.json")
    if not meta.exists():
        errors.append(f"{file}: missing metadata sidecar {meta.name}")
        continue
    meta_data = json.loads(meta.read_text())
    for key in ("git_sha", "image_digest", "dataset_hash", "dataset_release", "policy_hash"):
        if key not in meta_data:
            errors.append(f"{meta}: missing field {key}")

if errors:
    print("load result contract validation failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("load result contract validation passed")
