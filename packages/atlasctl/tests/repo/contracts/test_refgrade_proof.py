from __future__ import annotations

import json
import re
import shutil
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    return subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_golden_key_commands_doctor_inventory_gates_list() -> None:
    for cmd in (
        ("--quiet", "doctor", "--json"),
        ("--quiet", "inventory", "make", "--format", "json", "--dry-run"),
        ("--quiet", "gates", "list", "--report", "json"),
    ):
        proc = _run_cli(*cmd)
        assert proc.returncode == 0, proc.stderr
        payload = json.loads(proc.stdout)
        assert payload.get("schema_version", 1) >= 1
        assert payload.get("tool", "bijux-atlas") == "bijux-atlas"


def test_cli_commands_have_description_and_owner_contract() -> None:
    proc = _run_cli("--quiet", "help", "--json")
    assert proc.returncode == 0, proc.stderr
    commands = json.loads(proc.stdout)["commands"]
    owners = json.loads((ROOT / "configs/meta/ownership.json").read_text(encoding="utf-8"))["commands"]
    for row in commands:
        name = row["name"]
        assert row["help"].strip(), f"missing help text: {name}"
        assert f"bijux-atlas {name}" in owners, f"missing owner mapping: {name}"


def test_reports_from_key_commands_validate_base_contract() -> None:
    schema = json.loads((ROOT / "configs/contracts/scripts-tool-output.schema.json").read_text(encoding="utf-8"))
    import jsonschema

    for cmd in (
        ("--quiet", "legacy", "inventory", "--report", "json"),
        ("--quiet", "ports", "show", "--report", "json"),
        ("--quiet", "gates", "list", "--report", "json"),
    ):
        proc = _run_cli(*cmd)
        assert proc.returncode == 0, proc.stderr
        payload = json.loads(proc.stdout)
        if "status" in payload:
            jsonschema.validate(payload, schema)


def test_schema_files_are_referenced_by_docs_or_code() -> None:
    schema_files = sorted((ROOT / "configs/contracts").glob("*.schema.json"))
    corpus = []
    for path in [ROOT / "docs", ROOT / "packages/atlasctl/src", ROOT / "makefiles"]:
        for f in path.rglob("*"):
            if f.is_file():
                try:
                    corpus.append(f.read_text(encoding="utf-8"))
                except Exception:
                    pass
    text = "\n".join(corpus)
    transitional = {"inventory-scripts-migration.schema.json"}
    missing = [s.name for s in schema_files if s.name not in text and s.name not in transitional]
    assert not missing, f"unreferenced schemas: {missing}"


def test_inventory_is_deterministic_across_runs() -> None:
    a = _run_cli("--quiet", "inventory", "make", "--format", "json", "--dry-run")
    b = _run_cli("--quiet", "inventory", "make", "--format", "json", "--dry-run")
    assert a.returncode == 0 and b.returncode == 0
    assert a.stdout == b.stdout


def test_offline_mode_for_read_only_commands() -> None:
    for cmd in (
        ("--network", "forbid", "--quiet", "doctor", "--json"),
        ("--network", "forbid", "--quiet", "gates", "list", "--report", "json"),
        ("--network", "forbid", "--quiet", "inventory", "make", "--format", "json", "--dry-run"),
    ):
        proc = _run_cli(*cmd)
        assert proc.returncode == 0, proc.stderr


def test_network_allow_mode_does_not_install_network_guard() -> None:
    probe = ROOT / "artifacts/scripts/net_probe.py"
    probe.parent.mkdir(parents=True, exist_ok=True)
    probe.write_text(
        "import socket\n"
        "try:\n"
        "  socket.create_connection(('127.0.0.1', 9), timeout=0.1)\n"
        "except Exception as e:\n"
        "  print(str(e))\n",
        encoding="utf-8",
    )
    proc = _run_cli("--network", "allow", "--quiet", "run", str(probe.relative_to(ROOT)))
    assert proc.returncode in (0, 1)
    assert "network disabled by --no-network" not in proc.stderr


