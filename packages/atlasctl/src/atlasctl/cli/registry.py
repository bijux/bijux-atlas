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
    owner: str = "platform"
    doc_link: str = "docs/_generated/cli.md"


CANONICAL_DOMAINS: tuple[str, ...] = ("repo", "docs", "ops", "make", "configs", "contracts", "docker", "ci")


def command_registry() -> tuple[CommandSpec, ...]:
    return (
        CommandSpec("doctor", "show tooling and context diagnostics", ("artifacts/evidence/",), ("python3",), ("tooling probes unavailable",), owner="platform", doc_link="docs/atlasctl/BOUNDARIES.md"),
        CommandSpec("inventory", "generate and verify inventories", ("docs/_generated/",), (), ("inventory drift",), owner="repo", doc_link="docs/_generated/cli.md"),
        CommandSpec("gates", "run gate contracts and lane checks", ("configs/gates/", "artifacts/evidence/"), ("make",), ("gate contract failures",), owner="ops", doc_link="docs/_generated/cli.md"),
        CommandSpec("ops", "ops checks and suite orchestration", ("ops/", "artifacts/evidence/"), ("kubectl", "helm", "k6"), ("ops contract drift",), owner="ops", doc_link="docs/_generated/cli.md"),
        CommandSpec("docs", "docs checks and generation", ("docs/", "docs/_generated/"), ("mkdocs",), ("doc drift",), owner="docs", doc_link="docs/_generated/cli.md"),
        CommandSpec("configs", "configs checks and inventories", ("configs/",), (), ("schema/config validation errors",), owner="platform", doc_link="docs/_generated/cli.md"),
        CommandSpec("policies", "policy relaxations and bypass checks", ("configs/policy/",), (), ("policy violations",), owner="platform", doc_link="docs/_generated/cli.md"),
        CommandSpec("k8s", "k8s checks and suites", ("ops/k8s/",), ("kubectl", "helm"), ("k8s contract violations",), owner="ops", doc_link="docs/_generated/cli.md"),
        CommandSpec("stack", "stack lifecycle and checks", ("ops/stack/",), ("kubectl",), ("stack lifecycle failures",), owner="ops", doc_link="docs/_generated/cli.md"),
        CommandSpec("load", "load and perf suites", ("ops/load/",), ("k6",), ("load baseline regressions",), owner="ops", doc_link="docs/_generated/cli.md"),
        CommandSpec("obs", "observability checks and drills", ("ops/obs/",), (), ("observability coverage gaps",), owner="ops", doc_link="docs/_generated/cli.md"),
        CommandSpec("report", "unified report and scorecard commands", ("artifacts/evidence/",), (), ("report assembly failures",), owner="platform", doc_link="docs/_generated/cli.md"),
        CommandSpec("suite", "atlasctl-native suite runner", ("artifacts/reports/atlasctl/",), ("python3",), ("suite task failures",), owner="platform", doc_link="docs/_generated/cli.md"),
        CommandSpec("lint", "lint suite runner", ("ops/_lint/",), (), ("lint policy failures",), owner="platform", doc_link="docs/_generated/cli.md"),
        CommandSpec("legacy", "legacy migration audits", ("configs/layout/",), (), ("legacy reference drift",), owner="platform", doc_link="docs/_generated/cli.md"),
        CommandSpec("compat", "deprecated shim inventory and checks", ("configs/layout/",), (), ("compat shim drift",), stable=False, owner="platform", doc_link="docs/_generated/cli.md"),
        CommandSpec("contracts", "contracts domain commands", ("configs/contracts/",), (), ("contract generation/validation failures",), owner="platform", doc_link="docs/_generated/cli.md"),
        CommandSpec("registry", "registry domain commands", ("configs/ops/pins/",), (), ("registry inconsistency",), owner="platform", doc_link="docs/_generated/cli.md"),
        CommandSpec("layout", "layout domain commands", ("makefiles/", "docs/development/repo-layout.md"), (), ("layout boundary violations",), owner="repo", doc_link="docs/development/repo-layout.md"),
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
