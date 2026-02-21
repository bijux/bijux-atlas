from __future__ import annotations

from pathlib import Path

_ALLOWED = {
    "__init__.py",
    "command.py",
    "docs_command.py",
    "ops_command.py",
    "doctor.py",
    "fs.py",
    "subprocess.py",
    "logging.py",
    "structured_log.py",
    "domain_cmd.py",
}


def check_legacy_package_quarantine(repo_root: Path) -> tuple[int, list[str]]:
    legacy_root = repo_root / "packages/atlasctl/src/atlasctl/legacy"
    if not legacy_root.exists():
        return 0, []

    offenders: list[str] = []
    for path in sorted(legacy_root.glob("*.py")):
        if path.name not in _ALLOWED:
            offenders.append(path.relative_to(repo_root).as_posix())

    if offenders:
        return 1, [
            "legacy package is quarantined; only approved shim modules are allowed",
            *offenders,
        ]
    return 0, []
