from __future__ import annotations

import os
import sys
from pathlib import Path

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[6]))

from atlasctl.commands.ops.e2e.realdata._common import atlasctl, env_root, py


def main() -> int:
    root = env_root()
    env = os.environ.copy()
    env.setdefault("ATLAS_REALDATA_ROOT", str(root / "artifacts/real-datasets"))

    py("packages/atlasctl/src/atlasctl/commands/ops/datasets/fixtures/fetch_real_datasets.py", env=env)
    py("packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/cleanup_store.py", env=env)
    py("packages/atlasctl/src/atlasctl/commands/ops/datasets/publish_by_name.py", "real110", env=env)
    atlasctl("ops", "deploy", "--report", "text", "apply", env=env)
    py("packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/warmup.py", env=env)
    py("packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/smoke_queries.py", env=env)
    py("packages/atlasctl/src/atlasctl/commands/ops/e2e/realdata/verify_snapshots.py", env=env)

    print("realdata single-release scenario passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
