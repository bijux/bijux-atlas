#!/usr/bin/env sh
set -eu

cargo test -p bijux-atlas-cli command_surface_ssot_matches_doc -- --exact
