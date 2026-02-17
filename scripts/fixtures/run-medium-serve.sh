#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

cargo run -p bijux-atlas-cli --bin bijux-atlas -- atlas smoke \
  --root artifacts/medium-output \
  --dataset 110/homo_sapiens/GRCh38