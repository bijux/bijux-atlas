#!/usr/bin/env sh
# owner: operations
# purpose: collect a deterministic report bundle under artifacts/scripts.
# stability: public
# called-by: make ops-report
# Purpose: gather lightweight cluster/report diagnostics for workflows.
# Inputs: RUN_ID, OUT_DIR environment variables (optional).
# Outputs: artifacts/scripts/report_bundle/<run-id> bundle directory path.
set -eu

ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd)"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
OUT_DIR="${OUT_DIR:-$ROOT/artifacts/scripts/report_bundle/$RUN_ID}"

mkdir -p "$OUT_DIR"
SCRIPT_NAME="report_bundle" RUN_ID="$RUN_ID" "$ROOT/scripts/internal/env_dump.sh" >/dev/null
{
  echo "run_id=$RUN_ID"
  echo "generated_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "git_sha=$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"
} >"$OUT_DIR/metadata.txt"

kubectl get ns >/dev/null 2>&1 && kubectl get ns >"$OUT_DIR/namespaces.txt" 2>/dev/null || true
kubectl get pods -A >"$OUT_DIR/pods.txt" 2>/dev/null || true
SCRIPT_NAME="report_bundle" RUN_ID="$RUN_ID" "$ROOT/scripts/internal/exec.sh" sh -c "true" >/dev/null

printf '%s\n' "$OUT_DIR"
