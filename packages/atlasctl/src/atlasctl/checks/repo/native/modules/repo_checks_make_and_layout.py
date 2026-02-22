from __future__ import annotations

import json
import os
import re
from datetime import date, datetime, timezone
from fnmatch import fnmatch
from pathlib import Path

from atlasctl.core.exec import run

from ..runtime_modules.repo_native_runtime_core import (
    _find_python_migration_exception,
    check_duplicate_script_names,
    check_script_help,
    check_script_ownership,
)

_WRAPPER_FILES = (
    "makefiles/dev.mk",
    "makefiles/docs.mk",
    "makefiles/ops.mk",
    "makefiles/ci.mk",
    "makefiles/policies.mk",
    "makefiles/product.mk",
    "makefiles/env.mk",
    "makefiles/root.mk",
)
_BANNED_TARGET_ADJECTIVES = (
    "".join(["el", "ite"]),
    "".join(["ref", "grade"]),
    "".join(["reference", "-", "grade"]),
    "".join(["production", "-", "grade"]),
)
_MAKE_TARGET_RE = re.compile(r"^(?P<target>[A-Za-z0-9_./-]+):(?:\s|$)")


def _iter_make_targets(repo_root: Path, rel_path: str) -> list[tuple[str, list[tuple[int, str]]]]:
    path = repo_root / rel_path
    if not path.exists():
        return []
    targets: list[tuple[str, list[tuple[int, str]]]] = []
    current_target = ""
    current_lines: list[tuple[int, str]] = []
    for lineno, raw in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
        target_match = _MAKE_TARGET_RE.match(raw)
        if target_match and not raw.startswith("."):
            if current_target:
                targets.append((current_target, current_lines))
            current_target = target_match.group("target")
            current_lines = []
            continue
        if raw.startswith("\t") and current_target:
            body = raw[1:].strip()
            if body:
                current_lines.append((lineno, body))
    if current_target:
        targets.append((current_target, current_lines))
    return targets


def _wrapper_target_recipes(repo_root: Path) -> list[tuple[str, str, int, str]]:
    rows: list[tuple[str, str, int, str]] = []
    for rel in _WRAPPER_FILES:
        if rel == "makefiles/root.mk":
            continue
        for target, recipe_lines in _iter_make_targets(repo_root, rel):
            if not recipe_lines or target.startswith("internal/"):
                continue
            for lineno, body in recipe_lines:
                rows.append((rel, target, lineno, body))
    return rows


