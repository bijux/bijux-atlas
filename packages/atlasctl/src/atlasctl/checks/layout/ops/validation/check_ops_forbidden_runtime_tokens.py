from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
BASE = ROOT / 'packages' / 'atlasctl' / 'src' / 'atlasctl' / 'commands' / 'ops'
FORBIDDEN = ('ops/run/', '"bash"', "'bash'", ' -lc', '"-lc"', "'-lc'")


def _should_scan(path: Path) -> bool:
    rel = path.relative_to(BASE).as_posix()
    if '/internal/' in rel and '/migrate_' in rel:
        return False
    if '/tests/' in rel or rel.endswith('/tests.py'):
        return False
    if rel.endswith('.py') is False:
        return False
    return True


def main() -> int:
    errors: list[str] = []
    for path in sorted(BASE.rglob('*.py')):
        if not _should_scan(path):
            continue
        rel = path.relative_to(ROOT).as_posix()
        for lineno, line in enumerate(path.read_text(encoding='utf-8', errors='ignore').splitlines(), start=1):
            for token in FORBIDDEN:
                if token in line:
                    errors.append(f'{rel}:{lineno}: forbidden token `{token.strip()}` in ops command implementation')
    if errors:
        for err in errors:
            print(err, file=sys.stderr)
        return 1
    print('ops command implementations contain no forbidden legacy shell tokens')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
