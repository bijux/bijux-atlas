#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT_DIR"

VER="$(awk -F '"' '/^version = /{print $2; exit}' Cargo.toml)"
TAG="v${VER}"

TMP_FILE="$(mktemp)"
CURRENT_FILE="$(mktemp)"
trap 'rm -f "$TMP_FILE" "$CURRENT_FILE"' EXIT

cp docs/compatibility/umbrella-atlas-matrix.md "$CURRENT_FILE"

./scripts/release/update-compat-matrix.sh "$TAG"
cp docs/compatibility/umbrella-atlas-matrix.md "$TMP_FILE"
cp "$CURRENT_FILE" docs/compatibility/umbrella-atlas-matrix.md

if ! diff -u "$CURRENT_FILE" "$TMP_FILE"; then
  echo "compatibility matrix is out of date for workspace version ${VER}" >&2
  exit 1
fi
