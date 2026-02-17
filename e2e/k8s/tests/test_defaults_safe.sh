#!/usr/bin/env sh
set -eu
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
VALUES="$ROOT/charts/bijux-atlas/values.yaml"

grep -q '^\s*enableDebugDatasets: false' "$VALUES"
grep -q '^\s*cachedOnlyMode: false' "$VALUES"
grep -q '^\s*readOnlyFsMode: false' "$VALUES"
grep -q '^\s*requestTimeoutMs: 5000' "$VALUES"
grep -q '^\s*maxBodyBytes: 16384' "$VALUES"

echo "defaults-safe gate passed"
