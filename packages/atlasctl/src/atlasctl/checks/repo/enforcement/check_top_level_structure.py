from __future__ import annotations

from pathlib import Path

INTENDED = {
    "cli",
    "commands",
    "checks",
    "core",
    "contracts",
    "registry",
    "suite",
    "reporting",
    "policies",
    "internal",
}
TRANSITION_ALLOW = {"docs", "__pycache__"}


def check_top_level_structure(repo_root: Path) -> tuple[int, list[str]]:
    root = repo_root / "packages/atlasctl/src/atlasctl"
    if not root.exists():
        return 1, ["atlasctl source root missing"]
    dirs = sorted(p.name for p in root.iterdir() if p.is_dir())
    errors: list[str] = []
    unknown = [name for name in dirs if name not in INTENDED and name not in TRANSITION_ALLOW]
    if unknown:
        errors.append(f"unexpected top-level atlasctl packages: {', '.join(unknown)}")
    counted = [name for name in dirs if name not in TRANSITION_ALLOW]
    if len(counted) > 10:
        errors.append("top-level package budget exceeded (>10) excluding transition allowlist")
    missing = sorted(name for name in INTENDED if name not in dirs)
    if missing:
        errors.append(f"missing intended top-level atlasctl packages: {', '.join(missing)}")
    return (0 if not errors else 1), errors
