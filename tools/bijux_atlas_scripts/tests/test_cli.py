from bijux_atlas_scripts.cli import build_parser


def test_parser_run_subcommand() -> None:
    parser = build_parser()
    ns = parser.parse_args(["run", "scripts/areas/layout/render_public_help.py", "--mode", "list"])
    assert ns.cmd == "run"
    assert ns.script.endswith("render_public_help.py")


def test_parser_validate_output_subcommand() -> None:
    parser = build_parser()
    ns = parser.parse_args(["validate-output", "--schema", "a.json", "--file", "b.json"])
    assert ns.cmd == "validate-output"
