#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
SUITES = ROOT / 'ops/obs/suites/suites.json'
TESTS_DIR = ROOT / 'ops/obs/tests'


def _parse(argv: list[str]) -> tuple[str, list[str]]:
    suite = 'full'
    extra: list[str] = []
    i = 0
    while i < len(argv):
        a = argv[i]
        if a == '--suite':
            if i + 1 >= len(argv):
                raise SystemExit('--suite requires a value')
            suite = argv[i + 1]
            i += 2
            continue
        if not a.startswith('-') and suite == 'full':
            suite = a
            i += 1
            continue
        extra.append(a)
        i += 1
    return suite, extra


def _suite_tests(suite_id: str) -> list[str]:
    data = json.loads(SUITES.read_text(encoding='utf-8'))
    for suite in data.get('suites', []):
        if suite.get('id') == suite_id:
            return [str(t) for t in suite.get('tests', []) if str(t).strip()]
    raise SystemExit(f'unknown suite id: {suite_id}')


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    suite, extra = _parse(args)
    tests = _suite_tests(suite)
    for t in tests:
        if t.endswith(".py") or t.startswith("packages/"):
            target = ROOT / t
            proc = subprocess.run(["python3", str(target), *extra], cwd=ROOT)
        else:
            target = TESTS_DIR / t
            proc = subprocess.run(["bash", str(target), *extra], cwd=ROOT)
        if proc.returncode != 0:
            return proc.returncode
    print(f'obs suite passed: {suite}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
