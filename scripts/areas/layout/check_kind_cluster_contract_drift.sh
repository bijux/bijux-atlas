#!/usr/bin/env bash
# Purpose: require ops contract bump marker when kind cluster profile changes.
# Inputs: ops/stack/kind/cluster*.yaml and ops/CONTRACT.md hash marker.
# Outputs: non-zero on drift.
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)"
marker="$(sed -n 's/^kind-cluster-contract-hash: `\([a-f0-9]*\)`$/\1/p' "$ROOT/ops/CONTRACT.md" | head -n1)"
if [ -z "$marker" ]; then
  echo "missing kind-cluster-contract-hash marker in ops/CONTRACT.md" >&2
  exit 1
fi
calc="$(cat "$ROOT"/ops/stack/kind/cluster*.yaml | shasum -a 256 | awk '{print $1}')"
if [ "$marker" != "$calc" ]; then
  echo "kind cluster drift detected; update ops/CONTRACT.md marker to bump contract" >&2
  echo "expected: $calc" >&2
  echo "found:    $marker" >&2
  exit 1
fi
echo "kind cluster contract drift check passed"
