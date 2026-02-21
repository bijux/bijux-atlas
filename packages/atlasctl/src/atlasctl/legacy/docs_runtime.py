from __future__ import annotations

from pathlib import Path

_CHUNK_DIR = Path(__file__).with_name("docs_runtime_chunks")
_CHUNKS = (
    "docs_contracts.py",
    "docs_generation.py",
    "docs_validation.py",
    "docs_outputs.py",
    "docs_navigation.py",
    "docs_inventory.py",
    "docs_dispatch.py",
    "docs_parser.py",
)

for _chunk in _CHUNKS:
    _path = _CHUNK_DIR / _chunk
    exec(_path.read_text(encoding="utf-8"), globals())

__all__ = ["configure_docs_parser", "run_docs_command"]
