from __future__ import annotations

import json
from pathlib import Path

from tests.helpers import GOLDENS_MANIFEST, GOLDENS_ROOT


def test_golden_manifest_matches_filesystem() -> None:
    payload = json.loads(GOLDENS_MANIFEST.read_text(encoding='utf-8'))
    manifest = sorted((str(row['name']), str(row['path'])) for row in payload.get('entries', []) if isinstance(row, dict))

    actual: list[tuple[str, str]] = []
    for path in sorted(GOLDENS_ROOT.rglob('*')):
        if not path.is_file():
            continue
        rel = path.relative_to(GOLDENS_ROOT).as_posix()
        if rel == 'MANIFEST.json' or rel.startswith('__pycache__/'):
            continue
        actual.append((path.name, rel))
    actual = sorted(actual)

    assert manifest == actual


def test_golden_manifest_has_unique_names() -> None:
    payload = json.loads(GOLDENS_MANIFEST.read_text(encoding='utf-8'))
    names = [str(row['name']) for row in payload.get('entries', []) if isinstance(row, dict)]
    assert len(names) == len(set(names)), 'golden names must be unique in MANIFEST.json'
