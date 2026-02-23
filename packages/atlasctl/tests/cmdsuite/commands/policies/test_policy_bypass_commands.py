from __future__ import annotations

import argparse
import json

from atlasctl.commands.policies.runtime.command import run_policies_command
from atlasctl.core.context import RunContext


def test_policy_bypass_list_grouped_by_owner(monkeypatch, capsys) -> None:
    monkeypatch.setattr(
        "atlasctl.commands.policies.runtime.command.collect_bypass_inventory",
        lambda _repo: {
            "entries": [
                {"owner": "ops", "expiry": "2026-03-01"},
                {"owner": "ops", "expiry": "2026-04-01"},
                {"owner": "platform", "expiry": "2026-03-01"},
            ]
        },
    )
    ctx = RunContext.from_args("policy-by-owner", None, "test", False)
    ns = argparse.Namespace(policies_cmd="bypass", bypass_cmd="list", report="json", blame=False, by="owner")
    rc = run_policies_command(ctx, ns)
    assert rc == 0
    payload = json.loads(capsys.readouterr().out)
    assert payload["kind"] == "bypass-by-owner"
    assert any(row["key"] == "ops" and row["count"] == 2 for row in payload["groups"])


def test_policy_bypass_burn_due(monkeypatch, capsys) -> None:
    monkeypatch.setattr(
        "atlasctl.commands.policies.runtime.command.collect_bypass_inventory",
        lambda _repo: {
            "entries": [
                {
                    "source": "configs/policy/a.json",
                    "key": "a",
                    "owner": "ops",
                    "issue_id": "ISSUE-1",
                    "expiry": "2026-03-01",
                    "removal_plan": "docs/policies/bypass-removal-playbook.md",
                },
                {
                    "source": "configs/policy/b.json",
                    "key": "b",
                    "owner": "ops",
                    "issue_id": "ISSUE-2",
                    "expiry": "2026-05-01",
                    "removal_plan": "docs/policies/bypass-removal-playbook.md",
                },
            ]
        },
    )
    ctx = RunContext.from_args("policy-burn-due", None, "test", False)
    ns = argparse.Namespace(policies_cmd="bypass", bypass_cmd="burn", due="2026-03-15", report="json")
    rc = run_policies_command(ctx, ns)
    assert rc == 1
    payload = json.loads(capsys.readouterr().out)
    assert payload["kind"] == "bypass-burn-due"
    assert payload["count"] == 1
    assert payload["entries"][0]["key"] == "a"

