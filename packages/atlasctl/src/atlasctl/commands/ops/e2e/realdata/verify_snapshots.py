from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[6]))

from atlasctl.commands.ops.e2e.realdata._common import env_root
from atlasctl.commands.ops.e2e.realdata.generate_snapshots import main as generate_snapshots_main


def main(argv: list[str] | None = None) -> int:
    _ = argv
    root = env_root()
    baseline = root / "ops/e2e/realdata/snapshots/release110_snapshot.json"
    out = Path(
        os.environ.get(
            "ATLAS_REALDATA_SNAPSHOT_OUT",
            str(root / "artifacts/ops/e2e/realdata/release110_snapshot.generated.json"),
        )
    )

    generate_snapshots_main([str(out)])

    if os.environ.get("ATLAS_REALDATA_UPDATE_SNAPSHOT", "0") == "1":
        shutil.copy2(out, baseline)
        print(f"updated baseline snapshot: {baseline}")
        return 0

    if os.environ.get("ATLAS_REALDATA_ALLOW_BOOTSTRAP", "1") == "1":
        data = json.loads(baseline.read_text(encoding="utf-8"))
        if not data.get("entries"):
            shutil.copy2(out, baseline)
            print(f"bootstrapped baseline snapshot: {baseline}")
            return 0

    if subprocess.run(["diff", "-u", str(baseline), str(out)]).returncode != 0:
        print("realdata snapshot drift detected", file=sys.stderr)
        return 1

    print("realdata snapshots verified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
