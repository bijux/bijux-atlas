#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

root = Path('.').resolve()
docs = root / 'docs'
mkdocs = (root / 'mkdocs.yml').read_text(encoding='utf-8')

nav_refs = set(re.findall(r':\s+([A-Za-z0-9_./\-]+\.md)\s*$', mkdocs, re.MULTILINE))
index_refs = set()
index_dirs = set()
for idx in docs.rglob('INDEX.md'):
    index_dirs.add(str(idx.parent.relative_to(docs)))
    txt = idx.read_text(encoding='utf-8')
    for link in re.findall(r'\[[^\]]+\]\(([^)]+\.md)(?:#[^)]+)?\)', txt):
        p = (idx.parent / link).resolve()
        if p.exists() and docs in p.parents:
            index_refs.add(str(p.relative_to(docs)))

allow_prefixes = ('_generated/',)
errors = []
for md in docs.rglob('*.md'):
    rel = str(md.relative_to(docs))
    if rel.endswith('INDEX.md'):
        continue
    if any(rel.startswith(p) for p in allow_prefixes):
        continue
    parent = str(md.parent.relative_to(docs))
    if rel not in nav_refs and rel not in index_refs and parent not in index_dirs:
        errors.append(rel)

if errors:
    print('orphan docs detected:', file=sys.stderr)
    for e in sorted(errors):
        print(f'- {e}', file=sys.stderr)
    raise SystemExit(1)

print('no orphan docs check passed')
