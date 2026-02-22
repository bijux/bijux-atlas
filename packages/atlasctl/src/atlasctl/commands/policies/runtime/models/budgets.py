from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class BudgetRule:
    name: str
    path_glob: str
    enforce: bool
    max_entries_per_dir: int
    max_py_files_per_dir: int
    max_modules_per_dir: int
    max_shell_files_per_dir: int
    max_loc_per_file: int
    max_loc_per_dir: int
    max_total_bytes_per_dir: int
    max_imports_per_file: int
    max_public_symbols_per_module: int
    max_branch_keywords_per_file: int


@dataclass(frozen=True)
class BudgetException:
    path: str
    reason: str


@dataclass(frozen=True)
class DirStat:
    dir: str
    py_files: int
    modules: int
    shell_files: int
    total_loc: int
    total_bytes: int
    top_offenders: list[dict[str, Any]]
    rule: str
    enforce: bool
    budget: dict[str, int]


@dataclass(frozen=True)
class FileStat:
    path: str
    loc: int
    import_count: int
    public_symbols: int
    branch_keywords: int
    rule: str
    enforce: bool
    budget: dict[str, int]
