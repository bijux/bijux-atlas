from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parents[7]
    subprocess.run([sys.executable, str(root / "packages/atlasctl/src/atlasctl/commands/ops/stack/kind/down.py")], check=True, cwd=root)
    subprocess.run([sys.executable, str(root / "packages/atlasctl/src/atlasctl/commands/ops/stack/kind/up.py")], check=True, cwd=root)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
