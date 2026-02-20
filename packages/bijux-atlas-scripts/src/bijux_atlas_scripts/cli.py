from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

import jsonschema
import typer

from .run_id import make_run_id
from .version import __version__

app = typer.Typer(help="atlas-scripts command surface (SSOT wrapper CLI)")
gates_app = typer.Typer(help="gate discovery and execution")
make_app = typer.Typer(help="make metadata helpers")
docs_app = typer.Typer(help="docs helpers")
ops_app = typer.Typer(help="ops helpers")
obs_app = typer.Typer(help="observability helpers")
pins_app = typer.Typer(help="pin policy helpers")
inventory_app = typer.Typer(help="inventory helpers")
schema_app = typer.Typer(help="schema helpers")
json_app = typer.Typer(help="json helpers")
report_app = typer.Typer(help="report helpers")
evidence_app = typer.Typer(help="evidence helpers")

app.add_typer(gates_app, name="gates")
app.add_typer(make_app, name="make")
app.add_typer(docs_app, name="docs")
app.add_typer(ops_app, name="ops")
app.add_typer(obs_app, name="obs")
app.add_typer(pins_app, name="pins")
app.add_typer(inventory_app, name="inventory")
app.add_typer(schema_app, name="schema")
app.add_typer(json_app, name="json")
app.add_typer(report_app, name="report")
app.add_typer(evidence_app, name="evidence")


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[4]


def _run(cmd: list[str], cwd: Path | None = None) -> int:
    proc = subprocess.run(cmd, cwd=cwd or _repo_root(), check=False)
    return proc.returncode


def _run_make(target: str, *extra: str) -> int:
    return _run(["make", "-s", target, *extra], cwd=_repo_root())


@app.callback()
def _main(version: bool = typer.Option(False, "--version", help="show version and exit")) -> None:
    if version:
        typer.echo(f"atlas-scripts {__version__}")
        raise typer.Exit(0)


@app.command("doctor")
def doctor(json_out: bool = typer.Option(False, "--json")) -> None:
    payload = {"tool": "atlas-scripts", "version": __version__, "run_id": make_run_id("atlas-scripts")}
    if json_out:
        typer.echo(json.dumps(payload, sort_keys=True))
    else:
        typer.echo(f"tool={payload['tool']} version={payload['version']} run_id={payload['run_id']}")


GATE_TARGETS = {
    "root": "root",
    "root-local": "root-local",
    "scripts-check": "scripts-check",
    "docs-check": "docs/check",
    "ops-check": "ops/check",
    "pins-check": "pins/check",
}


@gates_app.command("list")
def gates_list() -> None:
    typer.echo(json.dumps({"gates": sorted(GATE_TARGETS.keys())}, sort_keys=True))


@gates_app.command("run")
def gates_run(gate: str) -> None:
    target = GATE_TARGETS.get(gate)
    if not target:
        raise typer.BadParameter(f"unknown gate: {gate}")
    raise typer.Exit(_run_make(target))


@make_app.command("catalog")
def make_catalog() -> None:
    raise typer.Exit(_run_make("inventory"))


@docs_app.command("build-metadata")
def docs_build_metadata() -> None:
    raise typer.Exit(_run(["./scripts/bin/bijux-atlas-scripts", "docs", "inventory", "--report", "json"], cwd=_repo_root()))


@docs_app.command("verify")
def docs_verify() -> None:
    raise typer.Exit(_run_make("docs/check"))


@ops_app.command("smoke")
def ops_smoke() -> None:
    raise typer.Exit(_run_make("ops/smoke"))


@ops_app.command("check")
def ops_check() -> None:
    raise typer.Exit(_run_make("ops/check"))


@ops_app.command("k8s-suite")
def ops_k8s_suite() -> None:
    raise typer.Exit(_run_make("ops-k8s-suite"))


@obs_app.command("verify")
def obs_verify(suite: str = typer.Option("cheap", "--suite")) -> None:
    raise typer.Exit(_run_make("ops-obs-verify", f"SUITE={suite}"))


@pins_app.command("check")
def pins_check() -> None:
    raise typer.Exit(_run_make("pins/check"))


@pins_app.command("update")
def pins_update(allow_update: bool = typer.Option(False, "--allow-update", help="guarded update flag")) -> None:
    if not allow_update:
        raise typer.BadParameter("pins update is guarded; use --allow-update")
    raise typer.Exit(_run_make("pins/update"))


@inventory_app.command("build")
def inventory_build() -> None:
    raise typer.Exit(_run_make("inventory"))


@inventory_app.command("drift")
def inventory_drift() -> None:
    raise typer.Exit(_run_make("verify-inventory"))


@schema_app.command("validate")
def schema_validate(path: str = typer.Argument(..., help="json file path to validate")) -> None:
    target = Path(path)
    if not target.exists():
        raise typer.BadParameter(f"missing file: {path}")
    data = json.loads(target.read_text(encoding="utf-8"))
    schema_path = _repo_root() / "ops/_schemas/report/unified.schema.json"
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    jsonschema.validate(data, schema)
    typer.echo("ok")


@json_app.command("canonicalize")
def json_canonicalize(path: str = typer.Argument(...), inplace: bool = typer.Option(True, "--inplace/--stdout")) -> None:
    target = Path(path)
    payload: Any = json.loads(target.read_text(encoding="utf-8"))
    canonical = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    if inplace:
        target.write_text(canonical + "\n", encoding="utf-8")
        typer.echo(str(target))
    else:
        typer.echo(canonical)


@report_app.command("unify")
def report_unify(run_id: str = typer.Option("latest", "--run-id")) -> None:
    if run_id == "latest":
        latest_file = _repo_root() / "artifacts/evidence/latest-run-id.txt"
        if latest_file.exists():
            run_id = latest_file.read_text(encoding="utf-8").strip() or "latest"
    raise typer.Exit(_run(["./scripts/bin/bijux-atlas-scripts", "report", "collect", "--run-id", run_id], cwd=_repo_root()))


@evidence_app.command("gc")
def evidence_gc() -> None:
    raise typer.Exit(_run_make("evidence/clean"))


def main() -> int:
    app(prog_name="atlas-scripts")
    return 0
