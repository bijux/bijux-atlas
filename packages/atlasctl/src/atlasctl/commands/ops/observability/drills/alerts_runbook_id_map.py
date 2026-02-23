from __future__ import annotations
import re
from atlasctl.core.runtime.repo_root import find_repo_root
from pathlib import Path

def main() -> int:
    root = find_repo_root()
    text = (root/'ops/obs/alerts/atlas-alert-rules.yaml').read_text(encoding='utf-8')
    runbooks = re.findall(r'runbook:\s*"([^"]+)"', text)
    if not runbooks:
        raise SystemExit('no runbook mappings in alerts file')
    for rb in runbooks:
        p = root / rb
        if not p.exists():
            raise SystemExit(f'missing runbook for alert mapping: {rb}')
    print('alerts runbook id map passed')
    return 0
if __name__ == '__main__': raise SystemExit(main())
