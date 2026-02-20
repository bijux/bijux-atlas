#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

python3 - <<'PY'
from __future__ import annotations

from pathlib import Path
import re
import subprocess

ROOT = Path('.')
CRATES = sorted([p for p in (ROOT / 'crates').iterdir() if p.is_dir()])
errors: list[str] = []

required_docs = {'INDEX.md', 'architecture.md', 'effects.md', 'public-api.md', 'testing.md'}
contracts_required = {'bijux-atlas-api', 'bijux-atlas-server', 'bijux-atlas-policies', 'bijux-atlas-store'}
failure_modes_required = {'bijux-atlas-server', 'bijux-atlas-store', 'bijux-atlas-ingest'}

required_sections = ['## Purpose', '## Invariants', '## Boundaries', '## Failure modes', '## How to test']

placeholder_pat = re.compile(r'\b(TODO|TBD|coming soon)\b', re.IGNORECASE)

# collect public API names from src/lib.rs
pub_pat = re.compile(r'^\s*pub\s+(?:struct|enum|trait|type)\s+([A-Z][A-Za-z0-9_]*)\b', re.MULTILINE)

for crate in CRATES:
    name = crate.name
    docs = crate / 'docs'
    readme = crate / 'README.md'
    if not docs.is_dir():
        errors.append(f'{crate}: missing docs directory')
        continue

    files = {p.name for p in docs.glob('*.md')}
    for req in required_docs:
        if req not in files:
            errors.append(f'{crate}/docs: missing {req}')

    if name in contracts_required and 'contracts.md' not in files:
        errors.append(f'{crate}/docs: missing contracts.md (required)')
    if name in failure_modes_required and 'failure-modes.md' not in files:
        errors.append(f'{crate}/docs: missing failure-modes.md (required)')

    # forbid legacy names
    for forbidden in ['HUMAN_MACHINE.md', 'PUBLIC_SURFACE_CHECKLIST.md', 'EFFECT_BOUNDARY_MAP.md', 'PUBLIC_API.md', 'ARCHITECTURE.md', 'EFFECTS.md']:
        if forbidden in files:
            errors.append(f'{crate}/docs: legacy filename forbidden: {forbidden}')

    # patterns.md optional but forbidden as empty stub
    if 'patterns.md' in files:
        text = (docs / 'patterns.md').read_text(encoding='utf-8')
        if len(text.strip()) < 120:
            errors.append(f'{crate}/docs/patterns.md: too small; remove or document real patterns')

    # major standardized docs: enforce depth/examples on docs that are meant to be executable guidance
    major = [docs / 'testing.md']
    if name in contracts_required:
        major.append(docs / 'contracts.md')
    if name in failure_modes_required:
        major.append(docs / 'failure-modes.md')

    for md in major:
        if not md.exists():
            continue
        txt = md.read_text(encoding='utf-8')
        if not re.search(r'^- Owner:\s*`[^`]+`\s*$', txt, re.MULTILINE):
            errors.append(f'{md}: missing owner header "- Owner: `...`"')
        for sec in required_sections:
            if sec not in txt:
                errors.append(f'{md}: missing section {sec}')
        if md.name == 'contracts.md' and '## Versioning' not in txt:
            errors.append(f'{md}: missing section ## Versioning')
        examples = txt.count('```') // 2
        if examples < 2:
            errors.append(f'{md}: requires at least 2 examples')
        if placeholder_pat.search(txt):
            errors.append(f'{md}: contains placeholder marker TODO/TBD/coming soon')

        # internal links stable and relative
        if re.search(r'\]\((?:https?://|file://|/)', txt):
            errors.append(f'{md}: contains non-relative internal link')

    # README rules
    if not readme.exists():
        errors.append(f'{crate}: missing README.md')
    else:
        rtxt = readme.read_text(encoding='utf-8')
        for sec in ['## Purpose', '## Public API', '## Boundaries', '## Effects', '## Telemetry', '## Tests', '## Benches', '## Docs index']:
            if sec not in rtxt:
                errors.append(f'{readme}: missing section {sec}')
        for req_link in ['docs/INDEX.md', 'docs/public-api.md']:
            if req_link not in rtxt:
                errors.append(f'{readme}: missing link {req_link}')
        docs_index_block = re.search(r'## Docs index\n([\s\S]*?)(?:\n## |\Z)', rtxt)
        if not docs_index_block:
            errors.append(f'{readme}: missing docs index block')
        else:
            links = re.findall(r'\[[^\]]+\]\([^\)]+\)', docs_index_block.group(1))
            if len(links) < 5:
                errors.append(f'{readme}: docs index must list at least 5 important docs')

    # INDEX link rules
    idx = docs / 'INDEX.md'
    if idx.exists():
        itxt = idx.read_text(encoding='utf-8')
        for req in ['public-api.md', 'effects.md', 'testing.md']:
            if req not in itxt:
                errors.append(f'{idx}: must link {req}')
        if '#how-to-extend' not in itxt and 'How to extend' not in itxt:
            errors.append(f'{idx}: must provide How to extend linkage')

    # public-api mention gate from src/lib.rs
    lib = crate / 'src' / 'lib.rs'
    if lib.exists() and (docs / 'public-api.md').exists():
        names = sorted(set(pub_pat.findall(lib.read_text(encoding='utf-8'))))
        ptxt = (docs / 'public-api.md').read_text(encoding='utf-8')
        if '../../../../docs/_style/stability-levels.md' not in ptxt:
            errors.append(f'{docs / "public-api.md"}: missing stability reference link to ../../../../docs/_style/stability-levels.md')
        for n in names:
            if n not in ptxt:
                errors.append(f'{docs / "public-api.md"}: missing mention of public type {n}')

if errors:
    print('crate docs contract check failed:')
    for e in errors:
        print(f'- {e}')
    raise SystemExit(1)

print('crate docs contract OK')
PY
