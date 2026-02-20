#!/usr/bin/env bash
# owner: platform
# purpose: diff policy contracts between two refs to detect policy drift explicitly.
# stability: public
# called-by: make policy-drift-diff
set -euo pipefail

from_ref="${1:-HEAD~1}"
to_ref="${2:-HEAD}"

paths=(
  "configs/policy/policy.json"
  "configs/policy/policy.schema.json"
  "configs/policy/policy-relaxations.json"
  "configs/policy/policy-enforcement-coverage.json"
  "docs/contracts/POLICY_SCHEMA.json"
)

for p in "${paths[@]}"; do
  echo "### ${p}: ${from_ref}..${to_ref}"
  git diff -- "${from_ref}" "${to_ref}" -- "${p}" || true
done
