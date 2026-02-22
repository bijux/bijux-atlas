from __future__ import annotations

import json
import os
import re
from dataclasses import dataclass
from datetime import date, datetime, timezone
from fnmatch import fnmatch
from pathlib import Path

from atlasctl.core.exec import run

@dataclass(frozen=True)
class PythonMigrationException:
    kind: str
    path_glob: str
    contains: str

def _load_python_migration_exceptions(repo_root: Path) -> list[PythonMigrationException]:
    payload = json.loads((repo_root / "configs/layout/python-migration-exceptions.json").read_text(encoding="utf-8"))
    items: list[PythonMigrationException] = []
    for row in payload.get("exceptions", []):
        items.append(
            PythonMigrationException(
                kind=str(row.get("kind", "")),
                path_glob=str(row.get("path_glob", "")),
                contains=str(row.get("contains", "")),
            )
        )
    return items

def _find_python_migration_exception(repo_root: Path, kind: str, rel_path: str, line: str) -> PythonMigrationException | None:
    for entry in _load_python_migration_exceptions(repo_root):
        if entry.kind != kind:
            continue
        if not fnmatch(rel_path, entry.path_glob):
            continue
        if entry.contains and entry.contains not in line:
            continue
        return entry
    return None

def check_duplicate_script_names(repo_root: Path) -> tuple[int, list[str]]:
    seen: dict[str, list[str]] = {}
    errors: list[str] = []
    for path in sorted((repo_root / "scripts").rglob("*")):
        if not path.is_file() or path.suffix not in {".sh", ".py"}:
            continue
        canonical = path.stem.replace("_", "-")
        rel = path.relative_to(repo_root).as_posix()
        seen.setdefault(canonical, []).append(rel)

    for canonical, paths in sorted(seen.items()):
        stems = {Path(p).stem for p in paths}
        if len(stems) > 1:
            errors.append(f"{canonical}: {', '.join(sorted(paths))}")
    return (0 if not errors else 1), errors

def check_script_help(repo_root: Path) -> tuple[int, list[str]]:
    targets = [
        repo_root / "bin/atlasctl",
    ]
    errors: list[str] = []
    for p in targets:
        if not p.exists():
            errors.append(f"missing help-gated script: {p.relative_to(repo_root)}")
            continue
        proc = run([str(p), "--help"], cwd=repo_root, text=True, capture_output=True)
        out = (proc.stdout or "") + (proc.stderr or "")
        if proc.returncode != 0:
            errors.append(f"{p.relative_to(repo_root)}: --help exited {proc.returncode}")
            continue
        low = out.lower()
        if "usage" not in low and "purpose" not in low and "contract" not in low:
            errors.append(f"{p.relative_to(repo_root)}: --help output missing usage/contract text")
    return (0 if not errors else 1), errors

def check_script_ownership(repo_root: Path) -> tuple[int, list[str]]:
    ownership_path = repo_root / "configs/meta/ownership.json"
    payload = json.loads(ownership_path.read_text(encoding="utf-8"))
    paths = set(payload.get("paths", {}).keys())
    errors: list[str] = []
    for p in sorted((repo_root / "scripts").rglob("*")):
        if not p.is_file():
            continue
        rel = p.relative_to(repo_root).as_posix()
        if rel.startswith("scripts/__pycache__"):
            continue
        matched = any(rel == path or rel.startswith(path + "/") for path in paths)
        if not matched:
            errors.append(rel)
    return (0 if not errors else 1), errors

