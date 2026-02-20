from __future__ import annotations

import argparse
import json

from .run_context import RunContext

DomainPayload = dict[str, object]


def register_domain_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser], name: str, help_text: str) -> None:
    p = sub.add_parser(name, help=help_text)
    p.add_argument("--json", action="store_true", help="emit machine-readable JSON output")
    p.add_argument("--out-file", help="write JSON output file under evidence root")
    p.set_defaults(domain=name)


def domain_payload(ctx: RunContext, domain: str) -> DomainPayload:
    return {
        "tool": "bijux-atlas-scripts",
        "domain": domain,
        "run_id": ctx.run_id,
        "profile": ctx.profile,
        "repo_root": str(ctx.repo_root),
        "evidence_root": str(ctx.evidence_root),
        "status": "ok",
    }


def render_payload(payload: DomainPayload, as_json: bool) -> str:
    if as_json:
        return json.dumps(payload, sort_keys=True)
    return f"{payload['domain']}: status={payload['status']} run_id={payload['run_id']}"
