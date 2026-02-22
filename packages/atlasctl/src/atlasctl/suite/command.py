from __future__ import annotations

import argparse
import json
import os
import shlex
import subprocess
import sys
import time
from dataclasses import dataclass
from fnmatch import fnmatch
from pathlib import Path
from typing import Any
from xml.etree.ElementTree import Element, SubElement, tostring

from ..checks.core.execution import run_function_checks
from ..checks.registry import check_tags, get_check, list_checks
from ..cli.surface_registry import command_registry
from ..contracts.catalog import schema_path_for
from ..contracts.ids import SUITE_RUN
from ..contracts.validate_self import validate_self
from ..contracts.validate import validate_file
from ..core.context import RunContext
from ..core.runtime.telemetry import emit_telemetry
from ..core.runtime.logging import log_event
from ..core.runtime.serialize import dumps_json
from ..core.errors import ScriptError
from ..core.exit_codes import ERR_CONFIG, ERR_USER
from .manifests import SuiteManifest, load_first_class_suites

try:
    import tomllib  # py311+
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore


@dataclass(frozen=True)
class SuiteSpec:
    name: str
    includes: tuple[str, ...]
    items: tuple[str, ...]
    complete: bool = False


@dataclass(frozen=True)
class TaskSpec:
    suite: str
    kind: str
    value: str

    @property
    def label(self) -> str:
        return f"{self.kind} {self.value}"


def _load_pyproject(repo_root: Path) -> dict[str, Any]:
    pyproject = repo_root / "packages/atlasctl/pyproject.toml"
    if not pyproject.exists():
        raise ScriptError("missing packages/atlasctl/pyproject.toml", ERR_CONFIG)
    return tomllib.loads(pyproject.read_text(encoding="utf-8"))


def load_suites(repo_root: Path) -> tuple[str, dict[str, SuiteSpec]]:
    data = _load_pyproject(repo_root)
    root = data.get("tool", {}).get("atlasctl", {}).get("suites", {})
    if not isinstance(root, dict):
        raise ScriptError("invalid [tool.atlasctl.suites] config", ERR_CONFIG)
    default = str(root.get("default", "required")).strip() or "required"
    suites: dict[str, SuiteSpec] = {}
    for name, value in root.items():
        if name == "default" or not isinstance(value, dict):
            continue
        includes = tuple(str(item).strip() for item in value.get("includes", []) if str(item).strip())
        items = tuple(str(item).strip() for item in value.get("items", []) if str(item).strip())
        suites[name] = SuiteSpec(name=name, includes=includes, items=items, complete=bool(value.get("complete", False)))
    if default not in suites:
        raise ScriptError(f"default suite `{default}` is not defined in pyproject", ERR_CONFIG)
    return default, suites


def _parse_task(suite: str, raw: str) -> TaskSpec:
    if ":" not in raw:
        raise ScriptError(f"invalid suite task entry `{raw}` in suite `{suite}`", ERR_CONFIG)
    kind, value = raw.split(":", 1)
    kind = kind.strip()
    value = value.strip()
    if kind not in {"check", "check-tag", "cmd", "schema"} or not value:
        raise ScriptError(f"invalid suite task entry `{raw}` in suite `{suite}`", ERR_CONFIG)
    return TaskSpec(suite=suite, kind=kind, value=value)


def expand_suite(suites: dict[str, SuiteSpec], suite_name: str) -> list[TaskSpec]:
    if suite_name not in suites:
        raise ScriptError(f"unknown suite `{suite_name}`", ERR_CONFIG)
    expanded: list[TaskSpec] = []
    seen_stack: list[str] = []

    def visit(name: str) -> None:
        if name in seen_stack:
            cycle = " -> ".join([*seen_stack, name])
            raise ScriptError(f"suite include cycle detected: {cycle}", ERR_CONFIG)
        spec = suites.get(name)
        if spec is None:
            raise ScriptError(f"suite `{name}` references unknown include", ERR_CONFIG)
        seen_stack.append(name)
        for include in spec.includes:
            visit(include)
        for item in spec.items:
            expanded.append(_parse_task(name, item))
        seen_stack.pop()

    visit(suite_name)
    return expanded


def _matches_any(value: str, patterns: list[str]) -> bool:
    if not patterns:
        return True
    return any(fnmatch(value, pat) for pat in patterns)


