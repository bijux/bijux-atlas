#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
OPS_ROOT = ROOT / 'packages/atlasctl/src/atlasctl/commands/ops'
AREAS = ('stack','k8s','obs','load','e2e','datasets','contracts','ports','artifacts','scenario')
MAX_LOC_BY_AREA = {
    "scenario": 60,  # thin validation wrapper; temporary until runtime extraction
    "contracts": 120,  # transitional command+dispatcher
    "ports": 120,  # legacy subcommands pending runtime extraction
    "artifacts": 120,  # legacy subcommands pending runtime extraction
}
ALLOW_BUSINESS_LOGIC_AREAS = {"scenario", "contracts", "ports", "artifacts"}


def main() -> int:
    errs: list[str] = []
    for area in AREAS:
        d = OPS_ROOT / area
        if not d.exists():
            errs.append(f'missing ops area module dir: {d.relative_to(ROOT).as_posix()}')
            continue
        if not (d / 'command.py').exists():
            errs.append(f'{d.relative_to(ROOT).as_posix()}: missing command.py')
        # command.py must stay thin
        cp = d / 'command.py'
        if cp.exists():
            txt = cp.read_text(encoding='utf-8', errors='ignore')
            if area not in ALLOW_BUSINESS_LOGIC_AREAS and any(
                tok in txt for tok in ('subprocess', 'run_command(', 'json.loads(', 'yaml.', 'Path(')
            ):
                errs.append(f'{cp.relative_to(ROOT).as_posix()}: business logic detected in command.py')
            max_loc = MAX_LOC_BY_AREA.get(area, 40)
            if len(txt.splitlines()) > max_loc:
                errs.append(
                    f'{cp.relative_to(ROOT).as_posix()}: command.py too large (>{max_loc} LOC); move logic to runtime.py'
                )
    if errs:
        print('\n'.join(errs))
        return 1
    print('ops command group public entrypoints OK')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
