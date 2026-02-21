from __future__ import annotations

from pathlib import Path

MAX_MODULE_LOC = 400
ALLOWLIST_PATH = Path("configs/layout/module-size-allowlist.txt")


def _load_allowlist(repo_root: Path) -> set[str]:
    path = repo_root / ALLOWLIST_PATH
    if not path.exists():
        return set()
    out: set[str] = set()
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        out.add(line)
    return out


def check_module_size(repo_root: Path) -> tuple[int, list[str]]:
    allowlist = _load_allowlist(repo_root)
    offenders: list[str] = []
    root = repo_root / "packages/atlasctl/src/atlasctl"
    for py in sorted(root.rglob("*.py")):
        rel = py.relative_to(repo_root).as_posix()
        if rel in allowlist:
            continue
        loc = sum(1 for _ in py.open("r", encoding="utf-8"))
        if loc > MAX_MODULE_LOC:
            offenders.append(f"{rel}: {loc} LOC > {MAX_MODULE_LOC}")
    if offenders:
        return 1, offenders
    return 0, []
