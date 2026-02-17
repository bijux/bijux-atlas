#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import json
import os
from pathlib import Path

report = Path("artifacts/ops/k8s/flake-report.json")
if not report.exists():
    print("flake report missing; skipping")
    raise SystemExit(0)

payload = json.loads(report.read_text())
count = int(payload.get("flake_count", 0))
if count == 0:
    print("no flakes detected")
    raise SystemExit(0)

print(f"flake detected: {count}")
for f in payload.get("flakes", []):
    print(f"- {f.get('script')} owner={f.get('owner')} attempts={f.get('attempts')}")

issue_path = Path("artifacts/ops/k8s/flake-issue.md")
issue_path.parent.mkdir(parents=True, exist_ok=True)
body = ["# K8s E2E Flake Detected", "", f"Count: {count}", "", "## Flakes"]
for f in payload.get("flakes", []):
    body.append(f"- `{f.get('script')}` owner={f.get('owner')} attempts={f.get('attempts')}")
body.append("\nAction: quarantine with TTL in `ops/e2e/k8s/tests/manifest.json`.")
issue_path.write_text("\n".join(body) + "\n")

# hard fail in CI to force explicit quarantine workflow.
if os.environ.get("CI", "").lower() in {"1", "true", "yes"}:
    raise SystemExit(1)
print("flake policy warning (non-CI)")
