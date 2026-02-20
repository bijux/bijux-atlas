#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
cd "$ROOT"

bad=0
banned='(mid-load|phase|step|task|stage)'

while IFS= read -r path; do
  name="$(basename "$path")"
  if [[ "$name" =~ $banned ]]; then
    echo "forbidden temporal/process token in path: $path" >&2
    bad=1
  fi
done < <(find ops -mindepth 1 \( -type d -o -type f \) \
  ! -path 'ops/_artifacts*' \
  ! -path 'ops/_generated*' \
  ! -path 'ops/stack/minio*' \
  -print)

while IFS= read -r path; do
  name="$(basename "$path")"
  if [[ "$name" == *minio* ]]; then
    if [[ "$path" == ops/stack/* ]]; then
      continue
    fi
    echo "durable names must use 'store' (minio allowed only in ops/stack/minio): $path" >&2
    bad=1
  fi
done < <(find ops -type f \( -name '*.sh' -o -name '*.json' -o -name '*.yaml' -o -name '*.yml' \) \
  ! -path 'ops/stack/minio*' \
  ! -path 'ops/_artifacts*' \
  ! -path 'ops/_generated*' \
  -print)

while IFS= read -r path; do
  name="$(basename "$path")"
  case "$name" in
    *.sh)
      if [[ ! "$name" =~ ^[a-z0-9]+(-[a-z0-9]+)*\.sh$ ]]; then
        echo "public shell script must use kebab-case: $path" >&2
        bad=1
      fi
      ;;
  esac
done < <(find ops/run scripts/areas/public/contracts -type f -name '*.sh' -print 2>/dev/null)

python3 ./scripts/areas/layout/check_no_mixed_script_name_variants.py || bad=1

exit "$bad"
