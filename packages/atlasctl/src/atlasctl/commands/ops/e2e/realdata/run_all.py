from __future__ import annotations

import os
import sys

from pathlib import Path

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[6]))

from atlasctl.commands.ops.e2e.realdata._common import atlasctl, env_root


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    root = env_root()
    dataset_source = os.environ.get("REALDATA_SOURCE", "ops/datasets/real-datasets.json")
    if not (root / dataset_source).is_file():
        print(f"missing declared dataset source: {dataset_source}", file=sys.stderr)
        return 2
    atlasctl("ops", "e2e", "run", "--suite", "realdata", *args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
