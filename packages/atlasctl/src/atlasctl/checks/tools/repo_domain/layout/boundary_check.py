from __future__ import annotations

import ast
from dataclasses import dataclass
from pathlib import Path

PACKAGE = "atlasctl"

ALLOWED_DEPS: dict[str, set[str]] = {
    "core": {"core"},
    "contracts": {"contracts", "core"},
    "ops": {"ops", "core", "contracts"},
    "make": {"make", "core", "contracts"},
    "docs": {"docs", "core", "contracts"},
    "configs": {"configs", "core", "contracts"},
    "policies": {"policies", "core", "contracts", "registry"},
    "registry": {"registry", "core", "contracts"},
    "report": {"report", "core", "contracts", "registry"},
    "layout": {"layout", "core", "contracts"},
    "inventory": {"inventory", "core", "contracts", "layout"},
    "cli": {
        "cli",
        "core",
        "contracts",
        "ops",
        "make",
        "docs",
        "configs",
        "policies",
        "registry",
        "inventory",
        "report",
        "layout",
        "doctor",
        "domain_cmd",
        "errors",
        "exit_codes",
        "network_guard",
        "runner",
        "surface",
        "compat",
    },
    "doctor": {"doctor", "core", "registry", "contracts"},
    "compat": {"compat", "core", "contracts"},
}

COMPAT_MODULES = {"run_context", "structured_log", "evidence_policy"}


@dataclass(frozen=True)
class Violation:
    source: str
    target: str
    file: Path
    line: int


def _module_group_for_path(file_path: Path, src_root: Path) -> str:
    rel = file_path.relative_to(src_root / PACKAGE)
    parts = rel.parts
    if len(parts) == 1:
        stem = file_path.stem
        if stem in COMPAT_MODULES:
            return "compat"
        return stem
    return parts[0]


def _imported_package_group(mod: str) -> str | None:
    if not mod.startswith(PACKAGE):
        return None
    tail = mod[len(PACKAGE) :].lstrip(".")
    if not tail:
        return None
    first = tail.split(".")[0]
    if first in COMPAT_MODULES:
        return "compat"
    return first


def check_boundaries(repo_root: Path) -> list[Violation]:
    src_root = repo_root / "packages" / "atlasctl" / "src"
    pkg_root = src_root / PACKAGE
    violations: list[Violation] = []

    for py_file in sorted(pkg_root.rglob("*.py")):
        if "__pycache__" in py_file.parts:
            continue
        source_group = _module_group_for_path(py_file, src_root)
        allowed = ALLOWED_DEPS.get(source_group, {source_group, "core", "contracts"})
        tree = ast.parse(py_file.read_text(encoding="utf-8"), filename=str(py_file))

        for node in ast.walk(tree):
            imported: list[tuple[str, int]] = []
            if isinstance(node, ast.Import):
                imported = [(alias.name, node.lineno) for alias in node.names]
            elif isinstance(node, ast.ImportFrom):
                module = node.module or ""
                if node.level > 0:
                    # Relative imports remain in-module by default in this package layout.
                    continue
                else:
                    imported = [(module, node.lineno)]

            for mod, lineno in imported:
                target_group = _imported_package_group(mod)
                if target_group is None:
                    continue
                if target_group == source_group:
                    continue
                if target_group not in allowed:
                    violations.append(
                        Violation(
                            source=source_group,
                            target=target_group,
                            file=py_file.relative_to(repo_root),
                            line=lineno,
                        )
                    )
    return violations


def main() -> int:
    repo_root = next((parent for parent in Path(__file__).resolve().parents if (parent / ".git").exists()), None)
    if repo_root is None:
        print("bijux-atlas boundary check failed: unable to locate repository root")
        return 1
    violations = check_boundaries(repo_root)
    if violations:
        print("bijux-atlas boundary check failed")
        for v in violations:
            print(f"- {v.file}:{v.line} disallowed import {v.source} -> {v.target}")
        return 1
    print("bijux-atlas boundary check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
