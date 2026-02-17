#!/usr/bin/env sh
set -eu
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
VALUES="$ROOT/charts/bijux-atlas/values.yaml"
CONTRACT="$ROOT/docs/k8s/values-schema.md"

for key in image replicaCount service resources podSecurityContext securityContext cache rateLimits concurrency server sequenceRateLimits catalog store networkPolicy serviceMonitor hpa pdb priorityClassName terminationGracePeriodSeconds nodeLocalSsdProfile rollout catalogPublishJob datasetWarmupJob extraEnv nodeSelector tolerations affinity; do
  grep -q "^$key:" "$VALUES" || { echo "missing top-level values key: $key" >&2; exit 1; }
  grep -q "\`$key\`" "$CONTRACT" || { echo "contract missing key: $key" >&2; exit 1; }
done

echo "values contract gate passed"
