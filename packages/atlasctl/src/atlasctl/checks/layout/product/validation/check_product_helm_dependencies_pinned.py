from __future__ import annotations

import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[8]


def _is_floating(version: str) -> bool:
    tokens = [">", "<", "*", "~", "^", " x", "X"]
    return any(t in version for t in tokens) or not version.strip()


def main() -> int:
    errs: list[str] = []
    for chart in sorted(ROOT.glob("ops/**/Chart.yaml")):
        rel = chart.relative_to(ROOT).as_posix()
        payload = yaml.safe_load(chart.read_text(encoding="utf-8")) or {}
        deps = payload.get("dependencies", [])
        if not isinstance(deps, list):
            continue
        for dep in deps:
            if not isinstance(dep, dict):
                continue
            name = str(dep.get("name", "?"))
            version = str(dep.get("version", ""))
            if _is_floating(version):
                errs.append(f"{rel}: dependency {name} has non-pinned version '{version}'")
    if errs:
        print("product helm dependency pinning check failed:", file=sys.stderr)
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    print("product helm dependency pinning check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
