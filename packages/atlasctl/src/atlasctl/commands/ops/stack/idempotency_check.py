#!/usr/bin/env python3
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _run(cmd: list[str], cwd: Path) -> None:
    p = subprocess.run(cmd, cwd=cwd, text=True, capture_output=True, check=False)
    if p.stdout:
        print(p.stdout, end='')
    if p.returncode != 0:
        if p.stderr:
            print(p.stderr, end='', file=sys.stderr)
        raise SystemExit(p.returncode)


def main() -> int:
    root = _repo_root()
    if os.environ.get("ATLASCTL_OPS_RUNTIME_CHECKS", "").strip() not in {"1", "true", "yes"}:
        print("stack idempotency check skipped (set ATLASCTL_OPS_RUNTIME_CHECKS=1 to run live)")
        return 0
    profile = os.environ.get('PROFILE', 'kind')
    _run(['./bin/atlasctl', 'ops', 'stack', 'up', '--report', 'text'], root)
    _run(['./bin/atlasctl', 'ops', 'stack', 'up', '--report', 'text'], root)
    _run(['./bin/atlasctl', 'ops', 'stack', 'down', '--report', 'text'], root)
    _run(['./bin/atlasctl', 'ops', 'stack', 'down', '--report', 'text'], root)
    print('stack up/down idempotency check passed')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
