#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm
rendered="$(mktemp)"
rendered_defaults="$(mktemp)"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" > "$rendered"
helm template "$RELEASE" "$CHART" -n "$NS" > "$rendered_defaults"

# Deployment must define readiness/liveness probes and resources (requests+limits).
grep -q "readinessProbe:" "$rendered"
grep -q "livenessProbe:" "$rendered"
grep -q "resources:" "$rendered"
grep -q "requests:" "$rendered"
grep -q "limits:" "$rendered"

# PDB is required by chart contract (defaults must include it).
grep -q "kind: PodDisruptionBudget" "$rendered_defaults"

rm -f "$rendered" "$rendered_defaults"
echo "chart required fields contract passed"
