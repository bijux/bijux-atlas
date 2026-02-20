from pathlib import Path


def test_help_golden_has_core_targets() -> None:
    golden = Path(__file__).resolve().parent / "goldens" / "help.expected.txt"
    text = golden.read_text(encoding="utf-8")
    for target in ("root", "root-local", "ci", "nightly"):
        assert target in text
