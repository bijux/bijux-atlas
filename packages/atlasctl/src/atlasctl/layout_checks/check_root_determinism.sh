#!/usr/bin/env bash
# Purpose: assert `make root` generated inventories are deterministic across two runs.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
cd "$ROOT"

run_a="det-a-$(date -u +%Y%m%dT%H%M%SZ)-$$"
run_b="det-b-$(date -u +%Y%m%dT%H%M%SZ)-$$"
out_a="artifacts/isolate/root-determinism/$run_a"
out_b="artifacts/isolate/root-determinism/$run_b"
mkdir -p "$out_a" "$out_b"

RUN_ID="$run_a" make -s root >/dev/null
cp docs/_generated/repo-surface.md "$out_a/repo-surface.md"
cp docs/_generated/naming-inventory.md "$out_a/naming-inventory.md"
cp docs/development/make-targets.md "$out_a/make-targets.md"

RUN_ID="$run_b" make -s root >/dev/null
cp docs/_generated/repo-surface.md "$out_b/repo-surface.md"
cp docs/_generated/naming-inventory.md "$out_b/naming-inventory.md"
cp docs/development/make-targets.md "$out_b/make-targets.md"

for f in repo-surface.md naming-inventory.md make-targets.md; do
  diff -u "$out_a/$f" "$out_b/$f" >/dev/null
  echo "deterministic: $f"
done

echo "root determinism check passed"
