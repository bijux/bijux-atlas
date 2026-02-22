from __future__ import annotations


def format_public_help(raw_help: str, public_groups: tuple[tuple[str, str], ...]) -> str:
    lines = [line for line in raw_help.splitlines() if "==SUPPRESS==" not in line]
    header: list[str] = []
    in_options = False
    for line in lines:
        if line.strip() == "options:":
            in_options = True
        if in_options:
            header.append(line)
    out = [
        "usage: atlasctl [global options] <group> ...",
        "",
        "control-plane groups:",
        *[f"  {name:<10} {desc}" for name, desc in public_groups],
        "",
        "run `atlasctl <group> --help` for group commands.",
        "",
        *header,
    ]
    return "\n".join(out).rstrip() + "\n"

