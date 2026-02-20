from __future__ import annotations

import socket

import pytest

_ALLOWED_MARKERS = {"unit", "integration", "slow"}


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
