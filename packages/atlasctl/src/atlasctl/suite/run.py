from __future__ import annotations

import argparse
import json
import os
import sys
import time
from pathlib import Path

from . import command as impl
from ..core.runtime.paths import write_text_file

def run_suite_command(ctx, ns: argparse.Namespace) -> int:
    as_json = ctx.output_format == "json" or bool(getattr(ns, "json", False))
    first_class = impl.load_first_class_suites()
    if not getattr(ns, "suite_cmd", None) and bool(getattr(ns, "list_suites", False)):
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "items": sorted([*first_class.keys(), *impl.load_suites(ctx.repo_root)[1].keys()]),
        }
        if as_json:
            print(impl.dumps_json(payload, pretty=False))
        else:
            for item in payload["items"]:
                print(item)
        return 0
    if ns.suite_cmd in first_class:
        manifest = first_class[ns.suite_cmd]
        if manifest.internal and os.environ.get("ATLASCTL_INTERNAL") != "1":
            msg = "internal suite execution requires ATLASCTL_INTERNAL=1"
            print(impl.dumps_json({"schema_version": 1, "tool": "atlasctl", "status": "error", "error": msg}, pretty=False) if as_json else msg)
            return impl.ERR_USER
        return impl._run_first_class_suite(
            ctx,
            manifest,
            as_json=as_json,
            list_only=bool(getattr(ns, "list", False) or getattr(ns, "dry_run", False)),
            target_dir=getattr(ns, "target_dir", None),
        )
    default_suite, suites = impl.load_suites(ctx.repo_root)
    if ns.suite_cmd == "explain":
        suite_name = ns.name or default_suite
        if suite_name in first_class:
            manifest = first_class[suite_name]
            lines = [
                f"suite {suite_name} rationale",
                f"- kind: first-class",
                f"- markers: {', '.join(manifest.markers) or 'none'}",
                f"- required_env: {', '.join(manifest.required_env) or 'none'}",
                f"- default_effects: {', '.join(manifest.default_effects) or 'none'}",
                f"- check_count: {len(manifest.check_ids)}",
            ]
            for check_id in manifest.check_ids:
                lines.append(f"- check:{check_id}: selected via suite registry markers/groups")
            text = "\n".join(lines)
            print(
                impl.dumps_json(
                    {"schema_version": 1, "tool": "atlasctl", "status": "ok", "suite": suite_name, "explain": lines},
                    pretty=False,
                )
                if as_json
                else text
            )
            return 0
        expanded = impl.expand_suite(suites, suite_name)
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
        print(impl.dumps_json({"schema_version": 1, "tool": "atlasctl", "status": "ok", "suite": suite_name, "explain": lines}, pretty=False) if as_json else text)
        return 0
    if ns.suite_cmd == "artifacts":
        run_id = ns.run_id or ctx.run_id
        target_dir = ctx.repo_root / "artifacts/isolate" / run_id / "atlasctl-suite"
        payload = {"schema_version": 1, "tool": "atlasctl", "status": "ok", "run_id": run_id, "target_dir": target_dir.as_posix(), "results_file": (target_dir / "results.json").as_posix()}
        print(impl.dumps_json(payload, pretty=False) if as_json else f"{payload['results_file']}")
        return 0
    if ns.suite_cmd == "doctor":
        run_id = ns.run_id or ctx.run_id
        results_file = ctx.repo_root / "artifacts/isolate" / run_id / "atlasctl-suite" / "results.json"
        if not results_file.exists():
            msg = f"no suite results for run_id `{run_id}`"
            print(impl.dumps_json({"schema_version": 1, "tool": "atlasctl", "status": "error", "error": msg}, pretty=False) if as_json else msg)
            return impl.ERR_USER
        payload = json.loads(results_file.read_text(encoding="utf-8"))
        failed = [row for row in payload.get("results", []) if row.get("status") == "fail"]
        advice = [f"fix failing task: {row.get('label')}" for row in failed[:10]]
        out = {"schema_version": 1, "tool": "atlasctl", "status": "ok", "run_id": run_id, "failed_count": len(failed), "advice": advice}
        print(impl.dumps_json(out, pretty=False) if as_json else "\n".join(advice or ["no failed tasks"]))
        return 0
    if ns.suite_cmd == "diff":
        left = ctx.repo_root / "artifacts/isolate" / ns.run1 / "atlasctl-suite" / "results.json"
        right = ctx.repo_root / "artifacts/isolate" / ns.run2 / "atlasctl-suite" / "results.json"
        if not left.exists() or not right.exists():
            missing = left if not left.exists() else right
            msg = f"missing suite results file: {missing.as_posix()}"
            print(impl.dumps_json({"schema_version": 1, "tool": "atlasctl", "status": "error", "error": msg}, pretty=False) if as_json else msg)
            return impl.ERR_USER
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
        print(impl.dumps_json(payload, pretty=False) if as_json else f"new_failures={len(payload['new_failures'])} fixed={len(payload['fixed'])}")
        return 0
    if ns.suite_cmd == "check":
        errors = impl.suite_inventory_violations(suites)
        errors.extend(impl._first_class_suite_coverage_violations(first_class))
        errors.extend(impl._suite_manifest_docs_violations(ctx.repo_root, first_class))
        errors.extend(impl._suite_markers_docs_violations(ctx.repo_root))
        errors.extend(impl._suite_legacy_check_violations(first_class))
        for manifest in first_class.values():
            errors.extend(impl._first_class_effect_policy_violations(manifest))
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok" if not errors else "error",
            "errors": errors,
        }
        if as_json:
            print(impl.dumps_json(payload, pretty=False))
        else:
            print("suite inventory: ok" if not errors else "suite inventory: fail")
            for err in errors[:40]:
                print(f"- {err}")
        return 0 if not errors else impl.ERR_USER
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
                print(impl.dumps_json(payload, pretty=False))
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
        print(impl.dumps_json(payload, pretty=not as_json))
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
            "unassigned": sorted([check.check_id for check in impl.list_checks() if check.check_id not in coverage]),
            "coverage": {check_id: sorted(names) for check_id, names in sorted(coverage.items())},
        }
        if as_json:
            print(impl.dumps_json(payload, pretty=False))
        else:
            for check_id, suites_for_check in payload["coverage"].items():
                print(f"{check_id}: {', '.join(suites_for_check)}")
            if payload["unassigned"]:
                print("unassigned:")
                for check_id in payload["unassigned"]:
                    print(f"- {check_id}")
        return 0 if not payload["unassigned"] else impl.ERR_USER

    suite_name = ns.name or default_suite
    if ns.suite_cmd == "run" and suite_name in first_class:
        manifest = first_class[suite_name]
        if manifest.internal and os.environ.get("ATLASCTL_INTERNAL") != "1":
            msg = "internal suite execution requires ATLASCTL_INTERNAL=1"
            print(impl.dumps_json({"schema_version": 1, "tool": "atlasctl", "status": "error", "error": msg}, pretty=False) if as_json else msg)
            return impl.ERR_USER
        return impl._run_first_class_suite(
            ctx,
            manifest,
            as_json=as_json,
            list_only=bool(getattr(ns, "list", False) or getattr(ns, "dry_run", False)),
            target_dir=getattr(ns, "target_dir", None),
        )
    expanded_raw = impl.expand_suite(suites, suite_name)
    expanded: list[impl.TaskSpec] = []
    for task in expanded_raw:
        if task.kind == "check-tag":
            expanded.extend(impl._expand_check_tag(task))
        else:
            expanded.append(task)
    selected = impl._filter_tasks(expanded, only=ns.only or [], skip=ns.skip or [])
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
        print(impl.dumps_json(payload, pretty=not as_json))
        return 0

    target_dir = Path(ns.target_dir) if ns.target_dir else (ctx.repo_root / "artifacts/isolate" / ctx.run_id / "atlasctl-suite")
    target_dir.mkdir(parents=True, exist_ok=True)

    start = time.perf_counter()
    results: list[dict[str, object]] = []
    maxfail = max(0, int(getattr(ns, "maxfail", 0) or 0))
    fail_seen = 0
    if ctx.verbose and not ctx.quiet:
        impl.log_event(ctx, "info", "suite", "start", suite=suite_name, total=len(selected))
    for idx, task in enumerate(selected, start=1):
        item_start = time.perf_counter()
        status, detail = impl._execute_task(ctx.repo_root, task, show_output=bool(ns.show_output))
        duration_ms = int((time.perf_counter() - item_start) * 1000)
        status_upper = "PASS" if status == "pass" else "FAIL"
        line = f"{status_upper} {task.label} ({duration_ms}ms)"
        if detail and status == "fail":
            line = f"{line} :: {detail}"
        if ctx.verbose:
            impl.log_event(
                ctx, "info" if status == "pass" else "error", "suite", "item", label=task.label, status=status, duration_ms=duration_ms
            )
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
        if status == "fail":
            fail_seen += 1
        if status == "fail" and ns.fail_fast:
            break
        if maxfail > 0 and fail_seen >= maxfail and not ns.keep_going:
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
        impl._write_junit(ctx.repo_root / ns.junit, suite_name, results)

    payload = {
        "schema_name": impl.SUITE_RUN,
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok" if failed == 0 else "error",
        "suite": suite_name,
        "summary": summary,
        "slow_threshold_ms": slow_threshold_ms,
        "slow_checks": slow_rows,
        "results": results,
        "target_dir": target_dir.as_posix(),
        "execution": "fail-fast" if ns.fail_fast else ("maxfail" if maxfail > 0 else "keep-going"),
        "maxfail": maxfail,
    }
    impl.validate_self(impl.SUITE_RUN, payload)
    write_text_file(target_dir / "results.json", impl.dumps_json(payload, pretty=True) + "\n")
    if getattr(ns, "slow_report", None):
        write_text_file(
            ctx.repo_root / ns.slow_report,
            impl.dumps_json(
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
        )
    if getattr(ns, "profile", False):
        profile_path = target_dir / "profile.json"
        write_text_file(
            profile_path,
            impl.dumps_json(
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
        )
    impl.emit_telemetry(
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
        print(impl.dumps_json(payload, pretty=False))
    return 0 if failed == 0 else impl.ERR_USER
