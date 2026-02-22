from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[6]))

from atlasctl.commands.ops.e2e.realdata._common import atlasctl, env_root, py


def _assert_diff_rows(url: str) -> None:
    body = subprocess.check_output(["curl", "-fsS", url], text=True)
    payload = json.loads(body)
    assert payload.get("diff", {}).get("rows") is not None


def main() -> int:
    root = env_root()
    env = os.environ.copy()
    env.setdefault("ATLAS_REALDATA_ROOT", str(root / "artifacts/real-datasets"))
    base_url = env.get("ATLAS_E2E_BASE_URL", "http://127.0.0.1:18080")

    py("packages/atlasctl/src/atlasctl/commands/ops/datasets/fixtures/fetch_real_datasets.py", env=env)
    py("packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/cleanup_store.py", env=env)
    py("packages/atlasctl/src/atlasctl/commands/ops/datasets/publish_by_name.py", "real110", env=env)
    py("packages/atlasctl/src/atlasctl/commands/ops/datasets/publish_by_name.py", "real111", env=env)

    diff_out = Path(env.get("ATLAS_E2E_DIFF_OUT", str(root / "artifacts/ops/release-diff/110_to_111")))
    store_root = Path(env.get("ATLAS_E2E_STORE_ROOT", str(root / "artifacts/e2e-store")))
    diff_out.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "bijux-atlas-cli",
            "--bin",
            "bijux-atlas",
            "--",
            "atlas",
            "diff",
            "build",
            "--root",
            str(store_root),
            "--from-release",
            "110",
            "--to-release",
            "111",
            "--species",
            "homo_sapiens",
            "--assembly",
            "GRCh38",
            "--out-dir",
            str(diff_out),
        ],
        check=True,
        cwd=root,
    )
    assert (diff_out / "diff.json").is_file()
    assert (diff_out / "diff.summary.json").is_file()

    atlasctl("ops", "deploy", "--report", "text", "apply", env=env)

    _assert_diff_rows(
        f"{base_url}/v1/diff/genes?from_release=110&to_release=111&species=homo_sapiens&assembly=GRCh38&limit=50"
    )
    _assert_diff_rows(
        f"{base_url}/v1/diff/region?from_release=110&to_release=111&species=homo_sapiens&assembly=GRCh38&region=chrA:1-80&limit=50"
    )

    print("realdata two-release diff scenario passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
