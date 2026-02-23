"""Parser wiring for `atlasctl check` and `atlasctl checks`."""

from __future__ import annotations

import argparse

from ...engine.runner import domains as check_domains


def configure_check_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("check", help="area-based checks mapped from control-plane registries")
    parser.add_argument("--fail-fast", action="store_true", help="stop after first failing check in multi-check runs")
    parser.add_argument("--json", action="store_true", help="emit JSON output")
    parser.add_argument("--list", dest="list_checks", action="store_true", help="list registered checks")
    parser.add_argument("--show-source", help="print source file for check id")
    parser_sub = parser.add_subparsers(dest="check_cmd", required=False)

    parser_sub.add_parser("all", help="run all native atlasctl checks")
    run = parser_sub.add_parser("run", help="run registered checks with pytest-like output")
    run.add_argument("--all", dest="include_all", action="store_true", help="include slow checks (default is fast-only)")
    run.add_argument("--quiet", dest="run_quiet", action="store_true", help="one line per check: PASS/FAIL/SKIP")
    run.add_argument("--info", dest="run_info", action="store_true", help="default info output mode with id + timing")
    run.add_argument("--verbose", dest="run_verbose", action="store_true", help="include timing, owners, and failure hints")
    run.add_argument("--maxfail", type=int, default=0, help="stop after N failing checks (0 disables)")
    run.add_argument("--max-failures", type=int, default=0, help="alias of --maxfail")
    run.add_argument("--failfast", action="store_true", help="stop after first failing check")
    run.add_argument("--fail-fast", dest="failfast", action="store_true", help="stop after first failing check")
    run.add_argument("--keep-going", action="store_true", help="continue through all checks (default)")
    run.add_argument("--durations", type=int, default=0, help="show N slowest checks in summary")
    run.add_argument("--junitxml", help="write junit xml output path")
    run.add_argument("--junit-xml", dest="junit_xml", help="write junit xml output path")
    run.add_argument("--json-report", help="write json report output path")
    run.add_argument("--report", dest="json_report", help="alias of --json-report")
    run.add_argument("--out", dest="json_report", help="alias of --json-report")
    run.add_argument("--jsonl", action="store_true", help="stream JSONL row events and summary")
    run.add_argument("--show-skips", action="store_true", help="show skipped checks in text output")
    run.add_argument("--write-root", help="required write root for checks declaring fs_write (must be under artifacts/runs/<run_id>/)")
    run.add_argument("--slow-report", help="write slow checks report output path")
    run.add_argument("--slow-threshold-ms", type=int, default=800, help="threshold for slow checks report")
    run.add_argument("--timeout-ms", type=int, default=2000, help="per-check timeout in milliseconds (0 disables timeout)")
    run.add_argument("--slow-ratchet-config", default="configs/policy/slow-checks-ratchet.json", help="slow-check ratchet config json")
    run.add_argument(
        "--ignore-speed-regressions",
        action="store_true",
        help="report speed regressions but do not fail the run on them",
    )
    run.add_argument("--profile", choices=["fast", "all", "slow"], help="selection shortcut profile")
    run.add_argument("--emit-profile", action="store_true", help="emit check run performance profile artifact")
    run.add_argument("--profile-out", help="performance profile output path")
    run.add_argument("--jobs", type=int, default=1, help="number of worker jobs for check execution")
    run.add_argument("--match", help="glob pattern over check ids/titles")
    run.add_argument("--group", help="filter checks by group/domain")
    run.add_argument("--exclude-group", action="append", default=[], help="exclude checks by group/domain (repeatable)")
    run.add_argument("--domain", dest="domain_filter", help="filter checks by domain")
    run.add_argument("--category", choices=["lint", "check"], help="filter checks by category")
    run.add_argument("--owner", action="append", default=[], help="filter checks by owner (repeatable)")
    run.add_argument("--id", help="run a single check id")
    run.add_argument("--legacy-id", action="store_true", help="allow legacy dotted check ids for transition period")
    run.add_argument("-k", help="substring selector over check id/title")
    run.add_argument("--slow", dest="only_slow", action="store_true", help="run only slow checks")
    run.add_argument("--only-slow", dest="only_slow", action="store_true", help="run only slow checks")
    run.add_argument("--fast", dest="only_fast", action="store_true", help="run only fast checks")
    run.add_argument("--exclude-slow", action="store_true", help="exclude slow checks explicitly")
    run.add_argument("--suite", help="run checks selected by suite registry name")
    run.add_argument("-m", "--marker", action="append", default=[], help="include only checks with marker(s), comma-separated allowed")
    run.add_argument("--tag", action="append", default=[], help="include only checks with tag(s), comma-separated allowed")
    run.add_argument("--exclude-marker", action="append", default=[], help="exclude checks with marker(s), comma-separated allowed")
    run.add_argument("--exclude-tag", action="append", default=[], help="exclude checks with tag(s), comma-separated allowed")
    run.add_argument("--include-internal", action="store_true", help="include internal checks in selection")
    run.add_argument("--changed-only", action="store_true", help="run only checks whose module paths are touched in git diff")
    run.add_argument("--list-selected", action="store_true", help="print resolved checks and exit without execution")
    run.add_argument("--from-registry", action="store_true", default=True, help="load checks from registry (default)")
    run.add_argument("--require-markers", action="append", default=[], help="require check markers/tags (repeatable or comma-separated)")
    run.add_argument("--select", help="check selector, e.g. atlasctl::docs::check_x")
    run.add_argument("check_target", nargs="?", help="fully-qualified check id, e.g. atlasctl::docs::check_x")
    run.add_argument("--json", action="store_true", help="emit JSON output")
    list_parser = parser_sub.add_parser("list", help="list registered checks")
    list_parser.add_argument("--domain", dest="domain_filter", help="filter checks by domain")
    list_parser.add_argument("--category", choices=["lint", "check"], help="filter checks by category")
    list_parser.add_argument("--json", action="store_true", help="emit JSON output")
    explain = parser_sub.add_parser("explain", help="explain a check id")
    explain.add_argument("check_id")
    explain.add_argument("--legacy-id", action="store_true", help="allow legacy dotted check ids for transition period")
    doc = parser_sub.add_parser("doc", help="print check metadata and remediation")
    doc.add_argument("check_id")
    doc.add_argument("--legacy-id", action="store_true", help="allow legacy dotted check ids for transition period")
    doctor = parser_sub.add_parser("doctor", help="validate checks registry integrity and canonical metadata")
    doctor.add_argument("--json", action="store_true", help="emit JSON output")
    groups = parser_sub.add_parser("groups", help="show checks grouped by tags/groups")
    groups.add_argument("--json", action="store_true", help="emit JSON output")
    gates = parser_sub.add_parser("gates", help="show gate to checks mapping")
    gates.add_argument("--json", action="store_true", help="emit JSON output")
    slow = parser_sub.add_parser("slow", help="list slow checks")
    slow.add_argument("--json", action="store_true", help="emit JSON output")
    runtime = parser_sub.add_parser("runtime-contracts", help="run unified runtime contract checks and emit artifact")
    runtime.add_argument("--out-file", help="optional artifact output path under evidence root")
    rename = parser_sub.add_parser("rename-report", help="list legacy check ids mapped to canonical checks_* ids")
    rename.add_argument("--json", action="store_true", help="emit JSON output")
    failures = parser_sub.add_parser("failures", help="summarize failing checks from a check-run report")
    failures.add_argument("--last-run", required=True, help="path to check-run report json or run directory")
    failures.add_argument("--group", help="filter by domain group, e.g. repo")
    failures.add_argument("--json", action="store_true", help="emit JSON output")
    triage_slow = parser_sub.add_parser("triage-slow", help="list top-N slow checks from a check-run report")
    triage_slow.add_argument("--last-run", required=True, help="path to check-run report json or run directory")
    triage_slow.add_argument("--top", type=int, default=10, help="number of slow checks to include")
    triage_slow.add_argument("--json", action="store_true", help="emit JSON output")
    triage_fail = parser_sub.add_parser("triage-failures", help="group failing checks by domain/area from a check-run report")
    triage_fail.add_argument("--last-run", required=True, help="path to check-run report json or run directory")
    triage_fail.add_argument("--json", action="store_true", help="emit JSON output")
    domain = parser_sub.add_parser("domain", help="run checks for one domain")
    domain.add_argument("domain", choices=check_domains())

    parser_sub.add_parser("layout", help="run layout checks")
    parser_sub.add_parser("shell", help="run shell policy checks")
    parser_sub.add_parser("make", help="run makefile checks")
    parser_sub.add_parser("docs", help="run docs checks")
    parser_sub.add_parser("configs", help="run configs checks")
    parser_sub.add_parser("license", help="run licensing checks")
    repo = parser_sub.add_parser("repo", help="run repo hygiene checks")
    repo.add_argument("repo_check", nargs="?", choices=["all", "module-size", "hygiene"], default="all")
    parser_sub.add_parser("repo-hygiene", help="run strict repository hygiene checks")
    parser_sub.add_parser("obs", help="run observability checks")
    parser_sub.add_parser("stack-report", help="validate stack report contracts")

    for name, help_text in [
        ("cli-help", "validate script/CLI help coverage"),
        ("ownership", "validate script ownership coverage"),
        ("bin-entrypoints", "validate scripts/bin entrypoint cap"),
        ("root-bin-shims", "validate root bin shim minimalism policy"),
        ("duplicate-script-names", "validate duplicate script names"),
        ("make-scripts-refs", "validate no makefile references to scripts paths"),
        ("docs-scripts-refs", "validate docs contain no scripts/ path references"),
        ("make-help", "validate deterministic make help output"),
        ("forbidden-paths", "forbid scripts/xtask/tools direct recipe paths"),
        ("no-xtask", "forbid xtask references outside ADR history"),
        ("no-python-shebang-outside-packages", "forbid executable python scripts outside packages/"),
        ("forbidden-top-dirs", "fail if forbidden top-level directories exist"),
        ("module-size", "enforce max python module LOC budget"),
        ("ops-generated-tracked", "fail if ops/_generated contains tracked files"),
        ("tracked-timestamps", "fail if tracked paths contain timestamp-like directories"),
        ("committed-generated-hygiene", "fail on runtime/timestamped artifacts in committed generated directories"),
        ("effects-lint", "forbid runtime effects leakage in pure/query HTTP layers"),
        ("naming-intent-lint", "forbid generic helpers naming in crates tree"),
        ("make-command-allowlist", "enforce direct make recipe command allowlist"),
        ("python-migration-exceptions-expiry", "fail on expired python migration exceptions"),
        ("python-lock", "validate scripts requirements lock line format"),
        ("scripts-lock-sync", "validate scripts lockfile remains in sync with pyproject dev deps"),
        ("no-adhoc-python", "validate no unregistered ad-hoc python scripts are tracked"),
        ("no-direct-python-invocations", "forbid direct python invocations in docs/makefiles"),
        ("no-direct-bash-invocations", "forbid direct bash script invocations in docs/makefiles"),
        ("invocation-parity", "validate atlasctl invocation parity in make/docs"),
        ("scripts-surface-docs-drift", "validate scripts surface docs coverage from python tooling config"),
        ("script-errors", "validate structured script error contract"),
        ("script-write-roots", "validate scripts write only under approved roots"),
        ("script-tool-guards", "validate tool-using scripts include guard calls"),
        ("script-shim-expiry", "validate shim expiry metadata and budget"),
        ("script-shims-minimal", "validate shim wrappers remain minimal and deterministic"),
        ("venv-location-policy", "validate .venv locations are restricted"),
        ("repo-script-boundaries", "validate script location boundaries and transition exceptions"),
        ("atlas-cli-contract", "validate atlasctl CLI help/version deterministic contract"),
        ("bijux-boundaries", "validate atlasctl import boundaries"),
        ("make-targets-drift", "validate make target SSOT drift"),
        ("make-delegation-only", "validate wrapper makefiles delegate only to atlasctl"),
        ("workflow-calls-atlasctl", "validate workflow calls resolve to atlasctl entrypoints"),
        ("ci-surface-documented", "validate DEV/CI command surface docs coverage"),
        ("ops-mk-contract", "validate ops.mk wrapper-only contract and target budget"),
        ("checks-registry-drift", "validate checks REGISTRY.generated.json is in sync with REGISTRY.toml"),
    ]:
        parser_sub.add_parser(name, help=help_text)

    runtime_artifacts = parser_sub.add_parser("python-runtime-artifacts", help="validate runtime python artifacts stay outside tracked paths")
    runtime_artifacts.add_argument("--fix", action="store_true", help="remove forbidden runtime artifact paths in-place")
    sbom = parser_sub.add_parser("generate-scripts-sbom", help="emit python lock SBOM json")
    sbom.add_argument("--lock", required=True)
    sbom.add_argument("--out", required=True)


