#!/usr/bin/env python3
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONTRACT_FILES = [
    "docs/contracts/ERROR_CODES.json",
    "docs/contracts/METRICS.json",
    "docs/contracts/TRACE_SPANS.json",
    "docs/contracts/ENDPOINTS.json",
    "docs/contracts/CHART_VALUES.json",
]


def git(*args: str) -> str:
    proc = subprocess.run(
        ["git", *args],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or "git command failed")
    return proc.stdout.strip()


def previous_tag() -> str | None:
    try:
        tags = [
            t for t in git("tag", "--list", "--sort=-creatordate", "v*").splitlines() if t.strip()
        ]
    except RuntimeError:
        return None
    return tags[0] if tags else None


def read_at_ref(ref: str, rel_path: str) -> dict | None:
    proc = subprocess.run(
        ["git", "show", f"{ref}:{rel_path}"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        return None
    return json.loads(proc.stdout)


def main() -> int:
    base_ref = previous_tag()
    if base_ref is None:
        print("contracts breaking-change check skipped (no v* tag found)")
        return 0

    breaking = False
    for rel in CONTRACT_FILES:
        current = json.loads((ROOT / rel).read_text())
        previous = read_at_ref(base_ref, rel)
        if previous is None:
            continue
        if rel.endswith("ERROR_CODES.json"):
            removed = sorted(set(previous["codes"]) - set(current["codes"]))
            if removed:
                print(f"{rel}: removed error codes since {base_ref}: {removed}", file=sys.stderr)
                breaking = True
        elif rel.endswith("METRICS.json"):
            prev = {m["name"] for m in previous["metrics"]}
            cur = {m["name"] for m in current["metrics"]}
            removed = sorted(prev - cur)
            if removed:
                print(f"{rel}: removed metrics since {base_ref}: {removed}", file=sys.stderr)
                breaking = True
        elif rel.endswith("TRACE_SPANS.json"):
            prev = {s["name"] for s in previous["spans"]}
            cur = {s["name"] for s in current["spans"]}
            removed = sorted(prev - cur)
            if removed:
                print(f"{rel}: removed spans since {base_ref}: {removed}", file=sys.stderr)
                breaking = True
        elif rel.endswith("ENDPOINTS.json"):
            prev = {(e["method"], e["path"]) for e in previous["endpoints"]}
            cur = {(e["method"], e["path"]) for e in current["endpoints"]}
            removed = sorted(prev - cur)
            if removed:
                print(
                    f"{rel}: removed endpoints since {base_ref}: {removed}",
                    file=sys.stderr,
                )
                breaking = True
        elif rel.endswith("CHART_VALUES.json"):
            removed = sorted(
                set(previous["top_level_keys"]) - set(current["top_level_keys"])
            )
            if removed:
                print(
                    f"{rel}: removed chart keys since {base_ref}: {removed}",
                    file=sys.stderr,
                )
                breaking = True

    if breaking:
        print(
            "breaking contract changes detected; require explicit compatibility sign-off",
            file=sys.stderr,
        )
        return 1

    print(f"contracts compatibility check passed vs {base_ref}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
