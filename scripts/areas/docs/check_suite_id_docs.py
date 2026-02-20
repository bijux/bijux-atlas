#!/usr/bin/env python3
# Purpose: enforce docs reference suite IDs instead of test script/scenario filenames.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
TARGETS = [
    ROOT / "docs/operations/k8s/k8s-test-contract.md",
    ROOT / "docs/operations/load/INDEX.md",
    ROOT / "docs/operations/load/k6.md",
    ROOT / "docs/operations/load/suites.md",
]

bad_patterns = [
    re.compile(r"\btest_[a-z0-9_]+\.sh\b"),
    re.compile(
        r"\b(?:mixed|spike|cold-start|stampede|store-outage-under-spike|pod-churn|cheap-only-survival|response-size-abuse|multi-release|sharded-fanout|diff-heavy|mixed-gene-sequence|soak-30m|redis-optional|catalog-federated|multi-dataset-hotset|large-dataset-simulation|load-under-rollout|load-under-rollback)\.json\b"
    ),
]

errors: list[str] = []
for p in TARGETS:
    rel = p.relative_to(ROOT).as_posix()
    text = p.read_text(encoding="utf-8", errors="ignore")
    for pat in bad_patterns:
        for m in pat.findall(text):
            errors.append(f"{rel}: reference suite ID instead of file `{m}`")

if errors:
    print("suite-id docs check failed", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("suite-id docs check passed")
