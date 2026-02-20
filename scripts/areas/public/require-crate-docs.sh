#!/usr/bin/env sh
# owner: docs-governance
# purpose: verify each crate has required docs contract pages.
# stability: public
# called-by: make crate-structure
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu
./scripts/areas/docs/check_crate_docs_contract.sh
