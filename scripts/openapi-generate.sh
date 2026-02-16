#!/usr/bin/env sh
set -eu

mkdir -p openapi/v1
cargo run --quiet -p bijux-atlas-api --bin atlas-openapi -- --out openapi/v1/openapi.generated.json
