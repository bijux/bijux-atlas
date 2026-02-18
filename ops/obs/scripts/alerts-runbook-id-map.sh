#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: prove alerts map to valid runbook ids/paths.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
python3 - <<'PY'
import pathlib,re
root=pathlib.Path('.').resolve()
text=(root/'ops/obs/alerts/atlas-alert-rules.yaml').read_text(encoding='utf-8')
runbooks=re.findall(r'runbook:\s*"([^"]+)"', text)
if not runbooks:
    raise SystemExit('no runbook mappings in alerts file')
for rb in runbooks:
    p=root/rb
    if not p.exists():
        raise SystemExit(f'missing runbook for alert mapping: {rb}')
print('alerts runbook id map passed')
PY
