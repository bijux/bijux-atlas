from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path
from typing import Sequence


def repo_root() -> Path:
    return Path(__file__).resolve().parents[7]


def env_root() -> Path:
    return Path(os.environ.get("ATLAS_REPO_ROOT", str(repo_root())))


def py(tool_rel: str, *args: str, env: dict[str, str] | None = None) -> None:
    root = env_root()
    subprocess.run([sys.executable, str(root / tool_rel), *args], check=True, env=env)


def atlasctl(*args: str, env: dict[str, str] | None = None) -> None:
    root = env_root()
    subprocess.run([str(root / "bin/atlasctl"), *args], check=True, env=env, cwd=root)


def sh(cmd: Sequence[str], *, env: dict[str, str] | None = None, cwd: Path | None = None) -> None:
    subprocess.run(list(cmd), check=True, env=env, cwd=str(cwd) if cwd else None)
