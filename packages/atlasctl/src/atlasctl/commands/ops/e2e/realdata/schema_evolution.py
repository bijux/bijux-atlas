from __future__ import annotations

import sys
from pathlib import Path

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[6]))

from atlasctl.commands.ops.e2e.realdata._common import env_root, sh


def main() -> int:
    root = env_root()
    sh(["cargo", "test", "-p", "bijux-atlas-server", "--test", "schema_evolution_regression"], cwd=root)
    print("schema evolution regression passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
