#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]


def main() -> int:
    obs_dir = ROOT / "packages/atlasctl/src/atlasctl/commands/ops/obs"
    observability_dir = ROOT / "packages/atlasctl/src/atlasctl/commands/ops/observability"
    errs: list[str] = []
    transitional_allowed_prefix = (obs_dir / "contracts").resolve()
    if obs_dir.exists():
        for path in sorted(obs_dir.rglob("*.py")):
            if path.name in {"command.py", "runtime.py"}:
                continue
            if path.resolve().is_relative_to(transitional_allowed_prefix):
                continue
            errs.append(
                f"{path.relative_to(ROOT).as_posix()}: `obs` is public facade only; place logic under commands/ops/observability/"
            )
    if not observability_dir.exists():
        errs.append("missing commands/ops/observability owner subsystem")
    if errs:
        print("\n".join(errs))
        return 1
    print("ops obs/observability ownership split OK (obs facade, observability owner)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
