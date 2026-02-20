#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
if [ ! -d "$root/bin" ]; then
  exit 0
fi

bad=0
while IFS= read -r -d '' p; do
  rel="${p#"$root"/}"
  echo "forbidden symlink in root bin/: $rel" >&2
  bad=1
done < <(find "$root/bin" -mindepth 1 -maxdepth 1 -type l -print0)

exit "$bad"