def test_run_dir_isolation_parallel_runs(tmp_path: Path) -> None:
    def run_one(run_id: str) -> int:
        p = _run_cli(
            "--quiet",
            "--run-id",
            run_id,
            "--evidence-root",
            str(tmp_path),
            "ports",
            "reserve",
            "--name",
            run_id,
            "--report",
            "json",
        )
        return p.returncode

    with ThreadPoolExecutor(max_workers=2) as ex:
        r1 = ex.submit(run_one, "iso-a")
        r2 = ex.submit(run_one, "iso-b")
        assert r1.result() == 0
        assert r2.result() == 0
    assert (tmp_path / "ports" / "iso-a" / "report.json").exists()
    assert (tmp_path / "ports" / "iso-b" / "report.json").exists()


def test_ops_generated_runtime_dir_not_required_by_tooling() -> None:
    command_roots = [
        ROOT / "packages/atlasctl/src/atlasctl/cli/main.py",
        ROOT / "packages/atlasctl/src/atlasctl/orchestrate/command.py",
        ROOT / "packages/atlasctl/src/atlasctl/reporting/command.py",
        ROOT / "packages/atlasctl/src/atlasctl/gates/command.py",
        ROOT / "packages/atlasctl/src/atlasctl/commands/docs/runtime.py",
        ROOT / "packages/atlasctl/src/atlasctl/configs/command.py",
        ROOT / "packages/atlasctl/src/atlasctl/commands/ops/runtime.py",
    ]
    for path in command_roots:
        text = path.read_text(encoding="utf-8")
        assert not re.search(r"ops/_generated/.*read_text\(", text, flags=re.S), (
            f"runtime read from deprecated path in {path}"
        )


def test_no_shell_true_subprocess_usage() -> None:
    offenders = []
    for path in sorted((ROOT / "packages/atlasctl/src").rglob("*.py")):
        text = path.read_text(encoding="utf-8")
        if "shell=True" in text:
            offenders.append(path.relative_to(ROOT).as_posix())
    assert not offenders, f"shell=True forbidden: {offenders}"


def test_python_dependencies_are_pinned_in_lock() -> None:
    lines = (ROOT / "packages/atlasctl/requirements.lock.txt").read_text(encoding="utf-8").splitlines()
    deps = [ln.strip() for ln in lines if ln.strip() and not ln.startswith("#")]
    assert deps
    assert all("==" in dep for dep in deps), f"unpinned deps found: {deps}"


def test_no_print_outside_cli_and_command_modules() -> None:
    offenders = []
    enforce_roots = (
        ROOT / "packages/atlasctl/src/atlasctl/core",
        ROOT / "packages/atlasctl/src/atlasctl/registry",
        ROOT / "packages/atlasctl/src/atlasctl/report",
    )
    for root in enforce_roots:
        for path in sorted(root.rglob("*.py")):
            if path.name in {"command.py", "doctor.py"}:
                continue
            text = path.read_text(encoding="utf-8")
            if "print(" in text:
                offenders.append(path.relative_to(ROOT).as_posix())
    assert not offenders, f"print() outside structured entrypoints: {offenders}"


def test_mkdocs_build_deterministic_when_available(tmp_path: Path) -> None:
    if shutil.which("mkdocs") is None:
        return
    site_a = tmp_path / "site-a"
    site_b = tmp_path / "site-b"
    a = subprocess.run(["mkdocs", "build", "--clean", "--site-dir", str(site_a)], cwd=ROOT, text=True, capture_output=True)
    b = subprocess.run(["mkdocs", "build", "--clean", "--site-dir", str(site_b)], cwd=ROOT, text=True, capture_output=True)
    assert a.returncode == 0, a.stderr
    assert b.returncode == 0, b.stderr
    index_a = (site_a / "index.html").read_text(encoding="utf-8", errors="ignore")
    index_b = (site_b / "index.html").read_text(encoding="utf-8", errors="ignore")
    assert index_a == index_b
