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
from ..cli.surface_registry import command_registry
from ..contracts.catalog import schema_path_for
from ..contracts.ids import SUITE_RUN
from ..contracts.validate_self import validate_self
from ..contracts.validate import validate_file
from ..core.context import RunContext
from ..core.telemetry import emit_telemetry
from ..core.logging import log_event
from ..core.serialize import dumps_json
from ..errors import ScriptError
from ..exit_codes import ERR_CONFIG, ERR_USER
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

    refgrade = suites.get("refgrade")
    if refgrade and refgrade.complete:
        refgrade_checks = {value for kind, value in task_sets.get("refgrade", set()) if kind == "check"}
        required = {check.check_id for check in list_checks() if "refgrade_required" in check_tags(check)}
        missing = sorted(required - refgrade_checks)
        for check_id in missing:
            errors.append(f"refgrade complete policy violation: missing refgrade_required check `{check_id}`")

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
    for marker in ("refgrade", "ci", "local", "slow"):
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


def run_suite_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    as_json = ctx.output_format == "json" or bool(getattr(ns, "json", False))
    first_class = load_first_class_suites()
    if not getattr(ns, "suite_cmd", None) and bool(getattr(ns, "list_suites", False)):
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "items": sorted([*first_class.keys(), *load_suites(ctx.repo_root)[1].keys()]),
        }
        if as_json:
            print(dumps_json(payload, pretty=False))
        else:
            for item in payload["items"]:
                print(item)
        return 0
    if ns.suite_cmd in first_class:
        manifest = first_class[ns.suite_cmd]
        if manifest.internal and os.environ.get("ATLASCTL_INTERNAL") != "1":
            msg = "internal suite execution requires ATLASCTL_INTERNAL=1"
            print(dumps_json({"schema_version": 1, "tool": "atlasctl", "status": "error", "error": msg}, pretty=False) if as_json else msg)
            return ERR_USER
        return _run_first_class_suite(
            ctx,
            manifest,
            as_json=as_json,
            list_only=bool(getattr(ns, "list", False) or getattr(ns, "dry_run", False)),
            target_dir=getattr(ns, "target_dir", None),
        )
    default_suite, suites = load_suites(ctx.repo_root)
    if ns.suite_cmd == "explain":
        suite_name = ns.name or default_suite
        expanded = expand_suite(suites, suite_name)
        lines = [f"suite {suite_name} rationale", f"- includes: {', '.join(suites[suite_name].includes) or 'none'}"]
        for task in expanded:
            if task.kind == "check":
                lines.append(f"- {task.kind}:{task.value}: registry check for policy/contract enforcement")
            elif task.kind == "check-tag":
                lines.append(f"- {task.kind}:{task.value}: expands to all checks tagged `{task.value}`")
            elif task.kind == "cmd":
                lines.append(f"- {task.kind}:{task.value}: command-level integration validation")
            else:
                lines.append(f"- {task.kind}:{task.value}: schema existence/payload validation")
        text = "\n".join(lines)
        print(dumps_json({"schema_version": 1, "tool": "atlasctl", "status": "ok", "suite": suite_name, "explain": lines}, pretty=False) if as_json else text)
        return 0
    if ns.suite_cmd == "artifacts":
        run_id = ns.run_id or ctx.run_id
        target_dir = ctx.repo_root / "artifacts/isolate" / run_id / "atlasctl-suite"
        payload = {"schema_version": 1, "tool": "atlasctl", "status": "ok", "run_id": run_id, "target_dir": target_dir.as_posix(), "results_file": (target_dir / "results.json").as_posix()}
        print(dumps_json(payload, pretty=False) if as_json else f"{payload['results_file']}")
        return 0
    if ns.suite_cmd == "doctor":
        run_id = ns.run_id or ctx.run_id
        results_file = ctx.repo_root / "artifacts/isolate" / run_id / "atlasctl-suite" / "results.json"
        if not results_file.exists():
            msg = f"no suite results for run_id `{run_id}`"
            print(dumps_json({"schema_version": 1, "tool": "atlasctl", "status": "error", "error": msg}, pretty=False) if as_json else msg)
            return ERR_USER
        payload = json.loads(results_file.read_text(encoding="utf-8"))
        failed = [row for row in payload.get("results", []) if row.get("status") == "fail"]
        advice = [f"fix failing task: {row.get('label')}" for row in failed[:10]]
        out = {"schema_version": 1, "tool": "atlasctl", "status": "ok", "run_id": run_id, "failed_count": len(failed), "advice": advice}
        print(dumps_json(out, pretty=False) if as_json else "\n".join(advice or ["no failed tasks"]))
        return 0
    if ns.suite_cmd == "diff":
        left = ctx.repo_root / "artifacts/isolate" / ns.run1 / "atlasctl-suite" / "results.json"
        right = ctx.repo_root / "artifacts/isolate" / ns.run2 / "atlasctl-suite" / "results.json"
        if not left.exists() or not right.exists():
            missing = left if not left.exists() else right
            msg = f"missing suite results file: {missing.as_posix()}"
            print(dumps_json({"schema_version": 1, "tool": "atlasctl", "status": "error", "error": msg}, pretty=False) if as_json else msg)
            return ERR_USER
        lhs = json.loads(left.read_text(encoding="utf-8"))
        rhs = json.loads(right.read_text(encoding="utf-8"))
        lf = {row["label"] for row in lhs.get("results", []) if row.get("status") == "fail"}
        rf = {row["label"] for row in rhs.get("results", []) if row.get("status") == "fail"}
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "run1": ns.run1,
            "run2": ns.run2,
            "new_failures": sorted(rf - lf),
            "fixed": sorted(lf - rf),
        }
        print(dumps_json(payload, pretty=False) if as_json else f"new_failures={len(payload['new_failures'])} fixed={len(payload['fixed'])}")
        return 0
    if ns.suite_cmd == "check":
        errors = suite_inventory_violations(suites)
        errors.extend(_first_class_suite_coverage_violations(first_class))
        errors.extend(_suite_manifest_docs_violations(ctx.repo_root, first_class))
        errors.extend(_suite_markers_docs_violations(ctx.repo_root))
        errors.extend(_suite_legacy_check_violations(first_class))
        for manifest in first_class.values():
            errors.extend(_first_class_effect_policy_violations(manifest))
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
        return 0 if not errors else ERR_USER
    if ns.suite_cmd == "list":
        if bool(getattr(ns, "by_group", False)):
            grouped: dict[str, list[str]] = {}
            for manifest in sorted(first_class.values(), key=lambda item: item.name):
                for marker in manifest.markers:
                    grouped.setdefault(marker, []).append(manifest.name)
            payload = {
                "schema_version": 1,
                "tool": "atlasctl",
                "status": "ok",
                "by_group": {k: sorted(set(v)) for k, v in sorted(grouped.items())},
            }
            if as_json:
                print(dumps_json(payload, pretty=False))
            else:
                for marker, names in payload["by_group"].items():
                    print(f"{marker}: {', '.join(names)}")
            return 0
        if not as_json:
            names = sorted({*first_class.keys(), *suites.keys()})
            for name in names:
                print(name)
            return 0
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "default": default_suite,
            "first_class_suites": [
                {
                    "name": manifest.name,
                    "required_env": list(manifest.required_env),
                    "default_effects": list(manifest.default_effects),
                    "time_budget_ms": manifest.time_budget_ms,
                    "check_count": len(manifest.check_ids),
                }
                for manifest in sorted(first_class.values(), key=lambda item: item.name)
            ],
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
    if ns.suite_cmd == "coverage":
        coverage: dict[str, list[str]] = {}
        for manifest in sorted(first_class.values(), key=lambda item: item.name):
            for check_id in manifest.check_ids:
                coverage.setdefault(check_id, []).append(manifest.name)
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "unassigned": sorted([check.check_id for check in list_checks() if check.check_id not in coverage]),
            "coverage": {check_id: sorted(names) for check_id, names in sorted(coverage.items())},
        }
        if as_json:
            print(dumps_json(payload, pretty=False))
        else:
            for check_id, suites_for_check in payload["coverage"].items():
                print(f"{check_id}: {', '.join(suites_for_check)}")
            if payload["unassigned"]:
                print("unassigned:")
                for check_id in payload["unassigned"]:
                    print(f"- {check_id}")
        return 0 if not payload["unassigned"] else ERR_USER

    suite_name = ns.name or default_suite
    if ns.suite_cmd == "run" and suite_name in first_class:
        manifest = first_class[suite_name]
        if manifest.internal and os.environ.get("ATLASCTL_INTERNAL") != "1":
            msg = "internal suite execution requires ATLASCTL_INTERNAL=1"
            print(dumps_json({"schema_version": 1, "tool": "atlasctl", "status": "error", "error": msg}, pretty=False) if as_json else msg)
            return ERR_USER
        return _run_first_class_suite(
            ctx,
            manifest,
            as_json=as_json,
            list_only=bool(getattr(ns, "list", False) or getattr(ns, "dry_run", False)),
            target_dir=getattr(ns, "target_dir", None),
        )
    expanded_raw = expand_suite(suites, suite_name)
    expanded: list[TaskSpec] = []
    for task in expanded_raw:
        if task.kind == "check-tag":
            expanded.extend(_expand_check_tag(task))
        else:
            expanded.append(task)
    selected = _filter_tasks(expanded, only=ns.only or [], skip=ns.skip or [])
    if ns.list or bool(getattr(ns, "dry_run", False)):
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "suite": suite_name,
            "mode": "dry-run" if bool(getattr(ns, "dry_run", False)) else "list",
            "total_count": len(selected),
            "tasks": [f"{task.kind}:{task.value}" for task in selected],
        }
        print(dumps_json(payload, pretty=not as_json))
        return 0

    target_dir = Path(ns.target_dir) if ns.target_dir else (ctx.repo_root / "artifacts/isolate" / ctx.run_id / "atlasctl-suite")
    target_dir.mkdir(parents=True, exist_ok=True)

    start = time.perf_counter()
    results: list[dict[str, object]] = []
    if ctx.verbose and not ctx.quiet:
        log_event(ctx, "info", "suite", "start", suite=suite_name, total=len(selected))
    for idx, task in enumerate(selected, start=1):
        item_start = time.perf_counter()
        status, detail = _execute_task(ctx.repo_root, task, show_output=bool(ns.show_output))
        duration_ms = int((time.perf_counter() - item_start) * 1000)
        status_upper = "PASS" if status == "pass" else "FAIL"
        line = f"{status_upper} {task.label} ({duration_ms}ms)"
        if detail and status == "fail":
            line = f"{line} :: {detail}"
        if ctx.verbose:
            log_event(ctx, "info" if status == "pass" else "error", "suite", "item", label=task.label, status=status, duration_ms=duration_ms)
        if not as_json and bool(getattr(ns, "pytest_q", False)):
            sys.stdout.write("." if status == "pass" else "F")
            sys.stdout.flush()
        elif not as_json and (not ctx.quiet or status == "fail"):
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
    slow_threshold_ms = max(1, int(getattr(ns, "slow_threshold_ms", 1000)))
    slow_rows = sorted(
        [row for row in results if int(row["duration_ms"]) >= slow_threshold_ms],
        key=lambda item: int(item["duration_ms"]),
        reverse=True,
    )
    if not as_json and bool(getattr(ns, "pytest_q", False)):
        seconds = total_duration_ms / 1000
        print()
        print(f"=== {passed} passed, {failed} failed, {skipped} skipped in {seconds:.2f}s ===")
    elif not as_json and not ctx.quiet:
        print(f"summary: passed={passed} failed={failed} skipped={skipped} total={len(results)} duration_ms={total_duration_ms}")

    if ns.junit:
        _write_junit(ctx.repo_root / ns.junit, suite_name, results)

    payload = {
        "schema_name": SUITE_RUN,
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok" if failed == 0 else "error",
        "suite": suite_name,
        "summary": summary,
        "slow_threshold_ms": slow_threshold_ms,
        "slow_checks": slow_rows,
        "results": results,
        "target_dir": target_dir.as_posix(),
    }
    validate_self(SUITE_RUN, payload)
    (target_dir / "results.json").write_text(dumps_json(payload, pretty=True) + "\n", encoding="utf-8")
    if getattr(ns, "slow_report", None):
        (ctx.repo_root / ns.slow_report).parent.mkdir(parents=True, exist_ok=True)
        (ctx.repo_root / ns.slow_report).write_text(
            dumps_json(
                {
                    "schema_version": 1,
                    "tool": "atlasctl",
                    "kind": "suite-slow-report",
                    "run_id": ctx.run_id,
                    "suite": suite_name,
                    "threshold_ms": slow_threshold_ms,
                    "slow_checks": slow_rows,
                    "summary": summary,
                },
                pretty=True,
            )
            + "\n",
            encoding="utf-8",
        )
    if getattr(ns, "profile", False):
        profile_path = target_dir / "profile.json"
        profile_path.write_text(
            dumps_json(
                {
                    "schema_version": 1,
                    "tool": "atlasctl",
                    "kind": "suite-profile",
                    "run_id": ctx.run_id,
                    "suite": suite_name,
                    "summary": summary,
                    "rows": results,
                },
                pretty=True,
            )
            + "\n",
            encoding="utf-8",
        )
    emit_telemetry(
        ctx,
        "suite.run",
        suite=suite_name,
        passed=passed,
        failed=failed,
        skipped=skipped,
        duration_ms=total_duration_ms,
        slow_checks=len(slow_rows),
    )
    if as_json:
        print(dumps_json(payload, pretty=False))
    return 0 if failed == 0 else ERR_USER


