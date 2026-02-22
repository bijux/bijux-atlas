from __future__ import annotations

import socket
from typing import Any, Callable

from ..effects import NetworkDecision, resolve_network_mode as resolve_network_mode_policy
from ..errors import ScriptError
from ..exit_codes import ERR_CONTEXT


class _NoNetworkSocket(socket.socket):
    def connect(self, address: Any) -> None:
        raise ScriptError(f"network disabled by --no-network: attempted connect to {address}", ERR_CONTEXT)


def install_no_network_guard() -> Callable[..., Any]:
    original_socket = socket.socket
    original_create_connection = socket.create_connection

    def blocked_create_connection(*args: Any, **kwargs: Any) -> Any:
        raise ScriptError("network disabled by --no-network: create_connection called", ERR_CONTEXT)

    socket.socket = _NoNetworkSocket  # type: ignore[assignment]
    socket.create_connection = blocked_create_connection  # type: ignore[assignment]

    def restore() -> Any:
        socket.socket = original_socket  # type: ignore[assignment]
        socket.create_connection = original_create_connection  # type: ignore[assignment]

    return restore


def resolve_network_mode(
    *,
    command_name: str,
    requested_allow_network: bool,
    explicit_network: str | None,
    deprecated_no_network: bool,
) -> NetworkDecision:
    return resolve_network_mode_policy(
        command_name=command_name,
        requested_allow_network=requested_allow_network,
        explicit_network=explicit_network,
        deprecated_no_network=deprecated_no_network,
    )
