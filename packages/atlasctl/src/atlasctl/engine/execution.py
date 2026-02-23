from __future__ import annotations

# Canonical check engine execution runtime (migrated from `checks/core/execution.py`
# during the runtime/check-engine cutover). Keep dependencies narrow and move policy enforcement to
# runtime/capability boundaries over time.

import time
import signal
import builtins
import socket
import subprocess
import os
from concurrent.futures import ThreadPoolExecutor
from contextlib import contextmanager
from pathlib import Path
from typing import Callable

from ..checks.core.base import CheckDef, CheckResult
from ..checks.effects import CheckEffect, normalize_effect
from ..checks.registry.catalog import check_tags
from ..core.process import run_command


class CommandCheckDef:
    def __init__(self, check_id: str, domain: str, cmd: list[str], budget_ms: int = 10_000) -> None:
        self.check_id = check_id
        self.domain = domain
        self.cmd = cmd
        self.budget_ms = budget_ms


def run_command_checks(repo_root: Path, checks: list[CommandCheckDef]) -> tuple[int, list[dict[str, object]]]:
    rows: list[dict[str, object]] = []
    failed = 0
    for chk in checks:
        start = time.perf_counter()
        cmd = list(chk.cmd)
        cmd_str = " ".join(cmd)
        if "atlasctl.cli" in cmd_str or (len(cmd) >= 3 and Path(cmd[0]).name.startswith("python") and cmd[1] == "-m"):
            src_path = str(repo_root / "packages/atlasctl/src")
            cmd = ["env", f"PYTHONPATH={src_path}", *cmd]
        result = run_command(cmd, repo_root)
        elapsed_ms = int((time.perf_counter() - start) * 1000)
        status = "pass" if result.code == 0 else "fail"
        if result.code != 0:
            failed += 1
        row: dict[str, object] = {
            "id": chk.check_id,
            "domain": chk.domain,
            "command": " ".join(cmd),
            "status": status,
            "duration_ms": elapsed_ms,
            "budget_ms": chk.budget_ms,
            "budget_status": "pass" if elapsed_ms <= chk.budget_ms else "warn",
        }
        if result.code != 0:
            row["error"] = result.combined_output
        rows.append(row)
    return failed, rows


