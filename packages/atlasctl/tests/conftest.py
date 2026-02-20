from __future__ import annotations

import socket
from pathlib import Path

import pytest
from hypothesis import settings
from hypothesis.database import DirectoryBasedExampleDatabase

_ALLOWED_MARKERS = {"unit", "integration", "slow"}

_ROOT = Path(__file__).resolve().parents[3]
_HYPOTHESIS_DB = _ROOT / "artifacts/atlasctl/.hypothesis/examples"
_HYPOTHESIS_DB.parent.mkdir(parents=True, exist_ok=True)
settings.register_profile("atlas", database=DirectoryBasedExampleDatabase(_HYPOTHESIS_DB))
settings.load_profile("atlas")


def pytest_collection_modifyitems(items: list[pytest.Item]) -> None:
    for item in items:
        names = {mark.name for mark in item.iter_markers()}
        if not names.intersection(_ALLOWED_MARKERS):
            item.add_marker("unit")


@pytest.fixture(autouse=True)
def no_network_for_unit(request: pytest.FixtureRequest, monkeypatch: pytest.MonkeyPatch) -> None:
    if request.node.get_closest_marker("integration") or request.node.get_closest_marker("slow"):
        return

    def _blocked(*_args: object, **_kwargs: object) -> socket.socket:
        raise RuntimeError("network disabled in unit tests")

    def _blocked_connect(*_args: object, **_kwargs: object) -> None:
        raise RuntimeError("network disabled in unit tests")

    monkeypatch.setattr(socket, "create_connection", _blocked)
    monkeypatch.setattr(socket.socket, "connect", _blocked_connect)


@pytest.fixture
def minimal_repo_root(tmp_path: Path) -> Path:
    repo = tmp_path / "repo"
    (repo / ".git").mkdir(parents=True)
    (repo / "makefiles").mkdir()
    (repo / "configs").mkdir()
    (repo / "docs/_generated").mkdir(parents=True)
    (repo / "ops").mkdir()
    (repo / "artifacts/evidence").mkdir(parents=True)
    return repo