def configure_suite_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("suite", help="run atlasctl named suites")
    parser.add_argument("--list", dest="list_suites", action="store_true", help="list configured suites")
    parser.add_argument("--json", action="store_true", help="emit machine-readable JSON output")
    suite_sub = parser.add_subparsers(dest="suite_cmd", required=False)

    suite_list = suite_sub.add_parser("list", help="list configured suites")
    suite_list.add_argument("--json", action="store_true")
    suite_list.add_argument("--by-group", action="store_true", help="group suites by marker")
    coverage = suite_sub.add_parser("coverage", help="show check-to-suite coverage")
    coverage.add_argument("--json", action="store_true")
    suite_check = suite_sub.add_parser("check", help="validate suite inventory/tag coverage policy")
    suite_check.add_argument("--json", action="store_true")

    run = suite_sub.add_parser("run", help="run a configured suite")
    run.add_argument("name", nargs="?", help="suite name (defaults to configured default)")
    run.add_argument("--json", action="store_true", help="emit machine-readable results")
    run.add_argument("--junit", help="write junit xml report")
    run.add_argument("--list", action="store_true", help="list expanded tasks without running")
    run.add_argument("--dry-run", action="store_true", help="print tasks without executing")
    run.add_argument("--only", action="append", default=[], help="glob pattern to include task labels")
    run.add_argument("--skip", action="append", default=[], help="glob pattern to skip task labels")
    run.add_argument("--target-dir", help="suite output directory")
    run.add_argument("--show-output", action="store_true", help="stream command task output")
    run.add_argument("--pytest-q", action="store_true", help="pytest-style quiet progress and summary output")
    run.add_argument("--slow-threshold-ms", type=int, default=1000, help="threshold for slow suite items report")
    run.add_argument("--slow-report", help="write slow suite items report path")
    run.add_argument("--profile", action="store_true", help="emit suite performance profile artifact")
    group = run.add_mutually_exclusive_group()
    group.add_argument("--fail-fast", action="store_true", help="stop at first failure")
    group.add_argument("--keep-going", action="store_true", help="continue through all tasks (default)")
    explain = suite_sub.add_parser("explain", help="explain suite tasks and rationale")
    explain.add_argument("name", nargs="?")
    explain.add_argument("--json", action="store_true")
    doctor = suite_sub.add_parser("doctor", help="suggest next actions from a suite run")
    doctor.add_argument("--run-id", help="run id to inspect")
    doctor.add_argument("--json", action="store_true")
    artifacts = suite_sub.add_parser("artifacts", help="print suite artifact locations")
    artifacts.add_argument("--run-id", help="run id to inspect")
    artifacts.add_argument("--json", action="store_true")
    diff = suite_sub.add_parser("diff", help="diff two suite runs")
    diff.add_argument("run1")
    diff.add_argument("run2")
    diff.add_argument("--json", action="store_true")

    for suite_name in ("docs", "dev", "ops", "policies", "configs", "local", "slow", "refgrade", "ci", "refgrade_proof", "all"):
        sp = suite_sub.add_parser(suite_name, help=f"run first-class `{suite_name}` suite")
        sp.add_argument("--json", action="store_true", help="emit machine-readable results")
        sp.add_argument("--list", action="store_true", help="list suite checks only")
        sp.add_argument("--dry-run", action="store_true", help="print checks without executing")
        sp.add_argument("--target-dir", help="suite output directory")
    internal = suite_sub.add_parser("internal", help="run first-class `internal` suite (requires ATLASCTL_INTERNAL=1)")
    internal.add_argument("--json", action="store_true", help="emit machine-readable results")
    internal.add_argument("--list", action="store_true", help="list suite checks only")
    internal.add_argument("--dry-run", action="store_true", help="print checks without executing")
    internal.add_argument("--target-dir", help="suite output directory")
