#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"

ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "ops-warm-stage"
ops_version_guard kind kubectl

mode="warmup"
if [ "${1:-}" = "--mode" ]; then
  mode="${2:-warmup}"
fi

dataset_profile="${ATLAS_DATASET_PROFILE:?ATLAS_DATASET_PROFILE is required by configs/ops/env.schema.json}"
if [ "${PROFILE:-kind}" = "ci" ]; then
  dataset_profile="ci"
fi

case "$dataset_profile" in
  ci)
    # CI profile is fixture-first and network-free for dataset resolution.
    export ATLAS_ALLOW_PRIVATE_STORE_HOSTS=0
    export ATLAS_E2E_ENABLE_OTEL="${ATLAS_E2E_ENABLE_OTEL}"
    ;;
  developer)
    # Developer profile prefers cache-first behavior and reports fetch plan.
    ./bin/atlasctl ops cache --report text status --plan || true
    ;;
  *)
    echo "invalid ATLAS_DATASET_PROFILE=${dataset_profile} (expected ci|developer)" >&2
    exit 2
    ;;
esac

profile="${PROFILE:-kind}"
guard_profile="$profile"
case "$guard_profile" in
  ci|developer) guard_profile="kind" ;;
esac
if ! ops_context_guard "$guard_profile"; then
  if [ "$guard_profile" = "kind" ]; then
    echo "ops-warm-stage: context missing; bootstrapping stack with reuse" >&2
    ./ops/run/stack-up.sh --reuse --profile "$guard_profile"
  fi
fi

case "$mode" in
  warmup) exec ./ops/e2e/scripts/warmup.sh ;;
  datasets) exec ./ops/e2e/runner/warm_datasets.sh ;;
  top) exec ./ops/e2e/runner/warm_top.sh ;;
  shards) exec ./ops/e2e/runner/warm_shards.sh ;;
  *)
    echo "invalid --mode=${mode} (expected warmup|datasets|top|shards)" >&2
    exit 2
    ;;
esac
