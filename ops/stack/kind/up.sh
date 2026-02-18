#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
# shellcheck source=ops/_lib/common.sh
source "$ROOT/ops/_lib/common.sh"
ops_init_run_id
ops_ci_no_prompt_policy

cluster="${ATLAS_E2E_CLUSTER_NAME:-bijux-atlas-e2e}"
profile="${ATLAS_KIND_PROFILE:-normal}"
case "$profile" in
  small) cfg="$ROOT/ops/stack/kind/cluster-small.yaml" ;;
  normal) cfg="$ROOT/ops/stack/kind/cluster.yaml" ;;
  perf) cfg="$ROOT/ops/stack/kind/cluster-perf.yaml" ;;
  *) echo "unknown ATLAS_KIND_PROFILE=$profile (small|normal|perf)" >&2; exit 2 ;;
esac

if kind get clusters | grep -qx "$cluster"; then
  echo "kind cluster already exists: $cluster"
  exit 0
fi

if [ "${OPS_DRY_RUN:-0}" = "1" ]; then
  echo "DRY-RUN kind create cluster --name $cluster --config $cfg"
  exit 0
fi

kind create cluster --name "$cluster" --config "$cfg"
echo "kind cluster up: $cluster profile=$profile"
