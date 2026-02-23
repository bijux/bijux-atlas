#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
SRC = ROOT / 'packages/atlasctl/src/atlasctl'
FORBID = ('registry/ops_tasks_catalog.json', 'registry/suites_catalog.json', 'registry/checks_catalog.json')
ALLOW = {
    'packages/atlasctl/src/atlasctl/checks/layout/architecture/check_registry_reads_centralized.py',
    'packages/atlasctl/src/atlasctl/registry/readers.py',
    'packages/atlasctl/src/atlasctl/registry/catalogs.py',
    'packages/atlasctl/src/atlasctl/registry/suites.py',
    'packages/atlasctl/src/atlasctl/commands/registry/command.py',
    'packages/atlasctl/src/atlasctl/checks/registry/ssot.py',
    'packages/atlasctl/src/atlasctl/checks/domains/internal/checks/check_registry_integrity.py',
    'packages/atlasctl/src/atlasctl/checks/domains/internal/checks/__init__.py',
    'packages/atlasctl/src/atlasctl/checks/domains/docs/integrity.py',
    'packages/atlasctl/src/atlasctl/checks/repo/native/modules/repo_checks_make_and_layout.py',
    'packages/atlasctl/src/atlasctl/commands/internal/refactor_check_ids.py',
}


def main() -> int:
    errs: list[str] = []
    for path in SRC.rglob('*.py'):
        rel = path.relative_to(ROOT).as_posix()
        if rel in ALLOW:
            continue
        text = path.read_text(encoding='utf-8', errors='ignore')
        for token in FORBID:
            if token in text:
                errs.append(f'{rel}: direct registry JSON path read detected (`{token}`); use atlasctl.registry.readers')
    if errs:
        print('\n'.join(sorted(errs)))
        return 1
    print('registry read paths centralized')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
