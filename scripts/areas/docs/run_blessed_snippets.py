#!/usr/bin/env python3
# Purpose: execute blessed docs snippets with network-safety guardrails.
# Inputs: artifacts/docs-snippets/manifest.json and snippets
# Outputs: exit code and execution summary
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "artifacts" / "docs-snippets" / "manifest.json"
NETWORK_TOKENS = ("curl ", "wget ", "http://", "https://", "nc ")


def main() -> int:
    if not MANIFEST.exists():
        print("snippet runner: manifest not found; run extract_code_blocks.py first")
        return 1

    data = json.loads(MANIFEST.read_text(encoding="utf-8"))
    snippets = data.get("snippets", [])
    failures = []

    for item in snippets:
        script = ROOT / item["path"]
        body = script.read_text(encoding="utf-8")
        if not item.get("allow_network", False):
            lowered = body.lower()
            if any(token in lowered for token in NETWORK_TOKENS):
                failures.append(f"{item['path']}: network command found without # allow-network")
                continue
        proc = subprocess.run(["sh", str(script)], cwd=ROOT, capture_output=True, text=True)
        if proc.returncode != 0:
            failures.append(f"{item['path']}: exit={proc.returncode}\n{proc.stderr.strip()}")

    if failures:
        print("snippet execution failed:")
        for fail in failures:
            print(f"- {fail}")
        return 1

    print(f"snippet execution passed ({len(snippets)} snippet(s))")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
