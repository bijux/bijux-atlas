from __future__ import annotations

from pathlib import Path
from ...core.exec_shell import run_shell_script

_CHUNK_DIR = Path(__file__).with_name("runtime_chunks")
_CHUNKS = (
    "core/docs_contracts_core.py",
    "extended/docs_contracts_extended.py",
    "core/docs_generation_core.py",
    "extended/docs_generation_extended.py",
    "core/docs_validation_core.py",
    "extended/docs_validation_extended.py",
    "core/docs_outputs_core.py",
    "extended/docs_outputs_extended.py",
    "core/docs_navigation_core.py",
    "extended/docs_navigation_extended.py",
    "docs_inventory.py",
    "docs_dispatch.py",
    "docs_parser.py",
)


for _chunk in _CHUNKS:
    _path = _CHUNK_DIR / _chunk
    exec(_path.read_text(encoding="utf-8"), globals())

__all__ = ["configure_docs_parser", "run_docs_command"]
