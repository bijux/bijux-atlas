#!/usr/bin/env sh
# owner: contracts
# purpose: detect OpenAPI drift against committed contract snapshot.
# stability: public
# called-by: make openapi-drift, make ops-openapi-validate
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -euo pipefail

./scripts/areas/internal/openapi-generate.sh
if ! diff -u configs/openapi/v1/openapi.snapshot.json configs/openapi/v1/openapi.generated.json; then
  echo "OpenAPI drift detected. Regenerate snapshot intentionally when API contract changes." >&2
  exit 1
fi
