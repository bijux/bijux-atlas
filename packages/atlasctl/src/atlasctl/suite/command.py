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

from ..checks.execution import run_function_checks
from ..checks.registry import check_tags, get_check, list_checks
from ..contracts.catalog import schema_path_for
from ..contracts.validate import validate_file
from ..core.context import RunContext
from ..core.serialize import dumps_json
from ..errors import ScriptError
from ..exit_codes import ERR_CONFIG

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
    default = str(root.get("default", "refgrade")).strip() or "refgrade"
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
        return False, f"unknown check id `{check_id}`"
    failed, results = run_function_checks(repo_root, [check])
    result = results[0]
    if failed == 0:
        return True, ""
    details = result.errors[:2] + result.warnings[:2]
    return False, "; ".join(details) if details else "check failed"


def _run_cmd_task(repo_root: Path, cmd_spec: str) -> tuple[bool, str]:
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
    if proc.returncode == 0:
        return True, ""
    line = ((proc.stderr or "") + "\n" + (proc.stdout or "")).strip().splitlines()
    return False, line[0] if line else f"command failed with exit {proc.returncode}"


def _run_schema_task(repo_root: Path, schema_spec: str) -> tuple[bool, str]:
    if "@" in schema_spec:
        schema_name, file_path = schema_spec.split("@", 1)
        path = (repo_root / file_path.strip()).resolve()
        if not path.exists():
            return False, f"missing payload file `{file_path.strip()}`"
        validate_file(schema_name.strip(), path)
        return True, ""
    schema_path_for(schema_spec.strip())
    return True, ""


def _execute_task(repo_root: Path, task: TaskSpec) -> tuple[str, str]:
    try:
        if task.kind == "check":
            ok, detail = _run_check_task(repo_root, task.value)
        elif task.kind == "cmd":
            ok, detail = _run_cmd_task(repo_root, task.value)
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
        if "experimental" in check_tags(check):
            continue
        errors.append(f"orphan check not assigned to any suite: {check_id}")

    refgrade = suites.get("refgrade")
    if refgrade and refgrade.complete:
        refgrade_checks = {value for kind, value in task_sets.get("refgrade", set()) if kind == "check"}
        required = {check.check_id for check in list_checks() if "refgrade_required" in check_tags(check)}
        missing = sorted(required - refgrade_checks)
        for check_id in missing:
            errors.append(f"refgrade complete policy violation: missing refgrade_required check `{check_id}`")
    return errors


def run_suite_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    as_json = ctx.output_format == "json" or bool(getattr(ns, "json", False))
    default_suite, suites = load_suites(ctx.repo_root)
    if ns.suite_cmd == "check":
        errors = suite_inventory_violations(suites)
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok" if not errors else "error",
            "errors": errors,
        }
        if as_json:
            print(dumps_json(payload, pretty=False))
        else:
            print("suite inventory: ok" if not errors else "suite inventory: fail")
            for err in errors[:40]:
                print(f"- {err}")
        return 0 if not errors else 1
    if ns.suite_cmd == "list":
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "default": default_suite,
            "suites": [
                {
                    "name": spec.name,
                    "includes": list(spec.includes),
                    "item_count": len(spec.items),
                    "items": list(spec.items),
                    "complete": spec.complete,
                }
                for spec in sorted(suites.values(), key=lambda item: item.name)
            ],
        }
        print(dumps_json(payload, pretty=not as_json))
        return 0

    suite_name = ns.name or default_suite
    expanded_raw = expand_suite(suites, suite_name)
    expanded: list[TaskSpec] = []
    for task in expanded_raw:
        if task.kind == "check-tag":
            expanded.extend(_expand_check_tag(task))
        else:
            expanded.append(task)
    selected = _filter_tasks(expanded, only=ns.only or [], skip=ns.skip or [])
    if ns.list:
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "suite": suite_name,
            "total_count": len(selected),
            "tasks": [f"{task.kind}:{task.value}" for task in selected],
        }
        print(dumps_json(payload, pretty=not as_json))
        return 0

    target_dir = Path(ns.target_dir) if ns.target_dir else (ctx.repo_root / "artifacts/isolate" / ctx.run_id / "atlasctl-suite")
    target_dir.mkdir(parents=True, exist_ok=True)

    start = time.perf_counter()
    results: list[dict[str, object]] = []
    for idx, task in enumerate(selected, start=1):
        item_start = time.perf_counter()
        status, detail = _execute_task(ctx.repo_root, task)
        duration_ms = int((time.perf_counter() - item_start) * 1000)
        line = f"({idx}/{len(selected)}) {'PASS' if status == 'pass' else 'FAIL'} {task.label}"
        if detail and status == "fail":
            line = f"{line} :: {detail}"
        if not as_json:
            print(line)
        results.append(
            {
                "index": idx,
                "suite": task.suite,
                "label": task.label,
                "kind": task.kind,
                "value": task.value,
                "status": status,
                "detail": detail,
                "duration_ms": duration_ms,
            }
        )
        if status == "fail" and ns.fail_fast:
            break

    total_duration_ms = int((time.perf_counter() - start) * 1000)
    failed = sum(1 for row in results if row["status"] == "fail")
    passed = sum(1 for row in results if row["status"] == "pass")
    skipped = len(selected) - len(results)
    summary = {
        "passed": passed,
        "failed": failed,
        "skipped": skipped,
        "duration_ms": total_duration_ms,
    }
    if not as_json:
        print(f"summary: passed={passed} failed={failed} skipped={skipped} duration_ms={total_duration_ms}")

    if ns.junit:
        _write_junit(ctx.repo_root / ns.junit, suite_name, results)

    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok" if failed == 0 else "error",
        "suite": suite_name,
        "summary": summary,
        "results": results,
        "target_dir": target_dir.as_posix(),
    }
    (target_dir / "results.json").write_text(dumps_json(payload, pretty=True) + "\n", encoding="utf-8")
    if as_json:
        print(dumps_json(payload, pretty=False))
    return 0 if failed == 0 else 1


def configure_suite_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("suite", help="run atlasctl named suites")
    suite_sub = parser.add_subparsers(dest="suite_cmd", required=True)

    suite_list = suite_sub.add_parser("list", help="list configured suites")
    suite_list.add_argument("--json", action="store_true")
    suite_check = suite_sub.add_parser("check", help="validate suite inventory/tag coverage policy")
    suite_check.add_argument("--json", action="store_true")

    run = suite_sub.add_parser("run", help="run a configured suite")
    run.add_argument("name", nargs="?", help="suite name (defaults to configured default)")
    run.add_argument("--json", action="store_true", help="emit machine-readable results")
    run.add_argument("--junit", help="write junit xml report")
    run.add_argument("--list", action="store_true", help="list expanded tasks without running")
    run.add_argument("--only", action="append", default=[], help="glob pattern to include task labels")
    run.add_argument("--skip", action="append", default=[], help="glob pattern to skip task labels")
    run.add_argument("--target-dir", help="suite output directory")
    group = run.add_mutually_exclusive_group()
    group.add_argument("--fail-fast", action="store_true", help="stop at first failure")
    group.add_argument("--keep-going", action="store_true", help="continue through all tasks (default)")
