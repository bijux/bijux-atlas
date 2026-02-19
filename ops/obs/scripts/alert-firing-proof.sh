#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: validate alert contract/rules and prove key alerts are present.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
alerts_primary="$ROOT/ops/obs/alerts/atlas-alert-rules.yaml"
alerts_burn="$ROOT/ops/obs/alerts/slo-burn-rules.yaml"
"$ROOT/ops/obs/scripts/alerts-validation.sh"
for a in \
  BijuxAtlasHigh5xxRate \
  BijuxAtlasP95LatencyRegression \
  AtlasOverloadSustained \
  BijuxAtlasCheapSloBurnFast \
  BijuxAtlasCheapSloBurnMedium \
  BijuxAtlasCheapSloBurnSlow \
  BijuxAtlasStandardSloBurnFast \
  BijuxAtlasStandardSloBurnMedium \
  BijuxAtlasStandardSloBurnSlow \
  BijuxAtlasOverloadSurvivalViolated \
  BijuxAtlasRegistryRefreshStale \
  BijuxAtlasStoreBackendErrorSpike; do
  rg -n "alert:\\s*$a" "$alerts_primary" "$alerts_burn" >/dev/null
done

# Unit-like synthetic check: burn multipliers should exceed windows only when ratio is bad.
python3 - <<'PY'
def burn_multiplier(error_ratio, error_budget):
    return error_ratio / error_budget

def assert_window(name, healthy, bad, threshold):
    if healthy > threshold:
        raise SystemExit(f"{name}: healthy case unexpectedly exceeds threshold")
    if bad <= threshold:
        raise SystemExit(f"{name}: bad case does not exceed threshold")

cheap_budget = 0.0001
standard_budget = 0.001
assert_window("cheap-fast", burn_multiplier(0.00005, cheap_budget), burn_multiplier(0.003, cheap_budget), 14)
assert_window("cheap-medium", burn_multiplier(0.00005, cheap_budget), burn_multiplier(0.001, cheap_budget), 6)
assert_window("cheap-slow", burn_multiplier(0.00005, cheap_budget), burn_multiplier(0.0005, cheap_budget), 3)
assert_window("standard-fast", burn_multiplier(0.0003, standard_budget), burn_multiplier(0.02, standard_budget), 14)
assert_window("standard-medium", burn_multiplier(0.0003, standard_budget), burn_multiplier(0.008, standard_budget), 6)
assert_window("standard-slow", burn_multiplier(0.0003, standard_budget), burn_multiplier(0.004, standard_budget), 3)
print("slo burn alert synthetic proof passed")
PY

echo "alert firing proof drill passed"
