from __future__ import annotations

import json
import subprocess
from pathlib import Path


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
        repo_root / "scripts/bin/bijux-atlas-dev",
        repo_root / "scripts/areas/check/no-duplicate-script-names.sh",
        repo_root / "scripts/areas/check/no-direct-path-usage.sh",
        repo_root / "scripts/areas/ci/scripts-ci.sh",
    ]
    errors: list[str] = []
    for p in targets:
        if not p.exists():
            errors.append(f"missing help-gated script: {p.relative_to(repo_root)}")
            continue
        proc = subprocess.run([str(p), "--help"], cwd=repo_root, text=True, capture_output=True, check=False)
        out = (proc.stdout or "") + (proc.stderr or "")
        if proc.returncode != 0:
            errors.append(f"{p.relative_to(repo_root)}: --help exited {proc.returncode}")
            continue
        low = out.lower()
        if "usage" not in low and "purpose" not in low and "contract" not in low:
            errors.append(f"{p.relative_to(repo_root)}: --help output missing usage/contract text")
    return (0 if not errors else 1), errors


def check_script_ownership(repo_root: Path) -> tuple[int, list[str]]:
    ownership_path = repo_root / "scripts/areas/_meta/ownership.json"
    payload = json.loads(ownership_path.read_text(encoding="utf-8"))
    areas = payload["areas"]
    errors: list[str] = []
    for p in sorted((repo_root / "scripts").rglob("*")):
        if not p.is_file():
            continue
        rel = p.relative_to(repo_root).as_posix()
        if rel.startswith("scripts/__pycache__"):
            continue
        matched = any(rel == area or rel.startswith(area + "/") for area in areas)
        if not matched:
            errors.append(rel)
    return (0 if not errors else 1), errors


def check_no_xtask_refs(repo_root: Path) -> tuple[int, list[str]]:
    include_roots = [
        repo_root / ".github",
        repo_root / "makefiles",
        repo_root / "configs",
        repo_root / "docs",
        repo_root / "scripts",
        repo_root / "packages",
        repo_root / "Cargo.toml",
    ]
    allowed_substrings = [
        "ADR",
        "adr",
    ]
    errors: list[str] = []
    ignore_paths = {
        "makefiles/ci.mk",
        "docs/development/xtask-removal-map.md",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/check/native.py",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/check/command.py",
        "packages/bijux-atlas-scripts/tests/test_check_native.py",
    }
    for root in include_roots:
        paths: list[Path]
        if isinstance(root, Path) and root.is_file():
            paths = [root]
        elif isinstance(root, Path) and root.exists():
            paths = [p for p in sorted(root.rglob("*")) if p.is_file()]
        else:
            paths = []
        for p in paths:
            rel = p.relative_to(repo_root).as_posix()
            if rel in ignore_paths:
                continue
            if p.suffix not in {".md", ".mk", ".toml", ".yml", ".yaml", ".json", ".py", ".sh", ""}:
                continue
            text = p.read_text(encoding="utf-8", errors="ignore")
            if "xtask" not in text:
                continue
            if any(tok in rel for tok in ("adr", "ADR")):
                continue
            if any(tok in text for tok in allowed_substrings) and ("history" in text.lower()):
                continue
            errors.append(rel)
    return (0 if not errors else 1), sorted(set(errors))


def check_make_help(repo_root: Path) -> tuple[int, list[str]]:
    cmd = ["make", "-s", "help"]
    p1 = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    p2 = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    if p1.returncode != 0 or p2.returncode != 0:
        return 1, ["`make -s help` failed while validating help output"]
    if p1.stdout != p2.stdout:
        return 1, ["`make -s help` output is non-deterministic across two runs"]
    return 0, []


def check_make_forbidden_paths(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    makefiles = [repo_root / "Makefile", *sorted((repo_root / "makefiles").glob("*.mk"))]
    forbidden = ("xtask/", "tools/")
    for mk in makefiles:
        text = mk.read_text(encoding="utf-8", errors="ignore").splitlines()
        for idx, line in enumerate(text, start=1):
            if not line.startswith("\t"):
                continue
            for token in forbidden:
                if token in line:
                    errors.append(f"{mk.relative_to(repo_root)}:{idx}: forbidden `{token}` in make recipe")
    return (0 if not errors else 1), errors
