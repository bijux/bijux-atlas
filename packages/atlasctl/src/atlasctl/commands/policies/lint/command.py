from __future__ import annotations

import argparse
import json

from ....core.context import RunContext
from ....core.fs import ensure_evidence_path
from ....core.runtime.paths import write_text_file
from ....core.runtime.script_registry import emit_script_registry_evidence, lint_script_registry
from ....core.schema.schema_utils import validate_json
from .suite_engine import run_lint_suite

SUITES = ("ops", "repo", "makefiles", "docs", "configs", "packages", "scripts")

def configure_lint_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("lint", help="run canonical lint suites")
    p.add_argument("suite", choices=SUITES)
    p.add_argument("--fail-fast", action="store_true")
    p.add_argument("--emit-artifacts", action="store_true")
    p.add_argument("--report", choices=["text", "json"], default="text")

def run_lint_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.suite == "scripts":
        code, payload = lint_script_registry(ctx.repo_root)
        if ns.emit_artifacts:
            emit_script_registry_evidence(ctx, payload)
        if ns.report == "json" or ctx.output_format == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(
                f"lint scripts: {payload['status']} "
                f"(registered={payload['registered_count']} referenced={payload['referenced_count']} errors={len(payload['errors'])})"
            )
            for err in payload["errors"]:
                print(f"- {err}")
        return code

    code, payload = run_lint_suite(ctx.repo_root, ns.suite, ns.fail_fast)
    schema_path = ctx.repo_root / "configs/contracts/scripts-tool-output.schema.json"
    validate_json({"schema_version": 1, "tool": payload["tool"], "status": payload["status"]}, schema_path)

    if ns.emit_artifacts:
        out = ensure_evidence_path(ctx, ctx.evidence_root / "lint" / ns.suite / ctx.run_id / "report.json")
        write_text_file(out, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if ns.report == "json" or ctx.output_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"lint {ns.suite}: {payload['status']} ({payload['failed_count']}/{payload['total_count']} failed)")
    return code
