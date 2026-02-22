#!/usr/bin/env bash
set -euo pipefail

./bin/atlasctl docs naming-inventory --report text
./bin/atlasctl docs legacy-terms-check --report text
./bin/atlasctl docs observability-docs-checklist --report text
./bin/atlasctl docs no-orphan-docs-check --report text
./bin/atlasctl docs script-locations-check --report text
./bin/atlasctl docs runbook-map-registration-check --report text
./bin/atlasctl docs contract-doc-pairs-check --report text
./bin/atlasctl run ./packages/atlasctl/src/atlasctl/load/contracts/validate_suite_manifest.py
./bin/atlasctl docs index-pages-check --report text
