from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
SSOT = ROOT / 'configs' / 'ops' / 'suites.json'
OPS_PARSER = ROOT / 'packages' / 'atlasctl' / 'src' / 'atlasctl' / 'commands' / 'ops' / 'runtime_modules' / 'ops_runtime_parser.py'
SUITES_CATALOG = ROOT / 'packages' / 'atlasctl' / 'src' / 'atlasctl' / 'registry' / 'suites_catalog.json'


def _public_ops_actions() -> set[str]:
    text = OPS_PARSER.read_text(encoding='utf-8', errors='ignore')
    actions: set[str] = set()
    mappings = {
        'stack': 'ops_stack_cmd',
        'k8s': 'ops_k8s_cmd',
        'obs': 'ops_obs_cmd',
        'load': 'ops_load_cmd',
        'e2e': 'ops_e2e_cmd',
        'datasets': 'ops_datasets_cmd',
        'deploy': 'ops_deploy_cmd',
    }
    for area, token in mappings.items():
        marker = f"dest=\"{token}\""
        if marker not in text:
            continue
        # conservative parser scan: collect sub.add_parser("...") within area block by simple regex-like pass
        anchor = f"{area} = ops_sub.add_parser(\"{area}\""
        start = text.find(anchor)
        if start < 0:
            continue
        end = text.find("\n\n", start)
        block = text[start:end if end > start else len(text)]
        for line in block.splitlines():
            if 'add_parser(' not in line:
                continue
            q = '"'
            try:
                name = line.split('add_parser(')[1].split(q)[1]
            except Exception:
                continue
            if name in {area}:  # top-level parser itself
                continue
            actions.add(f"{area}.{name}")
    # add simple top-level lane aliases intentionally considered public ops actions
    actions |= {'smoke', 'up', 'down', 'restart', 'doctor', 'prereqs', 'warm'}
    return actions


def _load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding='utf-8'))


def main() -> int:
    payload = _load_json(SSOT)
    catalog = _load_json(SUITES_CATALOG)
    errs: list[str] = []
    if int(payload.get('schema_version', 0) or 0) != 1:
        errs.append('configs/ops/suites.json schema_version must be 1')
    rows = payload.get('suites', [])
    if not isinstance(rows, list):
        errs.append('configs/ops/suites.json `suites` must be a list')
        rows = []
    catalog_names = {str(row.get('name')) for row in catalog.get('suites', []) if isinstance(row, dict)}
    seen_names: set[str] = set()
    covered_actions: set[str] = set()
    for idx, row in enumerate(rows):
        if not isinstance(row, dict):
            errs.append(f'suites[{idx}] must be object')
            continue
        name = str(row.get('name', '')).strip()
        atlasctl_suite = str(row.get('atlasctl_suite', '')).strip()
        speed = str(row.get('speed', '')).strip()
        markers = row.get('markers', [])
        actions = row.get('actions', [])
        evidence_area = str(row.get('evidence_area', '')).strip()
        if not name:
            errs.append(f'suites[{idx}].name is required')
            continue
        if name in seen_names:
            errs.append(f'duplicate ops suite name: {name}')
        seen_names.add(name)
        if atlasctl_suite not in catalog_names:
            errs.append(f'{name}: atlasctl_suite `{atlasctl_suite}` not found in suites catalog')
        if speed not in {'fast', 'slow'}:
            errs.append(f'{name}: speed must be fast|slow')
        if not isinstance(markers, list) or not all(isinstance(m, str) and m for m in markers):
            errs.append(f'{name}: markers must be non-empty string list')
        if speed == 'slow':
            if not str(row.get('slow_reason', '')).strip():
                errs.append(f'{name}: slow suite must declare slow_reason')
            if not str(row.get('reduction_plan', '')).strip():
                errs.append(f'{name}: slow suite must declare reduction_plan')
            if 'slow' not in markers:
                errs.append(f'{name}: slow suite markers must include `slow`')
        if speed == 'fast' and 'slow' in markers:
            errs.append(f'{name}: fast suite must not include `slow` marker')
        if not evidence_area:
            errs.append(f'{name}: evidence_area is required')
        else:
            expected_by_suite = {
                'ops.smoke': 'smoke',
                'ops.stack': 'stack',
                'ops.k8s': 'k8s',
                'ops.obs': 'obs',
                'ops.load.smoke': 'load-suite',
                'ops.load.regression': 'load-suite',
                'ops.e2e.smoke': 'e2e',
                'ops.e2e.realdata': 'e2e',
                'ops.datasets': 'datasets',
            }
            exp = expected_by_suite.get(name)
            if exp and evidence_area != exp:
                errs.append(f'{name}: evidence_area must be `{exp}` (got `{evidence_area}`)')
        if not isinstance(actions, list) or not all(isinstance(a, str) and a for a in actions):
            errs.append(f'{name}: actions must be non-empty string list')
        else:
            covered_actions.update(str(a) for a in actions)
    required_suite_names = {
        'ops.smoke', 'ops.stack', 'ops.k8s', 'ops.obs', 'ops.load.smoke', 'ops.load.regression', 'ops.e2e.smoke', 'ops.e2e.realdata'
    }
    missing_suites = sorted(required_suite_names - seen_names)
    if missing_suites:
        errs.append(f'missing required ops suites in configs/ops/suites.json: {missing_suites}')
    public_actions = _public_ops_actions()
    uncovered = sorted(a for a in public_actions if a not in covered_actions)
    if uncovered:
        errs.append(f'ops public actions missing suite membership: {uncovered}')
    if errs:
        print('ops suites contracts failed:', file=sys.stderr)
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    print('ops suites contracts passed')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
