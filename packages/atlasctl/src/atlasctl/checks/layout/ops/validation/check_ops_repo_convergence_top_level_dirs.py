from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
OPS = ROOT / "ops"

EXCLUDE = {
    "_artifacts",
    "_evidence",
    "_generated",
    "_generated.example",
    "_meta",
    "vendor",
    "manifests",
    "registry",  # legacy internal registry area; enforce on new top-level dirs going forward
    "docs",
    "env",
    "inventory",
    "schema",
    "observe",
    "helm",
    "kind",
    "report",
    "obs",
    "datasets",
    "e2e",
    "k8s",
    "load",
    "stack",
    "fixtures",
}


def main() -> int:
    errs: list[str] = []
    for d in sorted(p for p in OPS.iterdir() if p.is_dir()):
        if d.name.startswith("_") or d.name in EXCLUDE:
            continue
        for req in ("CONTRACT.md", "INDEX.md", "OWNER.md"):
            if not (d / req).exists():
                errs.append(f"{d.relative_to(ROOT).as_posix()}: missing {req}")
    if errs:
        print("ops repo convergence top-level dirs check failed:", file=sys.stderr)
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    print("ops repo convergence top-level dirs check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
