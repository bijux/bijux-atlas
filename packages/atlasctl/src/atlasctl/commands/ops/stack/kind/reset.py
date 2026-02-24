from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in (cur, *cur.parents):
        if (parent / ".git").exists() and (parent / "makefiles").is_dir() and (parent / "configs").is_dir():
            return parent
    raise RuntimeError("unable to resolve repository root")


def main() -> int:
    root = _repo_root()
    subprocess.run([sys.executable, str(root / "packages/atlasctl/src/atlasctl/commands/ops/stack/kind/down.py")], check=True, cwd=root)
    subprocess.run([sys.executable, str(root / "packages/atlasctl/src/atlasctl/commands/ops/stack/kind/up.py")], check=True, cwd=root)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
