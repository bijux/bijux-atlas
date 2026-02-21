from tests.helpers import golden_path


def test_help_golden_has_core_targets() -> None:
    golden = golden_path("help.expected.txt")
    text = golden.read_text(encoding="utf-8")
    for target in ("root", "root-local", "ci", "nightly"):
        assert target in text
