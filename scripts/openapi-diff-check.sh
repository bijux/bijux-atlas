#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -euo pipefail

./scripts/openapi-generate.sh
if ! diff -u openapi/v1/openapi.snapshot.json openapi/v1/openapi.generated.json; then
  echo "OpenAPI drift detected. Regenerate snapshot intentionally when API contract changes." >&2
  exit 1
fi