from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path

from atlasctl.core import isolation


def test_generate_isolate_tag_is_deterministic_with_fixed_clock(monkeypatch) -> None:
    class FixedDateTime:
        @classmethod
        def now(cls, _tz=None):
            return datetime(2026, 2, 21, 19, 50, 0, tzinfo=timezone.utc)

    monkeypatch.setattr(isolation, "datetime", FixedDateTime)
    monkeypatch.setattr(isolation.os, "getppid", lambda: 4242)
    tag = isolation.generate_isolate_tag(git_sha="abcdef123456", prefix="ci")
    assert tag == "ci-20260221T195000Z-abcdef12-4242"


def test_resolve_isolate_root_uses_override_when_relative() -> None:
    repo_root = Path("/repo")
    root = isolation.resolve_isolate_root(repo_root=repo_root, tag="x", env={"ISO_ROOT": "artifacts/isolate/custom"})
    assert root == repo_root / "artifacts/isolate/custom"
