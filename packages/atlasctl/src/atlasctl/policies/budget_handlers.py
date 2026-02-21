from __future__ import annotations

import json
from pathlib import Path
from typing import Callable

from .culprits import biggest_dirs, biggest_files, budget_suite, explain_budgets, evaluate_metric, render_table_text, render_text
from .dir_entry_budgets import evaluate_budget, render_budget_text, report_budgets
from .errors import BudgetMetricError


def handle_budget_command(ns, repo: Path, write_out_file: Callable[[Path, str, str], None]) -> int | None:
    if ns.policies_cmd == "check-dir-entry-budgets":
        payload = evaluate_budget(repo, "entries-per-dir", fail_on_warn=bool(getattr(ns, "fail_on_warn", False)), top_n=int(getattr(ns, "top", 10)))
        output = json.dumps(payload, sort_keys=True) if bool(getattr(ns, "json", False)) else render_budget_text(payload, print_culprits=bool(getattr(ns, "print_culprits", False)))
        write_out_file(repo, str(getattr(ns, "out_file", "")), output)
        print(output)
        return 0 if payload["status"] == "ok" else 1
    if ns.policies_cmd == "check-py-files-per-dir":
        payload = evaluate_budget(repo, "py-files-per-dir", fail_on_warn=bool(getattr(ns, "fail_on_warn", False)), top_n=int(getattr(ns, "top", 10)))
        output = json.dumps(payload, sort_keys=True) if bool(getattr(ns, "json", False)) else render_budget_text(payload, print_culprits=bool(getattr(ns, "print_culprits", False)))
        write_out_file(repo, str(getattr(ns, "out_file", "")), output)
        print(output)
        return 0 if payload["status"] == "ok" else 1
    if ns.policies_cmd == "report-budgets":
        payload = report_budgets(repo, by_domain=bool(getattr(ns, "by_domain", False)))
        output = json.dumps(payload, sort_keys=True) if bool(getattr(ns, "json", False)) else json.dumps(payload, indent=2, sort_keys=True)
        write_out_file(repo, str(getattr(ns, "out_file", "")), output)
        print(output)
        return 0 if payload["status"] == "ok" else 1
    if ns.policies_cmd == "culprits":
        return _run_metric(repo, ns.culprits_metric, ns.report, str(getattr(ns, "out_file", "")), write_out_file)
    if ns.policies_cmd in {"culprits-files-per-dir", "culprits-modules-per-dir", "culprits-loc-per-dir"}:
        metric = {
            "culprits-files-per-dir": "files-per-dir",
            "culprits-modules-per-dir": "modules-per-dir",
            "culprits-loc-per-dir": "loc-per-dir",
        }[ns.policies_cmd]
        return _run_metric(repo, metric, ns.report, str(getattr(ns, "out_file", "")), write_out_file)
    if ns.policies_cmd == "culprits-largest-files":
        payload = biggest_files(repo, limit=ns.limit)
        output = json.dumps(payload, sort_keys=True) if ns.report == "json" else render_text(payload)
        write_out_file(repo, str(getattr(ns, "out_file", "")), output)
        print(output)
        return 0 if payload["status"] == "ok" else 1
    if ns.policies_cmd == "culprits-biggest-files":
        payload = biggest_files(repo, limit=ns.limit)
        output = json.dumps(payload, sort_keys=True) if ns.report == "json" else render_table_text(payload, "biggest-files")
        write_out_file(repo, str(getattr(ns, "out_file", "")), output)
        print(output)
        return 0
    if ns.policies_cmd == "culprits-biggest-dirs":
        payload = biggest_dirs(repo, limit=ns.limit)
        output = json.dumps(payload, sort_keys=True) if ns.report == "json" else render_table_text(payload, "biggest-dirs")
        write_out_file(repo, str(getattr(ns, "out_file", "")), output)
        print(output)
        return 0
    if ns.policies_cmd == "culprits-suite":
        payload = budget_suite(repo)
        output = json.dumps(payload, sort_keys=True) if ns.report == "json" else "\n\n".join(
            render_text(report) for report in payload["reports"]
        )
        write_out_file(repo, str(getattr(ns, "out_file", "")), output)
        print(output)
        return 0 if payload["status"] == "ok" else 1
    if ns.policies_cmd == "explain" and ns.subject == "budgets":
        payload = explain_budgets(repo)
        print(json.dumps(payload, sort_keys=True) if ns.report == "json" else json.dumps(payload, indent=2, sort_keys=True))
        return 0
    return None


def _run_metric(repo: Path, metric: str, report: str, out_file: str, write_out_file: Callable[[Path, str, str], None]) -> int:
    try:
        payload = evaluate_metric(repo, metric)
    except BudgetMetricError as exc:
        print(f"{exc.code}: {exc.message}")
        return 2
    output = json.dumps(payload, sort_keys=True) if report == "json" else render_text(payload)
    write_out_file(repo, out_file, output)
    print(output)
    return 0 if payload["status"] == "ok" else 1
