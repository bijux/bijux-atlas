#!/usr/bin/env sh
set -eu

python3 - <<'PY'
import re
from pathlib import Path

root = Path('.')
exclude_parts = {'.git', 'artifacts', 'target', '.cargo'}
md_files = []
for p in root.rglob('*.md'):
    parts = set(p.parts)
    if parts & exclude_parts:
        continue
    md_files.append(p)

link_re = re.compile(r'\[[^\]]+\]\(([^)]+)\)')
errors = []
for md in md_files:
    text = md.read_text(encoding='utf-8')
    for target in link_re.findall(text):
        if target.startswith(('http://', 'https://', 'mailto:', '#')):
            continue
        rel = target.split('#', 1)[0]
        if not rel:
            continue
        path = (md.parent / rel).resolve()
        if not path.exists():
            errors.append(f"{md}: missing link target {target}")

if errors:
    print("\n".join(errors))
    raise SystemExit(1)
print(f"markdown links OK ({len(md_files)} files)")
PY
