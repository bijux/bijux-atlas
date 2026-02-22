#!/usr/bin/env bash
set -euo pipefail

./bin/atlasctl dev fmt
./bin/atlasctl dev lint
./bin/atlasctl dev test
