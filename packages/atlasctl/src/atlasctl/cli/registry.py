from __future__ import annotations

import argparse
import json
from dataclasses import dataclass

from ..run_context import RunContext

DomainPayload = dict[str, object]


@dataclass(frozen=True)
class CommandSpec:
    name: str
    help_text: str
    touches: tuple[str, ...] = ()
    tools: tuple[str, ...] = ()
    failure_modes: tuple[str, ...] = ()
    stable: bool = True


CANONICAL_DOMAINS: tuple[str, ...] = ("repo", "docs", "ops", "make", "configs", "contracts", "docker", "ci")


def command_registry() -> tuple[CommandSpec, ...]:
    return (
        CommandSpec("doctor", "show tooling and context diagnostics", ("artifacts/evidence/",), ("python3",), ("tooling probes unavailable",)),
        CommandSpec("inventory", "generate and verify inventories", ("docs/_generated/",), (), ("inventory drift",)),
        CommandSpec("gates", "run gate contracts and lane checks", ("configs/gates/", "artifacts/evidence/"), ("make",), ("gate contract failures",)),
        CommandSpec("ops", "ops checks and suite orchestration", ("ops/", "artifacts/evidence/"), ("kubectl", "helm", "k6"), ("ops contract drift",)),
        CommandSpec("docs", "docs checks and generation", ("docs/", "docs/_generated/"), ("mkdocs",), ("doc drift",)),
        CommandSpec("configs", "configs checks and inventories", ("configs/",), (), ("schema/config validation errors",)),
        CommandSpec("policies", "policy relaxations and bypass checks", ("configs/policy/",), (), ("policy violations",)),
        CommandSpec("k8s", "k8s checks and suites", ("ops/k8s/",), ("kubectl", "helm"), ("k8s contract violations",)),
        CommandSpec("stack", "stack lifecycle and checks", ("ops/stack/",), ("kubectl",), ("stack lifecycle failures",)),
        CommandSpec("load", "load and perf suites", ("ops/load/",), ("k6",), ("load baseline regressions",)),
        CommandSpec("obs", "observability checks and drills", ("ops/obs/",), (), ("observability coverage gaps",)),
        CommandSpec("report", "unified report and scorecard commands", ("artifacts/evidence/",), (), ("report assembly failures",)),
        CommandSpec("lint", "lint suite runner", ("ops/_lint/",), (), ("lint policy failures",)),
        CommandSpec("legacy", "legacy migration audits", ("configs/layout/",), (), ("legacy reference drift",)),
        CommandSpec("compat", "deprecated shim inventory and checks", (), (), ("compat shim drift",), stable=False),
        CommandSpec("contracts", "contracts domain commands", ("configs/contracts/",), (), ("contract generation/validation failures",)),
        CommandSpec("registry", "registry domain commands", ("configs/ops/pins/",), (), ("registry inconsistency",)),
        CommandSpec("layout", "layout domain commands", ("makefiles/", "docs/development/repo-layout.md"), (), ("layout boundary violations",)),
    )


def registry() -> tuple[CommandSpec, ...]:
    # Backward-compatible alias.
    return command_registry()


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
