#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-evidence-bundle"
ops_version_guard tar
run_id="${RUN_ID:-$(cat artifacts/evidence/latest-run-id.txt 2>/dev/null || true)}"
[ -n "$run_id" ] || { echo "missing RUN_ID and no artifacts/evidence/latest-run-id.txt" >&2; exit 2; }
src="artifacts/evidence/make/$run_id"
[ -d "$src" ] || { echo "missing evidence dir: $src" >&2; exit 2; }
out_dir="artifacts/ops/evidence-bundles"
mkdir -p "$out_dir"
out="$out_dir/$run_id.tar.zst"
tar --zstd -cf "$out" "$src"
echo "$out"
