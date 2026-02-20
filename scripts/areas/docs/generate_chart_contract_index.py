#!/usr/bin/env python3
# Purpose: generate docs/_generated/contracts/chart-contract-index.md from k8s chart contract tests.
# Inputs: ops/k8s/tests/manifest.json.
# Outputs: deterministic markdown doc.
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "ops" / "k8s" / "tests" / "manifest.json"
OUT = ROOT / "docs" / "_generated" / "contracts" / "chart-contract-index.md"


def main() -> int:
    doc = json.loads(MANIFEST.read_text(encoding="utf-8"))
    tests = []
    for t in doc.get("tests", []):
        groups = set(t.get("groups", []))
        if "chart-contract" not in groups:
            continue
        script = t["script"]
        if not script.startswith("checks/"):
            continue
        tests.append(
            {
                "script": script,
                "owner": t.get("owner", "unknown"),
                "failure": ", ".join(t.get("expected_failure_modes", [])) or "n/a",
                "timeout": t.get("timeout_seconds", "n/a"),
            }
        )
    tests.sort(key=lambda x: x["script"])

    lines = [
        "# Chart Contract Index",
        "",
        "Generated from `ops/k8s/tests/manifest.json` entries tagged with `chart-contract`.",
        "",
        "| Contract Test | Owner | Timeout (s) | Failure Modes |",
        "| --- | --- | ---: | --- |",
    ]
    for t in tests:
        lines.append(
            f"| `{t['script']}` | `{t['owner']}` | {t['timeout']} | `{t['failure']}` |"
        )
    lines.extend(
        [
            "",
            "## Regenerate",
            "",
            "```bash",
            "python3 scripts/areas/docs/generate_chart_contract_index.py",
            "```",
            "",
        ]
    )
    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"generated {OUT.relative_to(ROOT)} ({len(tests)} contracts)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
