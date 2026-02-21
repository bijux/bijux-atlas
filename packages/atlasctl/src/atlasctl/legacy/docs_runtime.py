from __future__ import annotations

from pathlib import Path

_CHUNK_DIR = Path(__file__).with_name("docs_runtime_chunks")
_CHUNKS = (
    "chunk_01.py",
    "chunk_02.py",
    "chunk_03.py",
    "chunk_04.py",
    "chunk_05.py",
    "chunk_06.py",
    "chunk_07.py",
    "chunk_08.py",
)

for _chunk in _CHUNKS:
    _path = _CHUNK_DIR / _chunk
    exec(_path.read_text(encoding="utf-8"), globals())

__all__ = ["configure_docs_parser", "run_docs_command"]
