#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
from __future__ import annotations

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[3]
CARGO = ROOT / 'Cargo.toml'
OUT = ROOT / 'docs' / 'development' / 'crates-map.md'

PURPOSE_HINTS = {
    'bijux-atlas-core': 'deterministic primitives, canonicalization, error types',
    'bijux-atlas-model': 'domain and artifact data types',
    'bijux-atlas-policies': 'runtime policy schema and validation',
    'bijux-atlas-store': 'artifact backends and integrity boundaries',
    'bijux-atlas-ingest': 'deterministic ingest pipeline to artifacts',
    'bijux-atlas-query': 'query planning, limits, and pagination',
    'bijux-atlas-api': 'wire contracts and request/response schemas',
    'bijux-atlas-server': 'runtime orchestration and effectful serving',
    'bijux-atlas-cli': 'plugin CLI and operational commands',
}


def main() -> int:
    text = CARGO.read_text(encoding='utf-8')
    m = re.search(r'members\s*=\s*\[(.*?)\]', text, re.S)
    if not m:
        raise SystemExit('workspace members not found in Cargo.toml')
    raw = m.group(1)
    crates = []
    for item in re.findall(r'"([^"]+)"', raw):
        if item.startswith('crates/'):
            crates.append(Path(item).name)
    crates = sorted(set(crates))

    lines = [
        '# Crates Map',
        '',
        '- Owner: `docs-governance`',
        '',
        '## What',
        '',
        'Generated map of workspace crates and primary purpose.',
        '',
        '## Why',
        '',
        'Provides a stable navigation index for crate responsibilities.',
        '',
        '## Scope',
        '',
        'Covers workspace crates from `Cargo.toml` members under `crates/`.',
        '',
        '## Non-goals',
        '',
        'Does not replace crate-level architecture and API docs.',
        '',
        '## Contracts',
    ]
    for c in crates:
        hint = PURPOSE_HINTS.get(c, 'crate responsibility documented in crate docs')
        lines.append(f'- `{c}`: {hint}.')

    lines.extend([
        '',
        '## Failure modes',
        '',
        'Stale maps can hide ownership drift and boundary violations.',
        '',
        '## How to verify',
        '',
        '```bash',
        '$ python3 scripts/areas/docs/generate_crates_map.py',
        '$ ./scripts/areas/docs/check_crate_docs_contract.sh',
        '```',
        '',
        'Expected output: crates map is regenerated and crate docs contract passes.',
        '',
        '## See also',
        '',
        '- [Crate Layout Contract](../architecture/crate-layout-contract.md)',
        '- [Crate Boundary Graph](../architecture/crate-boundary-dependency-graph.md)',
        '- [Terms Glossary](../_style/terms-glossary.md)',
        ''
    ])

    OUT.write_text('\n'.join(lines), encoding='utf-8')
    print(f'generated {OUT}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())