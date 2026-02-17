#!/usr/bin/env python3
# Purpose: shared path utilities for script tooling.
# Inputs: repo-relative path segments.
# Outputs: normalized pathlib paths rooted at repository root.
from __future__ import annotations

from pathlib import Path


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def repo_path(*parts: str) -> Path:
    return repo_root().joinpath(*parts)
