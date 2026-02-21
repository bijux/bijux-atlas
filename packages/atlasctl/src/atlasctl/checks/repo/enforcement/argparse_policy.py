from __future__ import annotations

from pathlib import Path


_ALLOWED_EXACT = {
    "packages/atlasctl/src/atlasctl/cli/main.py",
    "packages/atlasctl/src/atlasctl/cli/parser.py",
}


def _is_allowed(rel: str) -> bool:
    if rel in _ALLOWED_EXACT:
        return True
    return rel.startswith("packages/atlasctl/src/atlasctl/commands/") and rel.endswith("/parser.py")


def check_argparse_policy(repo_root: Path) -> tuple[int, list[str]]:
    """Forbid direct ArgumentParser construction outside canonical parser modules."""
    src_root = repo_root / "packages/atlasctl/src/atlasctl"
    offenders: list[str] = []
    for path in sorted(src_root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        # Scope to CLI surface modules only.
        if not (
            rel.startswith("packages/atlasctl/src/atlasctl/cli/")
            or rel.startswith("packages/atlasctl/src/atlasctl/commands/")
            or rel.endswith("/command.py")
        ):
            continue
        if _is_allowed(rel):
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "argparse.ArgumentParser(" in text:
            offenders.append(rel)
    if offenders:
        return 1, [
            "direct argparse parser creation is restricted to cli/parser.py and commands/*/parser.py",
            *offenders,
        ]
    return 0, []
