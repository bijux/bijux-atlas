#!/usr/bin/env python3
from __future__ import annotations
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
UP = ROOT / 'packages/atlasctl/src/atlasctl/commands/ops/stack/install.py'
DOWN = ROOT / 'packages/atlasctl/src/atlasctl/commands/ops/stack/uninstall.py'


def assert_order(file: Path, tokens: list[str]) -> list[str]:
    text = file.read_text(encoding='utf-8', errors='ignore') if file.exists() else ''
    errs=[]
    prev=-1
    lines=text.splitlines()
    for token in tokens:
        idx = next((i for i,l in enumerate(lines, start=1) if token in l), None)
        if idx is None:
            errs.append(f'stack order contract missing token in {file.name}: {token}')
            return errs
        if idx <= prev:
            errs.append(f'stack order contract violated in {file.name}: {token}')
            return errs
        prev=idx
    return errs


def main() -> int:
    errors=[]
    errors += assert_order(UP, ['stack/minio/minio.yaml','stack/prometheus/prometheus.yaml','stack/grafana/grafana.yaml'])
    errors += assert_order(DOWN, ['stack/toxiproxy/toxiproxy.yaml','stack/redis/redis.yaml','stack/otel/otel-collector.yaml','stack/grafana/grafana.yaml','stack/prometheus/prometheus.yaml','stack/minio/minio.yaml'])
    if errors:
        for e in errors:
            print(e, file=sys.stderr)
        return 1
    print('ops stack install/uninstall order contract passed')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
