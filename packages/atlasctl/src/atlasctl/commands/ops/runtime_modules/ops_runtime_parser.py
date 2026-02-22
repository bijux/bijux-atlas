from __future__ import annotations

import argparse

def configure_ops_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("ops", help="ops control-plane command surface")
    p.add_argument("--list", action="store_true", help="list available ops commands")
    p.add_argument("--json", action="store_true", help="emit machine-readable JSON output")
    ops_sub = p.add_subparsers(dest="ops_cmd", required=False)

    check = ops_sub.add_parser("check", help="run canonical ops/check lane")
    check.add_argument("--report", choices=["text", "json"], default="text")
    check.add_argument("--fix", action="store_true")
    check.add_argument("--all", action="store_true", help="include slow/full ops validations")
    help_cmd = ops_sub.add_parser("help", help="show canonical ops runbook index")
    help_cmd.add_argument("--report", choices=["text", "json"], default="text")
    up_cmd = ops_sub.add_parser("up", help="bring up full local ops environment")
    up_cmd.add_argument("--report", choices=["text", "json"], default="text")
    down_cmd = ops_sub.add_parser("down", help="tear down full local ops environment")
    down_cmd.add_argument("--report", choices=["text", "json"], default="text")
    restart_cmd = ops_sub.add_parser("restart", help="restart deployed atlas workloads safely")
    restart_cmd.add_argument("--report", choices=["text", "json"], default="text")
    deploy_cmd = ops_sub.add_parser("deploy", help="deploy atlas workloads")
    deploy_cmd.add_argument("--report", choices=["text", "json"], default="text")
    run_cmd = ops_sub.add_parser("run", help="run ops workflow manifest")
    run_cmd.add_argument("--report", choices=["text", "json"], default="text")
    run_cmd.add_argument("--manifest", required=True, help="ops workflow manifest path (.json/.yaml)")
    run_cmd.add_argument("--fail-fast", action="store_true", help="stop on first failing manifest step")

    lint = ops_sub.add_parser("lint", help="run canonical ops lint checks")
    lint.add_argument("--report", choices=["text", "json"], default="text")
    lint.add_argument("--fail-fast", action="store_true")
    lint.add_argument("--emit-artifacts", action="store_true")
    lint.add_argument("--fix", action="store_true")
    lint.add_argument("--all", action="store_true", help="include full lint validation set")

    env = ops_sub.add_parser("env", help="ops environment commands")
    env.add_argument("--report", choices=["text", "json"], default="text")
    env_sub = env.add_subparsers(dest="ops_env_cmd", required=True)
    env_validate = env_sub.add_parser("validate", help="validate ops env contract")
    env_validate.add_argument("--schema", default="configs/ops/env.schema.json")
    env_print = env_sub.add_parser("print", help="print resolved ops env settings")
    env_print.add_argument("--schema", default="configs/ops/env.schema.json")
    env_print.add_argument("--format", choices=["json", "text"], default="json")

    pins = ops_sub.add_parser("pins", help="ops pins commands")
    pins.add_argument("--report", choices=["text", "json"], default="text")
    pins_sub = pins.add_subparsers(dest="ops_pins_cmd", required=True)
    pins_sub.add_parser("check", help="validate pinned ops versions and drift contracts")
    pins_sub.add_parser("update", help="update ops pins")

    gen = ops_sub.add_parser("gen", help="ops generated artifacts commands")
    gen.add_argument("--report", choices=["text", "json"], default="text")
    gen_sub = gen.add_subparsers(dest="ops_gen_cmd", required=False)
    gen_sub.add_parser("run", help="regenerate committed ops outputs")
    gen_sub.add_parser("check", help="regenerate then fail on drift")

    stack = ops_sub.add_parser("stack", help="ops stack commands")
    stack.add_argument("--report", choices=["text", "json"], default="text")
    stack_sub = stack.add_subparsers(dest="ops_stack_cmd", required=True)
    stack_sub.add_parser("versions-sync", help="sync stack versions json from tool versions SSOT")
    stack_up = stack_sub.add_parser("up", help="bring up stack components")
    stack_up.add_argument("--profile", default="kind")
    stack_sub.add_parser("down", help="tear down stack components")
    stack_sub.add_parser("restart", help="restart atlas deployment")

    k8s = ops_sub.add_parser("k8s", help="ops kubernetes commands")
    k8s.add_argument("--report", choices=["text", "json"], default="text")
    k8s_sub = k8s.add_subparsers(dest="ops_k8s_cmd", required=True)
    k8s_sub.add_parser("contracts", help="validate k8s contracts")

    e2e = ops_sub.add_parser("e2e", help="ops end-to-end commands")
    e2e.add_argument("--report", choices=["text", "json"], default="text")
    e2e_sub = e2e.add_subparsers(dest="ops_e2e_cmd", required=True)
    e2e_sub.add_parser("validate", help="validate e2e scenarios and suites")
    e2e_run = e2e_sub.add_parser("run", help="run e2e suite")
    e2e_run.add_argument("--suite", choices=["smoke", "k8s-suite", "realdata"], default="smoke")

    obs = ops_sub.add_parser("obs", help="ops observability commands")
    obs.add_argument("--report", choices=["text", "json"], default="text")
    obs_sub = obs.add_subparsers(dest="ops_obs_cmd", required=True)
    obs_sub.add_parser("verify", help="run observability verification")
    obs_drill = obs_sub.add_parser("drill", help="run one observability drill")
    obs_drill.add_argument("--drill", required=True)

    kind = ops_sub.add_parser("kind", help="kind substrate commands")
    kind.add_argument("--report", choices=["text", "json"], default="text")
    kind_sub = kind.add_subparsers(dest="ops_kind_cmd", required=True)
    kind_sub.add_parser("up", help="create kind cluster")
    kind_sub.add_parser("down", help="delete kind cluster")
    kind_sub.add_parser("reset", help="reset kind cluster")
    kind_sub.add_parser("validate", help="validate kind substrate contracts")
    kind_fault = kind_sub.add_parser("fault", help="inject kind fault")
    kind_fault.add_argument("name", choices=["disk-pressure", "latency", "cpu-throttle"])

    load = ops_sub.add_parser("load", help="ops load commands")
    load.add_argument("--report", choices=["text", "json"], default="text")
    load_sub = load.add_subparsers(dest="ops_load_cmd", required=True)
    load_run = load_sub.add_parser("run", help="run load suite")
    load_run.add_argument("--suite", default="mixed-80-20")

    datasets = ops_sub.add_parser("datasets", help="ops dataset commands")
    datasets.add_argument("--report", choices=["text", "json"], default="text")
    datasets_sub = datasets.add_subparsers(dest="ops_datasets_cmd", required=True)
    datasets_sub.add_parser("verify", help="verify dataset state")
    datasets_sub.add_parser("fetch", help="warm/fetch datasets")
    datasets_sub.add_parser("pin", help="rebuild datasets lock manifest")

    for name, help_text in (
        ("surface", "validate or generate ops surface metadata"),
        ("contracts-check", "validate ops contracts index and schema pairs"),
        ("suites-check", "validate ops suite references"),
        ("schema-check", "validate ops schema contracts"),
        ("tool-versions-check", "validate pinned ops tool versions"),
        ("no-direct-script-usage-check", "validate direct ops script usage policy"),
        ("directory-budgets-check", "validate ops-related directory budgets"),
        ("naming-check", "validate ops naming conventions"),
        ("layer-drift-check", "validate cross-layer drift rules"),
        ("contracts-index", "generate ops contracts docs index"),
        ("policy-audit", "validate ops policy configs reflected in ops usage"),
        ("k8s-surface-generate", "generate k8s test surface docs from manifest"),
        ("k8s-checks-layout", "validate k8s checks layout budget"),
        ("k8s-test-lib-contract", "validate k8s tests checks/_lib helper contract"),
        ("k8s-flakes-check", "evaluate k8s flake report and quarantine policy"),
        ("k8s-test-contract", "validate k8s test manifest ownership/contract"),
        ("clean-generated", "remove runtime evidence files under ops/_generated"),
        ("clean", "alias for clean-generated"),
    ):
        cmd = ops_sub.add_parser(name, help=help_text)
        cmd.add_argument("--report", choices=["text", "json"], default="text")
        cmd.add_argument("--fix", action="store_true")
        if name in {"clean-generated", "clean"}:
            cmd.add_argument("--force", action="store_true")
