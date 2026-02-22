#!/usr/bin/env bash
set -euo pipefail

./bin/atlasctl docs durable-naming-check --report text
./bin/atlasctl docs duplicate-topics-check --report text
