#!/usr/bin/env python3
from __future__ import annotations

import json
import pathlib
import re
import time

ROOT = pathlib.Path(__file__).resolve().parents[3]


def parse_help_commands(help_text: str) -> list[str]:
    commands: list[str] = []
    for line in help_text.splitlines():
        if re.match(r"^\s{2}[a-z0-9-]+\s{2,}", line):
            commands.append(line.split()[0])
    return commands


def benchmark_help_parse(iterations: int = 5000) -> dict[str, object]:
    sample = (ROOT / "ops/cli/perf/fixtures/help-root.snapshot.txt").read_text(encoding="utf-8")
    started = time.perf_counter()
    total = 0
    for _ in range(iterations):
        total += len(parse_help_commands(sample))
    elapsed = time.perf_counter() - started
    return {
        "name": "help_parse",
        "iterations": iterations,
        "commands_counted": total,
        "elapsed_ms": round(elapsed * 1000, 3),
        "ops_per_sec": round(iterations / elapsed, 2) if elapsed > 0 else None,
    }


def main() -> int:
    payload = {"benchmarks": [benchmark_help_parse()]}
    out = ROOT / "ops/cli/reports/benchmark-report.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
