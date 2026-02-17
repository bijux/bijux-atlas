#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
LATENCY_MS="${1:-1200}"
JITTER_MS="${2:-200}"
"$ROOT/faults/toxiproxy-latency.sh" "$LATENCY_MS" "$JITTER_MS"
echo "slow store mode enabled via toxiproxy latency"
