#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
VALUES="$ROOT/charts/bijux-atlas/values.yaml"

grep -q '^\s*enableDebugDatasets: false' "$VALUES"
grep -q '^\s*cachedOnlyMode: false' "$VALUES"
grep -q '^\s*readOnlyFsMode: false' "$VALUES"
grep -q '^\s*requestTimeoutMs: 5000' "$VALUES"
grep -q '^\s*maxBodyBytes: 16384' "$VALUES"
grep -q '^\s*enabled: true' "$VALUES" # networkPolicy default
grep -q '^\s*allowPrivilegeEscalation: false' "$VALUES"
grep -q '^\s*runAsNonRoot: true' "$VALUES"
grep -q '^\s*enabled: false' "$VALUES" # ingress default disabled

echo "defaults-safe gate passed"
