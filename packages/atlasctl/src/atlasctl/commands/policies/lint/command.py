from __future__ import annotations

import argparse

from ....core.context import RunContext
from ...check.command import run_check_command

_LINT_DOMAIN_MAP = {
    "all": "",
    "ops": "ops",
    "make": "make",
    "docs": "docs",
    "configs": "configs",
}

def configure_lint_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("lint", help="run lint checks via check selector alias")
    p.add_argument("suite", choices=tuple(sorted(_LINT_DOMAIN_MAP.keys())))
    p.add_argument("--fail-fast", action="store_true")
    p.add_argument("--all", action="store_true", help="include slow lint checks")
    p.add_argument("--list-selected", action="store_true", help="print resolved lint checks and exit")
    p.add_argument("--json", action="store_true", help="emit JSON output")

def run_lint_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    mapped_domain = _LINT_DOMAIN_MAP[str(ns.suite)]
    alias_ns = argparse.Namespace(
        check_cmd="run",
        include_all=bool(getattr(ns, "all", False)),
        run_quiet=False,
        run_info=False,
        run_verbose=False,
        maxfail=0,
        max_failures=0,
        failfast=bool(getattr(ns, "fail_fast", False)),
        keep_going=not bool(getattr(ns, "fail_fast", False)),
        durations=0,
        junitxml=None,
        junit_xml=None,
        json_report=None,
        jsonl=False,
        slow_report=None,
        slow_threshold_ms=800,
        timeout_ms=2000,
        slow_ratchet_config="configs/policy/slow-checks-ratchet.json",
        ignore_speed_regressions=False,
        profile=False,
        profile_out=None,
        jobs=1,
        match=None,
        group=None,
        exclude_group=[],
        domain_filter=mapped_domain,
        id=None,
        legacy_id=False,
        k=None,
        only_slow=False,
        only_fast=False,
        exclude_slow=False,
        suite=None,
        marker=["lint"],
        tag=["lint"],
        exclude_marker=[],
        exclude_tag=[],
        list_selected=bool(getattr(ns, "list_selected", False)),
        from_registry=True,
        require_markers=[],
        select=None,
        check_target=None,
        json=bool(getattr(ns, "json", False)),
        category=None,
    )
    return run_check_command(ctx, alias_ns)