def run_function_checks(
    repo_root: Path,
    checks: list[CheckDef],
    on_result: Callable[[CheckResult], None] | None = None,
    timeout_ms: int | None = None,
    jobs: int = 1,
    run_root: Path | None = None,
) -> tuple[int, list[CheckResult]]:
    checks = sorted(checks, key=lambda c: c.check_id)
    strict_effects = os.environ.get("ATLASCTL_STRICT_CHECK_EFFECTS", "").strip().lower() in {"1", "true", "yes", "on"}

    def _is_write_mode(mode: str) -> bool:
        return any(token in mode for token in ("w", "a", "x", "+"))

    def _normalize_path(value: object) -> Path:
        if isinstance(value, Path):
            p = value
        else:
            p = Path(str(value))
        return p if p.is_absolute() else (repo_root / p)

    @contextmanager
    def _runtime_guards(chk: CheckDef):
        declared_effects = {normalize_effect(effect) for effect in chk.effects}
        declared_effects.add(CheckEffect.FS_READ.value)
        allowed_effectful = CheckEffect.FS_WRITE.value in declared_effects
        allow_subprocess = CheckEffect.SUBPROCESS.value in declared_effects
        allow_network = CheckEffect.NETWORK.value in declared_effects
        allowed_roots = [((repo_root / rel).resolve()) for rel in chk.writes_allowed_roots]
        if run_root is not None:
            allowed_roots.append(run_root.resolve())
        print_calls: list[str] = []
        observed_effects: set[str] = {CheckEffect.FS_READ.value}

        original_print = builtins.print
        original_open = builtins.open
        original_path_open = Path.open
        original_write_text = Path.write_text
        original_write_bytes = Path.write_bytes
        original_touch = Path.touch
        original_subprocess_run = subprocess.run
        original_socket_create_connection = socket.create_connection

        def _guarded_print(*args: object, **kwargs: object) -> None:  # noqa: ANN401
            msg = " ".join(str(x) for x in args)
            print_calls.append(msg)

        def _validate_write_target(file: object, mode: str) -> None:
            if _is_write_mode(mode):
                observed_effects.add(CheckEffect.FS_WRITE.value)
                if strict_effects and not allowed_effectful:
                    raise PermissionError(f"{chk.check_id}: file writes require effects.fs_write=true")
                target = _normalize_path(file).resolve()
                if strict_effects and not any(target == root or root in target.parents for root in allowed_roots):
                    raise PermissionError(f"{chk.check_id}: write outside allowed roots: {target}")

        def _guarded_open(file: object, mode: str = "r", *args: object, **kwargs: object):  # noqa: ANN401
            _validate_write_target(file, mode)
            return original_open(file, mode, *args, **kwargs)

        def _guarded_path_open(self: Path, mode: str = "r", *args: object, **kwargs: object):  # noqa: ANN001
            _validate_write_target(self, mode)
            return original_path_open(self, mode, *args, **kwargs)

        def _guarded_write_text(self: Path, data: str, *args: object, **kwargs: object):  # noqa: ANN001
            _validate_write_target(self, "w")
            return original_write_text(self, data, *args, **kwargs)

        def _guarded_write_bytes(self: Path, data: bytes, *args: object, **kwargs: object):  # noqa: ANN001
            _validate_write_target(self, "wb")
            return original_write_bytes(self, data, *args, **kwargs)

        def _guarded_touch(self: Path, *args: object, **kwargs: object):  # noqa: ANN001
            _validate_write_target(self, "a")
            return original_touch(self, *args, **kwargs)

        def _guarded_subprocess_run(*args: object, **kwargs: object):  # noqa: ANN401
            observed_effects.add(CheckEffect.SUBPROCESS.value)
            if strict_effects and not allow_subprocess:
                raise PermissionError(f"{chk.check_id}: subprocess usage requires effects.subprocess=true")
            return original_subprocess_run(*args, **kwargs)

        def _guarded_create_connection(*args: object, **kwargs: object):  # noqa: ANN401
            observed_effects.add(CheckEffect.NETWORK.value)
            if strict_effects and not allow_network:
                raise PermissionError(f"{chk.check_id}: network usage requires effects.network=true")
            return original_socket_create_connection(*args, **kwargs)

        builtins.print = _guarded_print
        builtins.open = _guarded_open
        Path.open = _guarded_path_open
        Path.write_text = _guarded_write_text
        Path.write_bytes = _guarded_write_bytes
        Path.touch = _guarded_touch
        subprocess.run = _guarded_subprocess_run
        socket.create_connection = _guarded_create_connection
        try:
            yield print_calls, observed_effects, declared_effects
        finally:
            builtins.print = original_print
            builtins.open = original_open
            Path.open = original_path_open
            Path.write_text = original_write_text
            Path.write_bytes = original_write_bytes
            Path.touch = original_touch
            subprocess.run = original_subprocess_run
            socket.create_connection = original_socket_create_connection

    def _run_one(chk: CheckDef) -> CheckResult:
        start = time.perf_counter()
        try:
            with _runtime_guards(chk) as (print_calls, observed_effects, declared_effects):
                if timeout_ms and timeout_ms > 0 and jobs <= 1:
                    class _CheckTimeoutError(Exception):
                        pass

                    def _raise_timeout(_signum: int, _frame: object) -> None:
                        raise _CheckTimeoutError()

                    prev_handler = signal.getsignal(signal.SIGALRM)
                    signal.signal(signal.SIGALRM, _raise_timeout)
                    signal.setitimer(signal.ITIMER_REAL, timeout_ms / 1000.0)
                    try:
                        code, errors = chk.fn(repo_root)
                    except _CheckTimeoutError:
                        code, errors = 1, [f"check timed out after {timeout_ms}ms"]
                    finally:
                        signal.setitimer(signal.ITIMER_REAL, 0)
                        signal.signal(signal.SIGALRM, prev_handler)
                else:
                    code, errors = chk.fn(repo_root)
                if strict_effects and print_calls:
                    code = 1
                    errors = [*errors, f"{chk.check_id}: checks must not print to stdout/stderr"]
                undeclared = sorted(observed_effects.difference(declared_effects))
                if strict_effects and undeclared:
                    code = 1
                    errors = [*errors, f"{chk.check_id}: undeclared effects used: {', '.join(undeclared)}"]
        except Exception as exc:
            code, errors = 1, [f"internal check error: {exc}"]
        elapsed_ms = int((time.perf_counter() - start) * 1000)
        warnings = sorted([msg.removeprefix("WARN: ").strip() for msg in errors if msg.startswith("WARN:")])
        normalized_errors = sorted([msg for msg in errors if not msg.startswith("WARN:")])
        status = "pass" if code == 0 else "fail"
        return CheckResult(
            id=chk.check_id,
            title=chk.title,
            domain=chk.domain,
            status=status,
            errors=normalized_errors,
            warnings=warnings,
            evidence_paths=list(chk.evidence),
            metrics={
                "duration_ms": elapsed_ms,
                "budget_ms": chk.budget_ms,
                "budget_status": "pass" if elapsed_ms <= chk.budget_ms else "warn",
            },
            description=chk.description,
            fix_hint=chk.fix_hint,
            category=chk.category.value,
            severity=chk.severity.value,
            tags=list(check_tags(chk)),
            effects=list(chk.effects),
            effects_used=sorted(observed_effects) if "observed_effects" in locals() else [CheckEffect.FS_READ.value],
            owners=list(chk.owners),
            writes_allowed_roots=list(chk.writes_allowed_roots),
            result_code=chk.result_code,
        )

    rows: list[CheckResult] = []
    failed = 0
    if jobs > 1:
        with ThreadPoolExecutor(max_workers=jobs) as ex:
            produced = list(ex.map(_run_one, checks))
        rows.extend(produced)
    else:
        for chk in checks:
            rows.append(_run_one(chk))
    for row in rows:
        if row.status != "pass":
            failed += 1
        if on_result is not None:
            on_result(row)
    return failed, rows
