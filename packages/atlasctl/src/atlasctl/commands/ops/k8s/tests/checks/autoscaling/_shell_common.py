from __future__ import annotations

import subprocess
import textwrap
from pathlib import Path


def repo_root_from(path: Path) -> Path:
    cur = path.resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def run_k8s_test_shell(body: str, self_path: Path) -> int:
    root = repo_root_from(self_path)
    script = textwrap.dedent(
        f"""\
        set -euo pipefail
        source "{root}/packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/assets/k8s_test_common.sh"
        {body}
        """
    )
    return subprocess.run(["bash", "-lc", script], cwd=str(root), check=False).returncode
