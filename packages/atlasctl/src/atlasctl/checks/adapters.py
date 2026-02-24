from __future__ import annotations

"""Compatibility adapter module for transitional check runtimes."""

import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

from atlasctl.checks.effects import CheckEffect


@dataclass(frozen=True)
class ProcResult:
    code: int
    stdout: str
    stderr: str
    argv: tuple[str, ...]
    cwd: str
    env_keys: tuple[str, ...]


class Proc:
    def __init__(self, *, repo_root: Path, env_allowlist: Iterable[str] = ()) -> None:
        self._repo_root = repo_root.resolve()
        self._env_allow = set(env_allowlist)

    def run(
        self,
        argv: list[str],
        *,
        cwd: Path | None = None,
        env: dict[str, str] | None = None,
        timeout: float | None = None,
    ) -> ProcResult:
        target_cwd = (cwd or self._repo_root).resolve()
        use_env = None if env is None else {k: v for k, v in env.items() if k in self._env_allow}
        proc = subprocess.run(
            argv,
            cwd=target_cwd,
            env=use_env,
            text=True,
            capture_output=True,
            check=False,
            timeout=timeout,
        )
        return ProcResult(
            code=int(proc.returncode),
            stdout=str(proc.stdout or ""),
            stderr=str(proc.stderr or ""),
            argv=tuple(str(v) for v in argv),
            cwd=target_cwd.as_posix(),
            env_keys=tuple(sorted((use_env or {}).keys())),
        )


class FS:
    def __init__(self, *, repo_root: Path, allowed_roots: Iterable[str]) -> None:
        self._repo_root = repo_root.resolve()
        self._roots = tuple((self._repo_root / rel).resolve() for rel in allowed_roots)

    def _assert_allowed(self, path: Path) -> Path:
        target = path if path.is_absolute() else (self._repo_root / path)
        resolved = target.resolve()
        if not any(resolved == root or root in resolved.parents for root in self._roots):
            raise PermissionError(f"write outside allowed roots: {resolved}")
        return resolved

    def write_text(self, path: Path, data: str, *, encoding: str = "utf-8") -> None:
        target = self._assert_allowed(path)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(data, encoding=encoding)

    def write_bytes(self, path: Path, data: bytes) -> None:
        target = self._assert_allowed(path)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)

    def read_text(self, path: Path, *, encoding: str = "utf-8") -> str:
        target = path if path.is_absolute() else (self._repo_root / path)
        return target.read_text(encoding=encoding)


class Network:
    def __init__(self, *, enabled: bool = False) -> None:
        self._enabled = enabled

    def ensure_enabled(self) -> None:
        if not self._enabled:
            raise PermissionError(f"network effect disabled by default; declare `{CheckEffect.NETWORK.value}` and enable explicitly")