def _filter_tasks(tasks: list[TaskSpec], only: list[str], skip: list[str]) -> list[TaskSpec]:
    filtered: list[TaskSpec] = []
    for task in tasks:
        if not _matches_any(task.label, only):
            continue
        if skip and _matches_any(task.label, skip):
            continue
        filtered.append(task)
    return filtered


def _run_check_task(repo_root: Path, check_id: str) -> tuple[bool, str]:
    check = get_check(check_id)
    if check is None:
        return False, f"why=unknown check id `{check_id}`; how_to_fix=use `atlasctl check list --json` to discover valid ids; evidence=n/a"
    failed, results = run_function_checks(repo_root, [check])
    result = results[0]
    if failed == 0:
        return True, ""
    details = result.errors[:2] + result.warnings[:2]
    reason = "; ".join(details) if details else "check failed"
    evidence = ", ".join(result.evidence_paths) if result.evidence_paths else "n/a"
    return False, f"why={reason}; how_to_fix={check.fix_hint}; evidence={evidence}"


def _run_cmd_task(repo_root: Path, cmd_spec: str, show_output: bool) -> tuple[bool, str]:
    parts = shlex.split(cmd_spec)
    if not parts:
        return False, "empty cmd task"
    if parts[0] == "atlasctl":
        cmd = [sys.executable, "-m", "atlasctl.cli", *parts[1:]]
    else:
        cmd = parts
    env = os.environ.copy()
    src_path = str(repo_root / "packages/atlasctl/src")
    existing = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = f"{src_path}:{existing}" if existing else src_path
    proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False, env=env)
    if show_output and (proc.stdout or "").strip():
        print(proc.stdout.rstrip())
    if show_output and (proc.stderr or "").strip():
        print(proc.stderr.rstrip(), file=sys.stderr)
    if proc.returncode == 0:
        return True, ""
    line = ((proc.stderr or "") + "\n" + (proc.stdout or "")).strip().splitlines()
    reason = line[0] if line else f"command failed with exit {proc.returncode}"
    return False, f"why={reason}; how_to_fix=run the command directly and resolve failures; evidence=n/a"


def _run_schema_task(repo_root: Path, schema_spec: str) -> tuple[bool, str]:
    if "@" in schema_spec:
        schema_name, file_path = schema_spec.split("@", 1)
        path = (repo_root / file_path.strip()).resolve()
        if not path.exists():
            return False, f"why=missing payload file `{file_path.strip()}`; how_to_fix=generate or provide payload file path; evidence=n/a"
        validate_file(schema_name.strip(), path)
        return True, ""
    schema_path_for(schema_spec.strip())
    return True, ""


def _execute_task(repo_root: Path, task: TaskSpec, show_output: bool) -> tuple[str, str]:
    try:
        if task.kind == "check":
            ok, detail = _run_check_task(repo_root, task.value)
        elif task.kind == "cmd":
            ok, detail = _run_cmd_task(repo_root, task.value, show_output=show_output)
        else:
            ok, detail = _run_schema_task(repo_root, task.value)
    except Exception as exc:  # pragma: no cover
        return "fail", str(exc)
    return ("pass", "") if ok else ("fail", detail)


def _expand_check_tag(task: TaskSpec) -> list[TaskSpec]:
    matched = [check for check in list_checks() if task.value in check_tags(check)]
    return [TaskSpec(suite=task.suite, kind="check", value=check.check_id) for check in matched]


def _write_junit(path: Path, suite_name: str, results: list[dict[str, object]]) -> None:
    total = len(results)
    failed = sum(1 for row in results if row["status"] == "fail")
    node = Element("testsuite", name=f"atlasctl-suite-{suite_name}", tests=str(total), failures=str(failed))
    for row in results:
        case = SubElement(node, "testcase", classname=f"atlasctl.suite.{suite_name}", name=str(row["label"]))
        if row["status"] == "fail":
            failure = SubElement(case, "failure", message=str(row.get("detail", "failed")))
            failure.text = str(row.get("detail", "failed"))
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(tostring(node, encoding="unicode"), encoding="utf-8")


def _suite_task_set(suites: dict[str, SuiteSpec], suite_name: str) -> set[tuple[str, str]]:
    tasks = expand_suite(suites, suite_name)
    entries: set[tuple[str, str]] = set()
    for task in tasks:
        if task.kind == "check-tag":
            for expanded in _expand_check_tag(task):
                entries.add((expanded.kind, expanded.value))
            continue
        entries.add((task.kind, task.value))
    return entries


