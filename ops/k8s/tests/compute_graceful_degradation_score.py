#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

TARGET_GROUPS = {"resilience", "availability", "load", "admission-control", "rate-limit"}


def main() -> int:
    import argparse

    p = argparse.ArgumentParser(description="Compute graceful degradation score from k8s test report")
    p.add_argument("--json", required=True)
    p.add_argument("--out", required=True)
    args = p.parse_args()

    payload = json.loads(Path(args.json).read_text(encoding="utf-8"))
    candidates = []
    passed = 0
    failed = []
    for r in payload.get("results", []):
        groups = set(r.get("groups", []))
        if not groups.intersection(TARGET_GROUPS):
            continue
        candidates.append(r.get("script"))
        if r.get("status") == "passed":
            passed += 1
        elif r.get("status") == "failed":
            failed.append(r.get("script"))

    total = len(candidates)
    score = round((passed / total) * 100.0, 2) if total else 0.0
    out_payload = {
        "schema_version": 1,
        "run_id": payload.get("run_id", "unknown"),
        "suite_id": payload.get("suite_id", "unknown"),
        "total_considered": total,
        "passed": passed,
        "failed": len(failed),
        "score_percent": score,
        "status": "pass" if failed == [] and total > 0 else "fail",
        "failed_tests": failed,
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(out_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
