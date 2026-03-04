#!/usr/bin/env bash
set -euo pipefail

cargo run --locked -q -p bijux-dev-atlas -- perf validate --format json
