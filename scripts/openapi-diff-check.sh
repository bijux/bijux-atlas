#!/usr/bin/env bash
set -euo pipefail

./scripts/openapi-generate.sh
if ! diff -u openapi/v1/openapi.snapshot.json openapi/v1/openapi.generated.json; then
  echo "OpenAPI drift detected. Regenerate snapshot intentionally when API contract changes." >&2
  exit 1
fi