def _atlasctl_subcommand_from_cmd_spec(cmd_spec: str) -> str | None:
    parts = shlex.split(cmd_spec)
    if len(parts) >= 2 and parts[0] == "atlasctl":
        return parts[1].strip()
    if len(parts) >= 4 and parts[1] == "-m" and parts[2] in {"atlasctl", "atlasctl.cli"}:
        return parts[3].strip()
    return None


def suite_inventory_violations(suites: dict[str, SuiteSpec]) -> list[str]:
    errors: list[str] = []
    all_check_ids = {check.check_id for check in list_checks()}
    task_sets = {name: _suite_task_set(suites, name) for name in suites}
    covered_checks = {value for entries in task_sets.values() for kind, value in entries if kind == "check"}
    orphan = sorted(all_check_ids - covered_checks)
    for check_id in orphan:
        check = get_check(check_id)
        if check is None:
            continue
        tags = set(check_tags(check))
        if "internal" in tags or "internal-only" in tags:
            continue
        errors.append(
            f"orphan check not assigned to any suite: {check_id}; every new check must declare suite membership or be internal-only"
        )

    required_suite = suites.get("required")
    if required_suite and required_suite.complete:
        required_suite_checks = {value for kind, value in task_sets.get("required", set()) if kind == "check"}
        required_tagged_checks = {check.check_id for check in list_checks() if "required" in check_tags(check)}
        missing = sorted(required_tagged_checks - required_suite_checks)
        for check_id in missing:
            errors.append(f"required complete policy violation: missing required check `{check_id}`")

    command_catalog = {spec.name for spec in command_registry()}
    for suite_name, entries in sorted(task_sets.items()):
        pytest_like = [value for kind, value in entries if kind == "cmd" and ("pytest" in value or "atlasctl test run" in value)]
        if suite_name == "ci":
            if any("pytest" in value and "atlasctl test run" not in value for value in pytest_like):
                errors.append("ci suite must not invoke pytest directly; use `atlasctl test run unit` once")
            atlasctl_test_runs = [value for value in pytest_like if "atlasctl test run" in value]
            if len(atlasctl_test_runs) != 1:
                errors.append(f"ci suite must include exactly one atlasctl test run entry; found {len(atlasctl_test_runs)}")
        for kind, value in sorted(entries):
            if kind != "cmd":
                continue
            subcommand = _atlasctl_subcommand_from_cmd_spec(value)
            if subcommand is None:
                continue
            if subcommand not in command_catalog:
                errors.append(
                    f"orphan command not in command catalog: suite `{suite_name}` references `atlasctl {subcommand}`"
                )
    return errors


def _suite_manifest_docs_violations(repo_root: Path, manifests: dict[str, SuiteManifest]) -> list[str]:
    path = repo_root / "packages/atlasctl/docs/control-plane/suites.md"
    if not path.exists():
        return [f"missing suite docs file: {path.relative_to(repo_root).as_posix()}"]
    text = path.read_text(encoding="utf-8", errors="ignore")
    errors: list[str] = []
    for name in sorted(manifests):
        marker = f"- `{name}`"
        if marker not in text:
            errors.append(f"suite docs drift: missing suite marker `{marker}` in packages/atlasctl/docs/control-plane/suites.md")
    return errors


def _suite_markers_docs_violations(repo_root: Path) -> list[str]:
    path = repo_root / "packages/atlasctl/docs/control-plane/suite-markers.md"
    if not path.exists():
        return [f"missing suite markers docs file: {path.relative_to(repo_root).as_posix()}"]
    text = path.read_text(encoding="utf-8")
    errors: list[str] = []
    for marker in ("required", "ci", "local", "slow"):
        if f"`{marker}`" not in text:
            errors.append(f"suite markers docs drift: missing marker `{marker}` in packages/atlasctl/docs/control-plane/suite-markers.md")
    return errors


def _first_class_suite_coverage_violations(manifests: dict[str, SuiteManifest]) -> list[str]:
    covered = set(manifests["all"].check_ids)
    errors: list[str] = []
    for check in list_checks():
        if check.check_id in covered:
            continue
        tags = set(check_tags(check))
        if {"internal", "internal-only"}.intersection(tags):
            continue
        errors.append(
            f"check missing suite membership: {check.check_id}; add it to a suite or mark as internal-only"
        )
    return errors


