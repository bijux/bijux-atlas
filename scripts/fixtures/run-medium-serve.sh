#!/usr/bin/env sh
set -eu

cargo run -p bijux-atlas-cli --bin atlas-cli -- smoke \
  --root artifacts/medium-output \
  --dataset 110/homo_sapiens/GRCh38
