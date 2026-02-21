from __future__ import annotations

import os
import shutil
from datetime import datetime, timezone
from pathlib import Path


def generate_isolate_tag(*, git_sha: str, prefix: str) -> str:
    ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    short = (git_sha or "nogit")[:8]
    return f"{prefix}-{ts}-{short}-{os.getppid()}"


def resolve_isolate_root(*, repo_root: Path, tag: str, env: dict[str, str] | None = None) -> Path:
    source = env or os.environ
    raw = source.get("ISO_ROOT", "")
    if raw:
        path = Path(raw)
        return path if path.is_absolute() else (repo_root / path)
    return (repo_root / "artifacts" / "isolate" / tag).resolve()


def build_isolate_env(
    *,
    repo_root: Path,
    git_sha: str,
    prefix: str,
    tag: str | None = None,
    base_env: dict[str, str] | None = None,
) -> dict[str, str]:
    env = dict(base_env or os.environ)
    iso_tag = tag or env.get("ISO_TAG", "") or generate_isolate_tag(git_sha=git_sha, prefix=prefix)
    iso_root = resolve_isolate_root(repo_root=repo_root, tag=iso_tag, env=env)
    env["ISO_TAG"] = iso_tag
    env["ISO_RUN_ID"] = env.get("ISO_RUN_ID", iso_tag)
    env["ISO_ROOT"] = str(iso_root)
    env["CARGO_TARGET_DIR"] = str(iso_root / "target")
    env["CARGO_HOME"] = str(iso_root / "cargo-home")
    env["TMPDIR"] = str(iso_root / "tmp")
    env["TMP"] = str(iso_root / "tmp")
    env["TEMP"] = str(iso_root / "tmp")
    env["TZ"] = "UTC"
    env["LC_ALL"] = "C"
    Path(env["CARGO_TARGET_DIR"]).mkdir(parents=True, exist_ok=True)
    Path(env["CARGO_HOME"]).mkdir(parents=True, exist_ok=True)
    Path(env["TMPDIR"]).mkdir(parents=True, exist_ok=True)
    return env


def require_isolate_env(env: dict[str, str] | None = None) -> tuple[bool, str]:
    source = env or os.environ
    required = ("ISO_TAG", "ISO_RUN_ID", "ISO_ROOT", "CARGO_TARGET_DIR", "CARGO_HOME", "TMPDIR", "TMP", "TEMP")
    for key in required:
        if not source.get(key):
            return False, f"missing env var: {key}"
    iso_root = Path(source["ISO_ROOT"]).resolve()
    if "artifacts/isolate" not in iso_root.as_posix():
        return False, f"ISO_ROOT must be under artifacts/isolate: {iso_root}"
    for key in ("CARGO_TARGET_DIR", "CARGO_HOME", "TMPDIR", "TMP", "TEMP"):
        path = Path(source[key]).resolve()
        if not str(path).startswith(str(iso_root) + "/"):
            return False, f"path not inside ISO_ROOT: {path}"
    return True, "OK"


def clean_isolate_roots(repo_root: Path, *, older_than_days: int = 14, keep_last: int = 20) -> list[str]:
    root = (repo_root / "artifacts" / "isolate").resolve()
    if not root.exists():
        return []
    cutoff = datetime.now(timezone.utc).timestamp() - (older_than_days * 86400)
    entries = sorted((p for p in root.iterdir() if p.is_dir()), key=lambda p: p.stat().st_mtime, reverse=True)
    keep = {p.resolve() for p in entries[:keep_last]}
    removed: list[str] = []
    for path in entries:
        if path.resolve() in keep:
            continue
        if path.stat().st_mtime <= cutoff:
            shutil.rmtree(path, ignore_errors=True)
            removed.append(str(path))
    return sorted(removed)

