#!/usr/bin/env bash
set -euo pipefail

./bin/atlasctl run ./ops/run/product_chart_verify.sh
./bin/atlasctl contracts generate --generators chart-schema
./bin/atlasctl contracts check --checks chart-values
