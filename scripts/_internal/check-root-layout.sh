#!/usr/bin/env sh
# Purpose: compatibility wrapper for root layout checks.
# Inputs: repository root directories.
# Outputs: delegated check status.
set -eu
exec "$(dirname "$0")/layout/check_root_shape.sh" "$@"
