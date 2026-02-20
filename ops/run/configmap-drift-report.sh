#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "configmap-drift-report"
ops_version_guard helm

run_dir="artifacts/evidence/configmap-drift/${RUN_ID}"
mkdir -p "$run_dir"
profiles=(local perf offline)

for profile in "${profiles[@]}"; do
  values_file="$ROOT/ops/k8s/values/${profile}.yaml"
  helm template "atlas-${profile}" "$ROOT/ops/k8s/charts/bijux-atlas" -n "${ATLAS_NS:-atlas-e2e}" -f "$values_file" \
    | awk '
      $0 ~ /^kind: ConfigMap$/ {in_cm=1}
      in_cm {print}
      in_cm && $0 == "---" {in_cm=0}
    ' > "${run_dir}/configmap.${profile}.yaml"
done

prev_id="$(cat artifacts/evidence/configmap-drift/latest-run-id.txt 2>/dev/null || true)"
{
  echo "configmap drift report run_id=${RUN_ID}"
  if [ -n "$prev_id" ] && [ -d "artifacts/evidence/configmap-drift/${prev_id}" ]; then
    echo "previous_run_id=${prev_id}"
    for profile in "${profiles[@]}"; do
      prev="artifacts/evidence/configmap-drift/${prev_id}/configmap.${profile}.yaml"
      curr="${run_dir}/configmap.${profile}.yaml"
      if [ -f "$prev" ]; then
        diff -u "$prev" "$curr" > "${run_dir}/diff.${profile}.patch" || true
      fi
    done
  else
    echo "previous_run_id=<none>"
  fi
} > "${run_dir}/report.txt"

echo "$RUN_ID" > artifacts/evidence/configmap-drift/latest-run-id.txt
echo "configmap drift report written: ${run_dir}"
