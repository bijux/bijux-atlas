#!/usr/bin/env sh
# owner: platform
# purpose: validate CLI command surface against docs snapshots.
# stability: public
# called-by: make cli-command-surface
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

cargo test -p bijux-atlas-cli command_surface_ssot_matches_doc -- --exact
cargo test -p bijux-atlas-cli help_output_command_surface_matches_doc_exactly -- --exact