def _first_class_effect_policy_violations(manifest: SuiteManifest) -> list[str]:
    allowed = set(manifest.default_effects)
    errors: list[str] = []
    for check_id in manifest.check_ids:
        check = get_check(check_id)
        if check is None:
            errors.append(f"suite `{manifest.name}` references unknown check id: {check_id}")
            continue
        unknown = sorted(set(check.effects) - allowed)
        if unknown:
            errors.append(
                f"suite `{manifest.name}` check `{check_id}` effects {unknown} violate suite default effects {sorted(allowed)}"
            )
    return errors


def _suite_legacy_check_violations(manifests: dict[str, SuiteManifest]) -> list[str]:
    import inspect

    errors: list[str] = []
    for manifest in sorted(manifests.values(), key=lambda item: item.name):
        for check_id in manifest.check_ids:
            check = get_check(check_id)
            if check is None:
                continue
            source = inspect.getsourcefile(check.fn) or ""
            if "/legacy/" in source.replace("\\", "/"):
                errors.append(f"suite `{manifest.name}` must not include legacy-path checks: {check_id}")
    return errors


def _run_first_class_suite(ctx: RunContext, manifest: SuiteManifest, as_json: bool, list_only: bool, target_dir: str | None) -> int:
    if list_only:
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "suite": manifest.name,
            "required_env": list(manifest.required_env),
            "markers": list(manifest.markers),
            "internal": manifest.internal,
            "default_effects": list(manifest.default_effects),
            "time_budget_ms": manifest.time_budget_ms,
            "check_ids": list(manifest.check_ids),
            "total_count": len(manifest.check_ids),
        }
        print(dumps_json(payload, pretty=not as_json))
        return 0

    checks = [check for check_id in manifest.check_ids if (check := get_check(check_id)) is not None]
    target = Path(target_dir) if target_dir else (ctx.repo_root / "artifacts/isolate" / ctx.run_id / "atlasctl-suite")
    target.mkdir(parents=True, exist_ok=True)
    started = time.perf_counter()
    failed, results = run_function_checks(ctx.repo_root, checks)
    total_duration_ms = int((time.perf_counter() - started) * 1000)
    budget_exceeded = total_duration_ms > manifest.time_budget_ms
    if budget_exceeded:
        failed += 1
    rows = [
        {
            "index": idx,
            "suite": manifest.name,
            "label": f"check {row.id}",
            "kind": "check",
            "value": row.id,
            "status": row.status,
            "duration_ms": int(row.metrics.get("duration_ms", 0)),
            "hint": row.fix_hint,
            "detail": "; ".join([*row.errors[:2], *row.warnings[:2]]),
        }
        for idx, row in enumerate(results, start=1)
    ]
    summary = {
        "passed": sum(1 for row in rows if row["status"] == "pass"),
        "failed": sum(1 for row in rows if row["status"] == "fail"),
        "total": len(rows),
        "pass_rate": (sum(1 for row in rows if row["status"] == "pass") / len(rows)) if rows else 1.0,
        "duration_ms": total_duration_ms,
        "time_budget_ms": manifest.time_budget_ms,
        "budget_status": "fail" if budget_exceeded else "pass",
    }
    payload = {
        "schema_version": 1,
        "schema_name": SUITE_RUN,
        "tool": "atlasctl",
        "status": "ok" if failed == 0 else "error",
        "suite": manifest.name,
        "markers": list(manifest.markers),
        "internal": manifest.internal,
        "required_env": list(manifest.required_env),
        "default_effects": list(manifest.default_effects),
        "summary": summary,
        "results": rows,
        "target_dir": target.as_posix(),
    }
    validate_self(SUITE_RUN, payload)
    (target / "results.json").write_text(dumps_json(payload, pretty=True) + "\n", encoding="utf-8")
    print(dumps_json(payload, pretty=not as_json))
    return 0 if failed == 0 else 1


def configure_suite_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    from .parser import configure_suite_parser as configure

    configure(sub)

def run_suite_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    from .run import run_suite_command as run

    return run(ctx, ns)


def configure_suite_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    from .parser import configure_suite_parser as configure

    configure(sub)
