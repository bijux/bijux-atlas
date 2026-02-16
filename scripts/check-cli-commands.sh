#!/usr/bin/env sh
set -eu

cargo test -p bijux-atlas-cli command_surface_ssot_matches_doc -- --exact
cargo test -p bijux-atlas-cli help_output_command_surface_matches_doc_exactly -- --exact
