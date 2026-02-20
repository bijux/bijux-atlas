from __future__ import annotations

import argparse
import json
from dataclasses import dataclass

from .run_context import RunContext

DomainPayload = dict[str, object]


@dataclass(frozen=True)
class CommandSpec:
    name: str
    help_text: str
    stable: bool = True


def registry() -> tuple[CommandSpec, ...]:
    return (
        CommandSpec("doctor", "show tooling and context diagnostics"),
        CommandSpec("inventory", "generate and verify inventories"),
        CommandSpec("gates", "run gate contracts and lane checks"),
        CommandSpec("ops", "ops checks and suite orchestration"),
        CommandSpec("docs", "docs checks and generation"),
        CommandSpec("configs", "configs checks and inventories"),
        CommandSpec("policies", "policy relaxations and bypass checks"),
        CommandSpec("k8s", "k8s checks and suites"),
        CommandSpec("stack", "stack lifecycle and checks"),
        CommandSpec("load", "load and perf suites"),
        CommandSpec("obs", "observability checks and drills"),
        CommandSpec("report", "unified report and scorecard commands"),
        CommandSpec("lint", "lint suite runner"),
        CommandSpec("compat", "deprecated shim inventory and checks", stable=False),
        CommandSpec("contracts", "contracts domain commands"),
        CommandSpec("registry", "registry domain commands"),
        CommandSpec("layout", "layout domain commands"),
    )


def register_domain_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser], name: str, help_text: str) -> None:
    p = sub.add_parser(name, help=help_text)
    p.add_argument("--json", action="store_true", help="emit machine-readable JSON output")
    p.add_argument("--out-file", help="write JSON output file under evidence root")
    p.set_defaults(domain=name)


def domain_payload(ctx: RunContext, domain: str) -> DomainPayload:
    return {
        "tool": "bijux-atlas",
        "domain": domain,
        "run_id": ctx.run_id,
        "profile": ctx.profile,
        "repo_root": str(ctx.repo_root),
        "evidence_root": str(ctx.evidence_root),
        "format": ctx.output_format,
        "network": ctx.network_mode,
        "status": "ok",
    }


def render_payload(payload: DomainPayload, as_json: bool) -> str:
    if as_json:
        return json.dumps(payload, sort_keys=True)
    return f"{payload['domain']}: status={payload['status']} run_id={payload['run_id']}"
