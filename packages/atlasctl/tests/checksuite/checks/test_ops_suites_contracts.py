from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]


def test_ops_suites_contracts_check_passes() -> None:
    proc = subprocess.run(
        ['python3', 'packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_ops_suites_contracts.py'],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr


def test_ops_suites_membership_golden_snapshot() -> None:
    payload = json.loads((ROOT / 'configs/ops/suites.json').read_text(encoding='utf-8'))
    rows = [
        {
            'name': str(s['name']),
            'atlasctl_suite': str(s['atlasctl_suite']),
            'speed': str(s['speed']),
            'markers': sorted(str(x) for x in s.get('markers', [])),
            'actions': sorted(str(x) for x in s.get('actions', [])),
            'evidence_area': str(s['evidence_area']),
        }
        for s in payload.get('suites', [])
        if isinstance(s, dict)
    ]
    rows = sorted(rows, key=lambda r: r['name'])
    got = json.dumps(rows, indent=2, sort_keys=True) + '\n'
    golden = (ROOT / 'packages/atlasctl/tests/goldens/check/ops-suites-membership.json.golden').read_text(encoding='utf-8')
    assert got == golden
