#!/usr/bin/env python3
# Purpose: generate k8s test surface docs from manifest and suites SSOT.
# Inputs: ops/k8s/tests/manifest.json and ops/k8s/tests/suites.json.
# Outputs: ops/k8s/tests/INDEX.md and docs/_generated/k8s-test-surface.md.
from __future__ import annotations

import json
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "ops/k8s/tests/manifest.json"
SUITES = ROOT / "ops/k8s/tests/suites.json"
OUT_INDEX = ROOT / "ops/k8s/tests/INDEX.md"
OUT_DOC = ROOT / "docs/_generated/k8s-test-surface.md"


def main() -> int:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    suites = json.loads(SUITES.read_text(encoding="utf-8"))
    tests = sorted(manifest.get("tests", []), key=lambda x: x["script"])

    by_group: dict[str, list[str]] = defaultdict(list)
    for t in tests:
        for g in sorted(t.get("groups", [])):
            by_group[g].append(t["script"])

    lines = [
        "# K8s Tests Index",
        "",
        "Generated from `ops/k8s/tests/manifest.json`.",
        "",
        "## Groups",
    ]
    for g in sorted(by_group):
        lines.append(f"- `{g}` ({len(by_group[g])})")
    lines.extend(["", "## Tests"])
    for t in tests:
        lines.append(
            f"- `{t['script']}` groups={','.join(t.get('groups', []))} owner={t.get('owner','unknown')}"
        )
    OUT_INDEX.write_text("\n".join(lines) + "\n", encoding="utf-8")

    s_map = {s["id"]: sorted(s.get("groups", [])) for s in suites.get("suites", [])}
    doc = [
        "# K8s Test Surface",
        "",
        "Generated from `ops/k8s/tests/manifest.json` and `ops/k8s/tests/suites.json`.",
        "",
        "## Suites",
    ]
    for sid in sorted(s_map):
        doc.append(f"- `{sid}` groups={','.join(s_map[sid]) if s_map[sid] else '*'}")
    doc.extend(["", "## Group -> Tests"])
    for g in sorted(by_group):
        doc.append(f"### `{g}`")
        for s in sorted(by_group[g]):
            doc.append(f"- `{s}`")
        doc.append("")
    OUT_DOC.write_text("\n".join(doc).rstrip() + "\n", encoding="utf-8")

    print(f"generated {OUT_INDEX.relative_to(ROOT)} and {OUT_DOC.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