def check_python_migration_exceptions_expiry(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / "configs" / "layout" / "python-migration-exceptions.json"
    payload = json.loads(path.read_text(encoding="utf-8"))
    today = date.today()
    errors: list[str] = []
    for row in payload.get("exceptions", []):
        expires_on = date.fromisoformat(str(row.get("expires_on", "")))
        if expires_on < today:
            errors.append(
                f"{row.get('id')} kind={row.get('kind')} owner={row.get('owner')} "
                f"expires_on={row.get('expires_on')} issue={row.get('issue')}"
            )
    return (0 if not errors else 1), errors

def check_bin_entrypoints(repo_root: Path) -> tuple[int, list[str]]:
    scripts_bin = repo_root / "scripts" / "bin"
    if not scripts_bin.exists():
        return 0, []
    files = sorted(p for p in scripts_bin.glob("*") if p.is_file())
    if len(files) > 15:
        return 1, [f"scripts/bin cap exceeded: {len(files)} > 15"]
    return 0, []

def check_python_lock(repo_root: Path) -> tuple[int, list[str]]:
    locks = [repo_root / "packages/atlasctl/requirements.lock.txt"]
    pat = re.compile(r"^[a-zA-Z0-9_.-]+==[a-zA-Z0-9_.-]+$")
    errors: list[str] = []
    for lock in locks:
        text = lock.read_text(encoding="utf-8")
        lines = [ln.strip() for ln in text.splitlines() if ln.strip() and not ln.strip().startswith("#")]
        invalid = [ln for ln in lines if not pat.match(ln)]
        for line in invalid:
            errors.append(f"{lock.relative_to(repo_root)}: {line}")
    return (0 if not errors else 1), errors

def check_scripts_lock_sync(repo_root: Path) -> tuple[int, list[str]]:
    cfg = json.loads((repo_root / "configs/scripts/python-tooling.json").read_text(encoding="utf-8"))
    errors: list[str] = []
    if cfg.get("toolchain") != "pip-tools":
        errors.append("python tooling SSOT must declare toolchain=pip-tools")
        return 1, errors
    lockfile = repo_root / str(cfg["lockfile"])
    if not lockfile.exists():
        return 1, [f"missing lockfile: {lockfile}"]
    pyproject = repo_root / "packages/atlasctl/pyproject.toml"
    text = pyproject.read_text(encoding="utf-8")
    match = re.search(r"\[project\.optional-dependencies\]\s*dev\s*=\s*\[(?P<body>.*?)\]", text, re.S)
    if not match:
        return 1, ["unable to parse [project.optional-dependencies].dev from pyproject.toml"]
    expected = sorted(re.findall(r'"([^"]+)"', match.group("body")))
    lines = [ln.strip() for ln in lockfile.read_text(encoding="utf-8").splitlines() if ln.strip() and not ln.startswith("#")]
    locked = sorted(lines)
    if expected != locked:
        return 1, [f"scripts lock drift: expected={expected} locked={locked}"]
    return 0, []

def check_no_adhoc_python(repo_root: Path) -> tuple[int, list[str]]:
    allowlist = repo_root / "configs/layout/python-legacy-allowlist.txt"
    allow = {
        line.strip()
        for line in allowlist.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.strip().startswith("#")
    }
    proc = run(["git", "ls-files", "*.py"], cwd=repo_root, text=True, capture_output=True)
    errors: list[str] = []
    for rel in sorted(ln.strip() for ln in proc.stdout.splitlines() if ln.strip()):
        if rel.startswith("packages/atlasctl/"):
            continue
        if "/tests/" in rel:
            continue
        if _find_python_migration_exception(repo_root, "python_scripts_path", rel, "") is not None:
            continue
        if rel in allow:
            continue
        errors.append(rel)
    return (0 if not errors else 1), errors

def check_no_direct_python_invocations(repo_root: Path) -> tuple[int, list[str]]:
    docs = repo_root / "docs"
    makefiles = repo_root / "makefiles"
    makefile = repo_root / "Makefile"
    direct_py_re = re.compile(r"\bpython3?\s+([^\s`]+\.py)\b")
    py_scripts_re = re.compile(r"\bpython3?\s+scripts/[^\s`]+\.py\b")
    allowed_make_re = re.compile(r"\bpython3?\s+-m\s+atlasctl(?:\b|$)")
    errors: list[str] = []

    def scan(path: Path, kind: str) -> None:
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for lineno, line in enumerate(text.splitlines(), start=1):
            if py_scripts_re.search(line):
                if _find_python_migration_exception(repo_root, "python_scripts_path", rel, line) is None:
                    errors.append(f"{rel}:{lineno}: direct `python scripts/*.py` invocation is forbidden")
            if kind == "docs":
                if direct_py_re.search(line):
                    if _find_python_migration_exception(repo_root, "docs_direct_python", rel, line) is None:
                        errors.append(
                            f"{rel}:{lineno}: docs must reference `atlasctl`, not direct python execution"
                        )
            if kind == "makefiles":
                if direct_py_re.search(line) and not allowed_make_re.search(line):
                    if _find_python_migration_exception(repo_root, "makefiles_direct_python", rel, line) is None:
                        errors.append(
                            f"{rel}:{lineno}: makefiles must use `atlasctl` or `python -m atlasctl...`"
                        )

    for path in docs.rglob("*.md"):
        if "docs/_generated/" in path.as_posix():
            continue
        scan(path, "docs")
    for path in makefiles.glob("*.mk"):
        scan(path, "makefiles")
    scan(makefile, "makefiles")
    return (0 if not errors else 1), errors

def check_no_direct_bash_invocations(repo_root: Path) -> tuple[int, list[str]]:
    docs = repo_root / "docs"
    makefiles = repo_root / "makefiles"
    makefile = repo_root / "Makefile"
    bash_scripts_re = re.compile(r"\bbash\s+([^\s`]*scripts/[^\s`]+)\b")
    errors: list[str] = []

    def scan(path: Path, kind: str) -> None:
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for lineno, line in enumerate(text.splitlines(), start=1):
            if not bash_scripts_re.search(line):
                continue
            exc_kind = "docs_direct_bash" if kind == "docs" else "makefiles_direct_bash"
            if _find_python_migration_exception(repo_root, exc_kind, rel, line) is None:
                errors.append(f"{rel}:{lineno}: direct `bash ...scripts/...` invocation is forbidden")

    for path in docs.rglob("*.md"):
        if "docs/_generated/" in path.as_posix():
            continue
        scan(path, "docs")
    for path in makefiles.glob("*.mk"):
        scan(path, "makefiles")
    scan(makefile, "makefiles")
    return (0 if not errors else 1), errors

def check_docs_no_ops_generated_run_paths(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    pattern = re.compile(r"ops/_generated/atlas-ops-[A-Za-z0-9._/-]*")
    for path in sorted((repo_root / "docs").rglob("*.md")):
        text = path.read_text(encoding="utf-8", errors="ignore")
        rel = path.relative_to(repo_root).as_posix()
        for lineno, line in enumerate(text.splitlines(), start=1):
            if pattern.search(line):
                errors.append(f"{rel}:{lineno}: forbidden run-scoped generated path reference")
    return (0 if not errors else 1), errors

def check_no_ops_generated_placeholder(repo_root: Path) -> tuple[int, list[str]]:
    candidates = (repo_root / "ops/_generated/.keep", repo_root / "ops/_generated/.gitkeep")
    errors = [f"forbidden placeholder present: {p.relative_to(repo_root).as_posix()}" for p in candidates if p.exists()]
    return (0 if not errors else 1), errors

def check_ops_examples_immutable(repo_root: Path) -> tuple[int, list[str]]:
    root = repo_root / "ops/_examples"
    expected = {"report.example.json", "report.unified.example.json"}
    errors: list[str] = []
    if not root.exists():
        return 1, ["missing examples directory: ops/_examples"]
    actual = {p.name for p in root.glob("*.json")}
    for name in sorted(expected - actual):
        errors.append(f"missing example file: ops/_examples/{name}")
    for name in sorted(actual - expected):
        errors.append(f"unexpected example file: ops/_examples/{name}")
    for name in sorted(expected & actual):
        path = root / name
        if path.stat().st_size > 32 * 1024:
            errors.append(f"example too large (>32KB): {path.relative_to(repo_root).as_posix()}")
        text = path.read_text(encoding="utf-8")
        if re.search(r"atlas-ops-\d{8}", text):
            errors.append(f"example contains run-scoped atlas-ops timestamp id: {path.relative_to(repo_root).as_posix()}")
        if "ops/_generated/" in text:
            errors.append(f"example references mutable ops/_generated path: {path.relative_to(repo_root).as_posix()}")
        try:
            json.loads(text)
        except json.JSONDecodeError as exc:
            errors.append(f"invalid JSON in example {path.relative_to(repo_root).as_posix()}: {exc}")
    return (0 if not errors else 1), errors

def check_invocation_parity(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    py_mk = repo_root / "makefiles/python.mk"
    text = py_mk.read_text(encoding="utf-8")
    if "python3 -m atlasctl.cli" not in text:
        errors.append("makefiles/python.mk must invoke atlasctl via python -m atlasctl.cli")
    docs_text = (repo_root / "docs/development/tooling/atlasctl.md").read_text(
        encoding="utf-8", errors="ignore"
    )
    if re.search(r"scripts/bin/atlasctl", docs_text):
        errors.append("docs still reference scripts/bin/atlasctl")
    if "atlasctl" not in docs_text:
        errors.append("docs/development/tooling/atlasctl.md must reference atlasctl")
    return (0 if not errors else 1), errors

def check_scripts_surface_docs_drift(repo_root: Path) -> tuple[int, list[str]]:
    doc = repo_root / "docs/development/tooling/atlasctl.md"
    cfg = repo_root / "configs/scripts/python-tooling.json"
    payload = json.loads(cfg.read_text(encoding="utf-8"))
    commands = [str(cmd) for cmd in payload.get("commands", [])]
    text = doc.read_text(encoding="utf-8")
    missing = [f"missing `{cmd}` in {doc.relative_to(repo_root)}" for cmd in commands if f"`{cmd}`" not in text]
    return (0 if not missing else 1), missing

def check_script_errors(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for path in sorted((repo_root / "scripts/bin").glob("bijux-atlas-*")):
        if not path.is_file() or path.name == "bijux-atlas-dev":
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "python3 -m atlasctl.cli" in text:
            continue
        if '"error_code"' not in text and "err(" not in text:
            errors.append(f"{path.relative_to(repo_root)} must emit structured JSON error_code or delegate to atlasctl")
    return (0 if not errors else 1), errors

def check_script_write_roots(repo_root: Path) -> tuple[int, list[str]]:
    allowed = (
        "artifacts/",
        "ops/_generated/",
        "ops/_generated_committed/",
        "artifacts/evidence/",
        "docs/_generated/",
        "scripts/_generated/",
    )
    write_re = re.compile(r"\b(?:>|>>|tee\s+|mkdir\s+-p\s+|cp\s+[^\n]*\s+)([^\s\"']+)")
    errors: list[str] = []
    for path in sorted((repo_root / "scripts/bin").glob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for match in write_re.finditer(text):
            target = match.group(1)
            if target.startswith("$") or target.startswith("/") or target.startswith("."):
                continue
            if any(target.startswith(prefix) for prefix in allowed):
                continue
            errors.append(f"{rel}: {target}")
    return (0 if not errors else 1), errors
