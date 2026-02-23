from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
BASELINE = ROOT / "configs/ops/product-task-scripts-baseline.json"
OPS_DIR = ROOT / "ops"
NAME_RE = re.compile(r"(^|[_./-])(product|release|artifact|chart|docker)([_./-]|$)", re.IGNORECASE)


def _iter_product_task_scripts() -> list[str]:
    hits: list[str] = []
    if not OPS_DIR.exists():
        return hits
    for path in sorted(OPS_DIR.rglob('*')):
        if not path.is_file() or path.suffix not in {'.sh', '.py'}:
            continue
        rel = path.relative_to(ROOT).as_posix()
        if '/fixtures/' in rel:
            continue
        if NAME_RE.search(rel):
            hits.append(rel)
    return hits


def main() -> int:
    baseline = json.loads(BASELINE.read_text(encoding='utf-8')) if BASELINE.exists() else {"max_count": 0}
    max_count = int(baseline.get('max_count', 0))
    files = _iter_product_task_scripts()
    errors: list[str] = []
    if len(files) > max_count:
        errors.append(f'product-task scripts under ops exceed baseline ({len(files)} > {max_count})')
    if files:
        errors.extend(files)
    if errors:
        print('\n'.join(errors))
        return 1
    print(f'ops product-task script inventory: count={len(files)} max={max_count}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