def check_no_xtask_refs(repo_root: Path) -> tuple[int, list[str]]:
    include_roots = [
        repo_root / ".github",
        repo_root / "makefiles",
        repo_root / "configs",
        repo_root / "docs",
        repo_root / "packages",
        repo_root / "Cargo.toml",
    ]
    allowed_substrings = [
        "ADR",
        "adr",
    ]
    errors: list[str] = []
    ignore_paths = {
        "configs/policy/target-renames.json",
        "makefiles/ci.mk",
        "docs/development/task-runner-removal-map.md",
        "packages/atlasctl/src/atlasctl/checks/runner.py",
        "packages/atlasctl/src/atlasctl/checks/repo/__init__.py",
        "packages/atlasctl/src/atlasctl/checks/repo/paths.py",
        "packages/atlasctl/src/atlasctl/checks/repo/domains/paths.py",
        "packages/atlasctl/src/atlasctl/checks/repo/native/modules/repo_checks_scripts_and_docker.py",
        "packages/atlasctl/tests/checksuite/checks/test_check_native.py",
        "packages/atlasctl/tests/checksuite/checks/test_check_registry_features.py",
        "packages/atlasctl/src/atlasctl/checks/REGISTRY.toml",
        "packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json",
        "packages/atlasctl/src/atlasctl/commands/check/run.py",
        "packages/atlasctl/docs/checks/index.md",
        "packages/atlasctl/src/atlasctl/registry/checks_catalog.json",
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
            if "/.pytest_cache/" in rel or rel.startswith(".pytest_cache/"):
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
    p1 = run(cmd, cwd=repo_root, text=True, capture_output=True)
    p2 = run(cmd, cwd=repo_root, text=True, capture_output=True)
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


def check_make_no_direct_python_script_invocations(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    makefiles = [repo_root / "Makefile", *sorted((repo_root / "makefiles").glob("*.mk"))]
    direct_py_path_re = re.compile(r"\bpython3?\s+([^\s`]+\.py)\b")
    allowed_module_re = re.compile(r"\bpython3?\s+-m\s+atlasctl(?:\b|$)")
    for mk in makefiles:
        lines = mk.read_text(encoding="utf-8", errors="ignore").splitlines()
        for idx, line in enumerate(lines, start=1):
            if not line.startswith("\t"):
                continue
            if not direct_py_path_re.search(line):
                continue
            if allowed_module_re.search(line):
                continue
            rel = mk.relative_to(repo_root).as_posix()
            if _find_python_migration_exception(repo_root, "makefiles_direct_python", rel, line) is not None:
                continue
            errors.append(f"{rel}:{idx}: direct `python path/to/script.py` invocation is forbidden in Makefiles")
    return (0 if not errors else 1), errors


def _git_ls_files(repo_root: Path, pathspecs: list[str]) -> list[str]:
    cmd = ["git", "ls-files", "--", *pathspecs]
    proc = run(cmd, cwd=repo_root, text=True, capture_output=True)
    if proc.returncode != 0:
        return []
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]


def _looks_like_timestamp_segment(segment: str) -> bool:
    patterns = (
        r"^\d{8}$",
        r"^\d{8}-\d{6}$",
        r"^\d{4}-\d{2}-\d{2}$",
        r"^\d{4}-\d{2}-\d{2}[T_]\d{2}[:\-]?\d{2}[:\-]?\d{2}Z?$",
    )
    return any(re.match(pat, segment) for pat in patterns)


def check_ops_generated_tracked(repo_root: Path) -> tuple[int, list[str]]:
    tracked = _git_ls_files(repo_root, ["ops/_generated"])
    errors = [f"tracked runtime artifact: {path}" for path in tracked]
    return (0 if not errors else 1), errors


def check_tracked_timestamp_paths(repo_root: Path) -> tuple[int, list[str]]:
    tracked = _git_ls_files(repo_root, ["."])
    errors: list[str] = []
    for rel in tracked:
        segments = Path(rel).parts
        if any(_looks_like_timestamp_segment(seg) for seg in segments):
            errors.append(f"tracked path contains timestamp-like segment: {rel}")
    return (0 if not errors else 1), errors


def check_committed_generated_hygiene(repo_root: Path) -> tuple[int, list[str]]:
    tracked = _git_ls_files(
        repo_root,
        ["docs/_generated", "ops/_generated_committed", "ops/_generated.example"],
    )
    forbidden_suffixes = (".log", ".stderr", ".stdout", ".tmp")
    errors: list[str] = []
    for rel in tracked:
        path = Path(rel)
        segments = path.parts
        if any(_looks_like_timestamp_segment(seg) for seg in segments):
            errors.append(f"timestamp-like path in committed generated area: {rel}")
        if any(rel.endswith(sfx) for sfx in forbidden_suffixes):
            errors.append(f"runtime/log artifact in committed generated area: {rel}")
    return (0 if not errors else 1), errors


def _load_make_command_allowlist(repo_root: Path) -> list[str]:
    allowlist = repo_root / "configs/layout/make-command-allowlist.txt"
    if not allowlist.exists():
        return []
    return [
        ln.strip()
        for ln in allowlist.read_text(encoding="utf-8").splitlines()
        if ln.strip() and not ln.lstrip().startswith("#")
    ]


def _first_recipe_token(cmd: str) -> str:
    line = cmd.strip()
    while True:
        match = re.match(r"^[A-Za-z_][A-Za-z0-9_]*=(?:\"[^\"]*\"|'[^']*'|[^\s]+)\s+", line)
        if not match:
            break
        line = line[match.end() :].lstrip()
    if not line:
        return ""
    return line.split()[0]


def check_make_command_allowlist(repo_root: Path) -> tuple[int, list[str]]:
    allow = _load_make_command_allowlist(repo_root)
    if not allow:
        return 1, ["missing allowlist: configs/layout/make-command-allowlist.txt"]
    makefiles = [repo_root / "Makefile", *sorted((repo_root / "makefiles").glob("*.mk"))]
    skip_prefixes = ("if ", "for ", "while ", "case ", "{ ", "(", "then", "else", "fi", "do", "done")
    skip_tokens = {"\\", "-u", "-n", "-c", "-", "exit", "trap", "done;", "then", "fi;", "do"}
    violations: list[str] = []
    for mk in makefiles:
        continued = False
        phony_block = False
        for idx, raw in enumerate(mk.read_text(encoding="utf-8").splitlines(), start=1):
            if not raw.startswith("\t"):
                continued = False
                phony_block = raw.strip().startswith(".PHONY:")
                continue
            if phony_block:
                phony_block = raw.rstrip().endswith("\\")
                continue
            if continued:
                continued = raw.rstrip().endswith("\\")
                continue
            cmd = raw.lstrip()[1:].strip() if raw.lstrip().startswith("@") else raw.strip()
            continued = raw.rstrip().endswith("\\")
            if not cmd or cmd.startswith("#") or cmd.startswith("-"):
                continue
            if "$$(" in cmd or "|" in cmd:
                continue
            if ":?" in cmd:
                continue
            if any(cmd.startswith(prefix) for prefix in skip_prefixes):
                continue
            tok = _first_recipe_token(cmd)
            if not tok:
                continue
            if tok in skip_tokens:
                continue
            if tok.startswith("./") or tok.startswith('"') or tok.startswith("'"):
                continue
            if tok.startswith("$(") or tok.startswith("$${") or tok.startswith('"$('):
                continue
            if not re.fullmatch(r"[A-Za-z0-9_.+-]+", tok):
                continue
            if any(tok == item or tok.startswith(item) for item in allow):
                continue
            violations.append(f"{mk.relative_to(repo_root)}:{idx}: disallowed recipe command `{tok}`")
    return (0 if not violations else 1), violations


def check_make_wrapper_target_budget(repo_root: Path) -> tuple[int, list[str]]:
    budgets: dict[str, int] = {
        "makefiles/dev.mk": 10,
        "makefiles/docs.mk": 5,
        "makefiles/ops.mk": 6,
        "makefiles/ci.mk": 10,
        "makefiles/atlasctl.mk": 16,
        "makefiles/policies.mk": 1,
        "makefiles/product.mk": 24,
        "makefiles/root.mk": 220,
    }
    target_re = re.compile(r"^(?P<target>[A-Za-z0-9_./-]+):(?:\s|$)")
    errors: list[str] = []
    for rel, max_targets in sorted(budgets.items()):
        path = repo_root / rel
        if not path.exists():
            errors.append(f"{rel}: missing wrapper makefile for target budget check")
            continue
        count = 0
        for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
            m = target_re.match(line)
            if not m:
                continue
            target = m.group("target")
            if target.startswith("."):
                continue
            if target.startswith("internal/"):
                continue
            count += 1
        if count > max_targets:
            errors.append(f"{rel}: wrapper target budget exceeded ({count} > {max_targets})")
    return (0 if not errors else 1), errors


def check_make_wrapper_no_multiline_recipes(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for rel in _WRAPPER_FILES:
        if rel == "makefiles/root.mk":
            continue
        for target, recipe_lines in _iter_make_targets(repo_root, rel):
            if not recipe_lines or target.startswith("internal/"):
                continue
            if len(recipe_lines) != 1:
                errors.append(f"{rel}:{target}: wrapper target must have exactly one recipe line")
    return (0 if not errors else 1), sorted(errors)


def check_make_wrapper_only_calls_bin_atlasctl(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for rel, target, lineno, body in _wrapper_target_recipes(repo_root):
        if body.startswith("@./bin/atlasctl ") or body.startswith("./bin/atlasctl "):
            continue
        errors.append(f"{rel}:{lineno}: {target} must delegate via ./bin/atlasctl")
    return (0 if not errors else 1), sorted(errors)


def check_make_wrapper_no_env_side_effects(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    forbidden = ("export ", "unset ", "mkdir ", "rm ", "cp ", "mv ", "touch ", "tee ", ">>", ";", "&&", "||", "|")
    for rel, target, lineno, body in _wrapper_target_recipes(repo_root):
        clean = body.lstrip("@").strip()
        if not clean.startswith("./bin/atlasctl "):
            continue
        atlasctl_call = clean[len("./bin/atlasctl "):]
        if any(token in atlasctl_call for token in forbidden):
            errors.append(f"{rel}:{lineno}: {target} contains shell/env side-effect tokens")
    return (0 if not errors else 1), sorted(errors)


def check_make_wrapper_no_direct_cargo(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for rel, target, lineno, body in _wrapper_target_recipes(repo_root):
        if re.search(r"\bcargo\b", body):
            errors.append(f"{rel}:{lineno}: {target} must not invoke cargo directly")
    return (0 if not errors else 1), sorted(errors)


def check_make_wrapper_shell_is_sh(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for rel in _WRAPPER_FILES:
        path = repo_root / rel
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "SHELL := /bin/sh" not in text:
            errors.append(f"{rel}: wrapper makefile must declare `SHELL := /bin/sh`")
    return (0 if not errors else 1), sorted(errors)


def check_make_wrapper_phony_complete(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for rel in _WRAPPER_FILES:
        if rel == "makefiles/root.mk":
            continue
        path = repo_root / rel
        if not path.exists():
            continue
        phony: set[str] = set()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for line in text.splitlines():
            if line.startswith(".PHONY:"):
                phony.update(line.replace(".PHONY:", "", 1).split())
        for target, recipe_lines in _iter_make_targets(repo_root, rel):
            if not recipe_lines:
                continue
            if target.startswith("internal/"):
                continue
            if target not in phony:
                errors.append(f"{rel}:{target}: missing from .PHONY")
    return (0 if not errors else 1), sorted(errors)


def check_make_target_names_no_banned_adjectives(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    files = [("Makefile", repo_root / "Makefile"), *((p.relative_to(repo_root).as_posix(), p) for p in sorted((repo_root / "makefiles").glob("*.mk")))]
    for rel_name, path in files:
        if not path.exists():
            continue
        for target, _ in _iter_make_targets(repo_root, rel_name):
            lower = target.lower()
            banned = [token for token in _BANNED_TARGET_ADJECTIVES if token in lower]
            if banned:
                errors.append(f"{rel_name}:{target}: banned adjective in target name ({', '.join(sorted(set(banned)))})")
    return (0 if not errors else 1), sorted(errors)


def check_make_no_duplicate_all_variants(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for rel in _WRAPPER_FILES:
        if rel == "makefiles/root.mk":
            continue
        target_rows = _iter_make_targets(repo_root, rel)
        recipe_by_target: dict[str, str] = {}
        for target, recipe_lines in target_rows:
            if not recipe_lines:
                continue
            if len(recipe_lines) != 1:
                continue
            recipe_by_target[target] = recipe_lines[0][1].strip()
        for target, recipe in sorted(recipe_by_target.items()):
            if not target.endswith("-all"):
                continue
            base = target[:-4]
            if base not in recipe_by_target:
                errors.append(f"{rel}:{target}: -all variant exists but base target `{base}` is missing")
                continue
            base_recipe = recipe_by_target[base]
            if recipe == base_recipe:
                errors.append(f"{rel}:{target}: -all recipe duplicates base target `{base}`")
            if "--all" not in recipe and "--include-slow" not in recipe and "--slow" not in recipe:
                errors.append(f"{rel}:{target}: -all variant must include explicit full-behavior flag")
    return (0 if not errors else 1), errors


def check_layout_contract(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []

    # Root shape contract.
    surfaces = json.loads((repo_root / "configs/repo/surfaces.json").read_text(encoding="utf-8"))
    allow_dirs = set(surfaces.get("allowed_root_dirs", [])) | set(surfaces.get("canonical_surfaces", []))
    allow_files = {
        ln.strip()
        for ln in (repo_root / "configs/repo/root-files-allowlist.txt").read_text(encoding="utf-8").splitlines()
        if ln.strip() and not ln.strip().startswith("#")
    }
    allow_files |= set(surfaces.get("allowed_root_files", []))
    for entry in sorted(repo_root.iterdir(), key=lambda p: p.name):
        if entry.name in {".git"}:
            continue
        if entry.name in {".DS_Store"}:
            continue
        if entry.is_dir():
            if entry.name not in allow_dirs:
                errors.append(f"unexpected root directory: {entry.name}")
        elif entry.is_file() or entry.is_symlink():
            if entry.name not in allow_files:
                errors.append(f"unexpected root file/symlink: {entry.name}")

    # Workflow make-only contract.
    run_line = re.compile(r"^\s*-\s*run:\s*(.+)\s*$")
    for wf in sorted((repo_root / ".github/workflows").glob("*.yml")):
        for idx, line in enumerate(wf.read_text(encoding="utf-8").splitlines(), start=1):
            m = run_line.match(line)
            if not m:
                continue
            cmd = m.group(1).strip().strip('"')
            if cmd.startswith("|"):
                errors.append(f"{wf.relative_to(repo_root)}:{idx}: multiline run block forbidden")
                continue
            if not cmd.startswith("make "):
                errors.append(f"{wf.relative_to(repo_root)}:{idx}: workflow run must use make, found `{cmd}`")

    # Legacy target names contract.
    target_re = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", re.M)
    forbidden_legacy = re.compile(r"(^|/)legacy($|-)")
    for mk in sorted((repo_root / "makefiles").glob("*.mk")):
        text = mk.read_text(encoding="utf-8")
        for target in target_re.findall(text):
            if target.startswith("."):
                continue
            if forbidden_legacy.search(target):
                errors.append(f"{mk.relative_to(repo_root)}: forbidden legacy target `{target}`")

    # Symlink policy contract.
    symlink_cfg = json.loads((repo_root / "configs/repo/symlink-allowlist.json").read_text(encoding="utf-8"))
    allowed_root = symlink_cfg.get("root", {})
    allowed_non_root = symlink_cfg.get("non_root", {})
    for rel, target in sorted(allowed_root.items()):
        p = repo_root / rel
        if not p.is_symlink():
            errors.append(f"missing allowlisted root symlink: {rel}")
            continue
        resolved = p.resolve()
        try:
            got = resolved.relative_to(repo_root).as_posix()
        except ValueError:
            errors.append(f"root symlink points outside repo: {rel}")
            continue
        if got != target:
            errors.append(f"root symlink target drift: {rel} -> {got} (expected {target})")
    for rel, target in sorted(allowed_non_root.items()):
        p = repo_root / rel
        if not p.exists():
            continue
        if not p.is_symlink():
            errors.append(f"allowlisted non-root path exists but is not symlink: {rel}")
            continue
        resolved = p.resolve()
        try:
            got = resolved.relative_to(repo_root).as_posix()
        except ValueError:
            errors.append(f"non-root symlink points outside repo: {rel}")
            continue
        if got != target:
            errors.append(f"non-root symlink target drift: {rel} -> {got} (expected {target})")

    # Existing tracked/generated hygiene contracts.
    for fn in (check_ops_generated_tracked, check_tracked_timestamp_paths, check_committed_generated_hygiene):
        _code, errs = fn(repo_root)
        errors.extend(errs)

    return (0 if not errors else 1), errors
