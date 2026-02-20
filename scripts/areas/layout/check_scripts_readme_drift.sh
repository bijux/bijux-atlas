#!/usr/bin/env bash
# Purpose: ensure scripts/README.md stays generated and current.
# Inputs: scripts tree and scripts/areas/gen/generate_scripts_readme.py.
# Outputs: exits non-zero on drift.
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
hash_file() {
  python3 - "$1" <<'PY'
from pathlib import Path
import hashlib
import sys
p = Path(sys.argv[1])
print(hashlib.sha256(p.read_bytes()).hexdigest() if p.exists() else "")
PY
}

before_readme="$(hash_file "$ROOT/scripts/README.md")"
before_index="$(hash_file "$ROOT/scripts/INDEX.md")"
python3 "$ROOT/scripts/areas/gen/generate_scripts_readme.py" >/dev/null
after_readme="$(hash_file "$ROOT/scripts/README.md")"
after_index="$(hash_file "$ROOT/scripts/INDEX.md")"
if [[ "$before_readme" != "$after_readme" || "$before_index" != "$after_index" ]]; then
  echo "scripts index drift detected; run: make scripts-index" >&2
  git -C "$ROOT" --no-pager diff -- scripts/README.md scripts/INDEX.md >&2 || true
  exit 1
fi
echo "scripts README drift check passed"
