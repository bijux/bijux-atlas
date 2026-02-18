#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

LOCK=ops/fixtures/medium/v1/manifest.lock
[ -f "$LOCK" ] || { echo "missing $LOCK" >&2; exit 1; }

url=$(awk -F= '/^url=/{print $2}' "$LOCK")
sha=$(awk -F= '/^sha256=/{print $2}' "$LOCK")
archive=$(awk -F= '/^archive=/{print $2}' "$LOCK")
extract_dir=$(awk -F= '/^extract_dir=/{print $2}' "$LOCK")

tmp="artifacts/fixtures"
mkdir -p "$tmp"
out="$tmp/$archive"

if [ -f "$url" ]; then
  cp "$url" "$out"
else
  curl -fsSL "$url" -o "$out"
fi

actual=$(shasum -a 256 "$out" | awk '{print $1}')
[ "$actual" = "$sha" ] || { echo "checksum mismatch: $actual != $sha" >&2; exit 1; }

mkdir -p "$extract_dir"
tar -xzf "$out" -C "$extract_dir"

echo "fetched medium fixture to $extract_dir"