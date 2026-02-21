"""External tool adapter helpers."""

from __future__ import annotations

from ..runtime.tooling import read_pins, read_tool_versions


__all__ = ["read_tool_versions", "read_pins", "tool_versions", "tool_pins"]


def tool_versions(repo_root):
    return read_tool_versions(repo_root)


def tool_pins(repo_root):
    return read_pins(repo_root)