def configure_checks_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("checks", help="alias of `atlasctl check`")
    parser.add_argument("--fail-fast", action="store_true", help="stop after first failing check in multi-check runs")
    parser.add_argument("--json", action="store_true", help="emit JSON output")
    parser.add_argument("--list", dest="list_checks", action="store_true", help="list registered checks")
    parser.add_argument("--show-source", help="print source file for check id")
    parser_sub = parser.add_subparsers(dest="check_cmd", required=False)
    checks_list = parser_sub.add_parser("list", help="list registered checks")
    checks_list.add_argument("--domain", dest="domain_filter", help="filter checks by domain")
    checks_list.add_argument("--category", choices=["lint", "check"], help="filter checks by category")
    checks_list.add_argument("--json", action="store_true", help="emit JSON output")
    parser_sub.add_parser("tree", help="show checks grouped by domain/area")
    owners = parser_sub.add_parser("owners", help="show check ownership report")
    owners.add_argument("--json", action="store_true", help="emit JSON output")
    groups = parser_sub.add_parser("groups", help="show checks grouped by tags/groups")
    groups.add_argument("--json", action="store_true", help="emit JSON output")
    gates = parser_sub.add_parser("gates", help="show gate to checks mapping")
    gates.add_argument("--json", action="store_true", help="emit JSON output")
    slow = parser_sub.add_parser("slow", help="list slow checks")
    slow.add_argument("--json", action="store_true", help="emit JSON output")
    rename = parser_sub.add_parser("rename-report", help="list legacy check ids mapped to canonical checks_* ids")
    rename.add_argument("--json", action="store_true", help="emit JSON output")
    failures = parser_sub.add_parser("failures", help="summarize failing checks from a check-run report")
    failures.add_argument("--last-run", required=True, help="path to check-run report json or run directory")
    failures.add_argument("--group", help="filter by domain group, e.g. repo")
    failures.add_argument("--json", action="store_true", help="emit JSON output")
    triage_slow = parser_sub.add_parser("triage-slow", help="list top-N slow checks from a check-run report")
    triage_slow.add_argument("--last-run", required=True, help="path to check-run report json or run directory")
    triage_slow.add_argument("--top", type=int, default=10, help="number of slow checks to include")
    triage_slow.add_argument("--json", action="store_true", help="emit JSON output")
    triage_fail = parser_sub.add_parser("triage-failures", help="group failing checks by domain/area from a check-run report")
    triage_fail.add_argument("--last-run", required=True, help="path to check-run report json or run directory")
    triage_fail.add_argument("--json", action="store_true", help="emit JSON output")
    explain = parser_sub.add_parser("explain", help="explain a check id")
    explain.add_argument("check_id")
    explain.add_argument("--legacy-id", action="store_true", help="allow legacy dotted check ids for transition period")
    doctor = parser_sub.add_parser("doctor", help="validate checks registry integrity and canonical metadata")
    doctor.add_argument("--json", action="store_true", help="emit JSON output")


def configure(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    configure_check_parser(sub)

__all__ = ["configure", "configure_check_parser", "configure_checks_parser"]
