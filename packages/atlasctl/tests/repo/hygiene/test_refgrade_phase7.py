from __future__ import annotations

import ast
import json
from pathlib import Path

from tests.helpers import run_atlasctl

ROOT = Path(__file__).resolve().parents[3]
SRC_ROOT = ROOT / "packages/atlasctl/src/atlasctl"


def test_no_symlinks_inside_src_tree() -> None:
    links = [p.relative_to(ROOT).as_posix() for p in SRC_ROOT.rglob("*") if p.is_symlink()]
    assert links == [], f"symlinks are forbidden in src tree: {links}"


def test_no_duplicate_command_names_and_no_shadow_import_modules() -> None:
    proc = run_atlasctl("--quiet", "help", "--json")
    assert proc.returncode == 0, proc.stderr
    commands = json.loads(proc.stdout)["commands"]
    names = [row["name"] for row in commands]
    assert len(names) == len(set(names)), "duplicate command names in CLI inventory"

    forbidden = {"argparse", "json", "logging", "pathlib", "subprocess", "typing"}
    top_level = {p.stem for p in SRC_ROOT.glob("*.py")}
    overlap = sorted(top_level.intersection(forbidden))
    assert overlap == [], f"top-level atlasctl modules shadow stdlib names: {overlap}"


def test_refgrade_target_tree_shape() -> None:
    package_root = ROOT / "packages/atlasctl"
    allowed = {
        "src",
        "tests",
        "docs",
        "pyproject.toml",
        "README.md",
        "LICENSE",
        "requirements.in",
        "requirements.lock.txt",
        ".pytest_cache",
    }
    current = {p.name for p in package_root.iterdir()}
    unexpected = sorted(current - allowed)
    assert unexpected == [], f"unexpected package-root items: {unexpected}"

    offenders: list[str] = []
    for path in sorted(SRC_ROOT.rglob("*.py")):
        rel = path.relative_to(ROOT).as_posix()
        if "/legacy/" in rel:
            continue
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                if any(alias.name.startswith("atlasctl.legacy") for alias in node.names):
                    offenders.append(rel)
                    break
            elif isinstance(node, ast.ImportFrom):
                if node.level == 0 and node.module and node.module.startswith("atlasctl.legacy"):
                    offenders.append(rel)
                    break
    assert offenders == [], f"modern modules importing atlasctl.legacy: {sorted(set(offenders))}"


def test_goldens_are_generated_only_via_gen_command() -> None:
    offenders: list[str] = []
    for path in sorted(SRC_ROOT.rglob("*.py")):
        rel = path.relative_to(ROOT).as_posix()
        if rel == "packages/atlasctl/src/atlasctl/gen/command.py":
            continue
        text = path.read_text(encoding="utf-8")
        if "tests/goldens" in text:
            offenders.append(rel)
    assert offenders == [], f"only atlasctl gen goldens may reference tests/goldens write path: {offenders}"
