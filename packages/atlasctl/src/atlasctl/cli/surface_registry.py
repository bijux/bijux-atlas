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
    effect_level: str = "effectful"
    run_id_mode: str = "accept_or_generate"
    supports_dry_run: bool = True
    aliases: tuple[str, ...] = ()
    purpose: str = ""
    examples: tuple[str, ...] = ()
    internal: bool = False


CANONICAL_DOMAINS: tuple[str, ...] = ("repo", "docs", "ops", "make", "configs", "contracts", "docker", "ci")


def command_registry() -> tuple[CommandSpec, ...]:
    return (
        CommandSpec("doctor", "show tooling and context diagnostics", ("artifacts/evidence/",), ("python3",), ("tooling probes unavailable",), owner="platform", doc_link="docs/atlasctl/BOUNDARIES.md", purpose="show command/runtime diagnostics", examples=("atlasctl doctor --json",)),
        CommandSpec("inventory", "generate and verify inventories", ("docs/_generated/",), (), ("inventory drift",), owner="repo", doc_link="docs/_generated/cli.md", purpose="build and verify canonical inventories", examples=("atlasctl inventory --category all --format json",)),
        CommandSpec("gates", "run gate contracts and lane checks", ("configs/gates/", "artifacts/evidence/"), ("make",), ("gate contract failures",), owner="ops", doc_link="docs/_generated/cli.md", purpose="execute gate policies and lane checks", examples=("atlasctl gates --report json",)),
        CommandSpec("ops", "ops checks and suite orchestration", ("ops/", "artifacts/evidence/"), ("kubectl", "helm", "k6"), ("ops contract drift",), owner="ops", doc_link="docs/_generated/cli.md", purpose="run ops domain checks and orchestration", examples=("atlasctl ops status --report json",)),
        CommandSpec("docs", "docs checks and generation", ("docs/", "docs/_generated/"), ("mkdocs",), ("doc drift",), owner="docs", doc_link="docs/_generated/cli.md", purpose="validate and generate docs surfaces", examples=("atlasctl docs check --report json",)),
        CommandSpec("configs", "configs checks and inventories", ("configs/",), (), ("schema/config validation errors",), owner="platform", doc_link="docs/_generated/cli.md", purpose="validate and inspect configuration", examples=("atlasctl configs validate --report json",)),
        CommandSpec("policies", "policy relaxations and bypass checks", ("configs/policy/",), (), ("policy violations",), owner="platform", doc_link="docs/_generated/cli.md", purpose="run policy checks and culprits reports", examples=("atlasctl policies explain budgets",)),
        CommandSpec("repo", "repository stats and density reports", ("configs/policy/", "artifacts/reports/atlasctl/"), (), ("repo stats generation failure",), owner="repo", doc_link="docs/_generated/cli.md", purpose="analyze repository structure and stats", examples=("atlasctl repo stats --json",)),
        CommandSpec("k8s", "k8s checks and suites", ("ops/k8s/",), ("kubectl", "helm"), ("k8s contract violations",), owner="ops", doc_link="docs/_generated/cli.md", purpose="run k8s checks and suites", examples=("atlasctl k8s conformance --report json",)),
        CommandSpec("stack", "stack lifecycle and checks", ("ops/stack/",), ("kubectl",), ("stack lifecycle failures",), owner="ops", doc_link="docs/_generated/cli.md", purpose="manage and verify stack lifecycle", examples=("atlasctl stack status --report json",)),
        CommandSpec("load", "load and perf suites", ("ops/load/",), ("k6",), ("load baseline regressions",), owner="ops", doc_link="docs/_generated/cli.md", purpose="run load/perf workflows", examples=("atlasctl load run --report json",)),
        CommandSpec("obs", "observability checks and drills", ("ops/obs/",), (), ("observability coverage gaps",), owner="ops", doc_link="docs/_generated/cli.md", purpose="validate observability coverage and contracts", examples=("atlasctl obs checks --report json",)),
        CommandSpec("report", "unified report and scorecard commands", ("artifacts/evidence/",), (), ("report assembly failures",), owner="platform", doc_link="docs/_generated/cli.md", purpose="assemble and emit canonical reports", examples=("atlasctl report summary --run-id local",)),
        CommandSpec("suite", "atlasctl-native suite runner", ("artifacts/reports/atlasctl/",), ("python3",), ("suite task failures",), owner="platform", doc_link="docs/_generated/cli.md", purpose="run canonical suite definitions", examples=("atlasctl suite run ci --json",)),
        CommandSpec("lint", "lint suite runner", ("ops/_lint/",), (), ("lint policy failures",), owner="platform", doc_link="docs/_generated/cli.md", purpose="run lint lanes through atlasctl", examples=("atlasctl lint run --report json",)),
        CommandSpec("contracts", "contracts domain commands", ("configs/contracts/",), (), ("contract generation/validation failures",), owner="platform", doc_link="docs/_generated/cli.md", purpose="manage schema contracts and validation", examples=("atlasctl contracts list --json",)),
        CommandSpec("check", "run registered check suites and individual checks", ("artifacts/evidence/",), (), ("check failures",), owner="platform", doc_link="docs/_generated/cli.md", purpose="execute registered policy checks", examples=("atlasctl check list --json",)),
        CommandSpec("list", "list checks/commands/suites from canonical registries", ("artifacts/evidence/",), (), ("registry list failures",), owner="platform", doc_link="docs/_generated/cli.md", purpose="query canonical registry inventories", examples=("atlasctl list checks --json",)),
        CommandSpec("dev", "dev control-plane group command surface", ("artifacts/evidence/",), (), ("dev group forwarding failure",), owner="platform", doc_link="docs/_generated/cli.md", purpose="group development commands under one control-plane entry", examples=("atlasctl dev check -- domain repo",)),
        CommandSpec("internal", "internal control-plane group command surface", ("artifacts/evidence/",), (), ("internal group forwarding failure",), owner="platform", doc_link="docs/_generated/cli.md", purpose="group internal diagnostics and legacy inventory surfaces", examples=("atlasctl internal legacy inventory --report json",), stable=False, internal=True),
        CommandSpec("test", "run canonical atlasctl test suites", ("artifacts/isolate/",), ("python3",), ("pytest suite failures",), owner="platform", doc_link="docs/_generated/cli.md", purpose="run canonical test lanes", examples=("atlasctl test all --report json",)),
        CommandSpec("registry", "registry domain commands", ("configs/ops/pins/",), (), ("registry inconsistency",), owner="platform", doc_link="docs/_generated/cli.md", purpose="inspect command and check registry state", examples=("atlasctl registry list --json",)),
        CommandSpec("layout", "layout domain commands", ("makefiles/", "docs/development/repo-layout.md"), (), ("layout boundary violations",), owner="repo", doc_link="docs/development/repo-layout.md", purpose="validate repository layout contracts", examples=("atlasctl layout root-shape --json",)),
    )


def registry() -> tuple[CommandSpec, ...]:
    # Backward-compatible alias.
    return command_registry()


def command_spec(name: str) -> CommandSpec | None:
    for spec in command_registry():
        if spec.name == name:
            return spec
    return None


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
