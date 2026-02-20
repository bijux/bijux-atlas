#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
export ROOT

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  cat <<'EOF'
Usage: scripts/areas/check/no-duplicate-script-names.sh

Fails when dash/underscore duplicate script names exist.
EOF
  exit 0
fi

python3 - <<'PY'
from pathlib import Path
import sys
import os

root = Path(os.environ["ROOT"])
seen = {}
errors = []
for p in sorted((root / "scripts").rglob("*")):
    if not p.is_file() or p.suffix not in {".sh", ".py"}:
        continue
    name = p.stem
    canonical = name.replace("_", "-")
    rel = p.relative_to(root).as_posix()
    seen.setdefault(canonical, []).append(rel)

for canonical, paths in seen.items():
    stems = {Path(p).stem for p in paths}
    if len(stems) > 1:
        errors.append((canonical, sorted(paths)))

if errors:
    print("duplicate dash/underscore script names detected:", file=sys.stderr)
    for key, paths in errors:
        print(f"- {key}:", file=sys.stderr)
        for p in paths:
            print(f"  - {p}", file=sys.stderr)
    raise SystemExit(1)
print("no duplicate script names")
PY
