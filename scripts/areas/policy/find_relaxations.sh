#!/usr/bin/env sh
# owner: platform
# purpose: scan repository code surfaces for policy-relaxation markers and emit JSON findings.
# stability: public
# called-by: make policy-audit, make ci-policy-relaxations
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
OUT="${1:-$ROOT/artifacts/policy/relaxations-grep.json}"
TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT

scan() {
  pattern_id="$1"
  regex="$2"
  requires_exception="$3"
  severity="$4"

  rg -n --no-heading -S "$regex" \
    "$ROOT/crates" "$ROOT/scripts" "$ROOT/xtask" "$ROOT/makefiles" "$ROOT/.github/workflows" "$ROOT/Makefile" \
    -g '*.rs' -g '*.sh' -g '*.py' -g '*.mk' -g '*.yml' -g '*.yaml' -g 'Makefile' \
    -g '!**/target/**' -g '!**/artifacts/**' \
    | while IFS= read -r line; do
      file="$(printf '%s' "$line" | cut -d: -f1)"
      ln="$(printf '%s' "$line" | cut -d: -f2)"
      text="$(printf '%s' "$line" | cut -d: -f3-)"
      exc="$(printf '%s' "$text" | sed -n 's/.*\(ATLAS-EXC-[0-9][0-9][0-9][0-9]\).*/\1/p')"
      printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$pattern_id" "$requires_exception" "$severity" "$file" "$ln" "$exc" >>"$TMP"
    done
}

scan "allowlist_token" "allowlist" "false" "info"
scan "skip_token" "\\bskip\\b" "false" "info"
scan "bypass_token" "bypass" "false" "warning"
scan "cfg_test_token" "cfg\\(test\\)" "false" "info"
scan "todo_relax_token" "TODO relax" "true" "error"
scan "unsafe_token" "\\bunsafe\\b" "false" "warning"
scan "unwrap_token" "unwrap\\(" "false" "warning"
scan "temporary_token" "temporary" "true" "warning"
scan "compat_token" "compat" "false" "info"
scan "legacy_token" "legacy" "false" "info"
scan "ignore_token" "\\bignore\\b" "false" "info"

mkdir -p "$(dirname "$OUT")"

python3 - "$TMP" "$OUT" <<'PY'
import json
import sys
from pathlib import Path

tmp = Path(sys.argv[1])
out = Path(sys.argv[2])
findings = []
skip_files = {
    str((Path.cwd() / "scripts/areas/policy/find_relaxations.sh").resolve()),
    str((Path.cwd() / "scripts/areas/public/policy-audit.py").resolve()),
}
for row in tmp.read_text().splitlines():
    pattern_id, req, severity, file, line, exc = row.split("\t")
    if str(Path(file).resolve()) in skip_files:
        continue
    findings.append(
        {
            "source": "grep",
            "pattern_id": pattern_id,
            "requires_exception": req == "true",
            "severity": severity,
            "file": file,
            "line": int(line),
            "exception_id": exc or None,
        }
    )
findings.sort(key=lambda x: (x["file"], x["line"], x["pattern_id"]))
payload = {"schema_version": 1, "findings": findings}
out.write_text(json.dumps(payload, indent=2) + "\n")
print(out)
PY
