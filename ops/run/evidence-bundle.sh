#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
run_id="${RUN_ID:-$(cat ops/_evidence/latest-run-id.txt 2>/dev/null || true)}"
[ -n "$run_id" ] || { echo "missing RUN_ID and no ops/_evidence/latest-run-id.txt" >&2; exit 2; }
src="ops/_evidence/make/$run_id"
[ -d "$src" ] || { echo "missing evidence dir: $src" >&2; exit 2; }
out_dir="artifacts/ops/evidence-bundles"
mkdir -p "$out_dir"
out="$out_dir/$run_id.tar.zst"
tar --zstd -cf "$out" "$src"
echo "$out"
