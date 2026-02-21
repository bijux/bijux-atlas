# Install Atlasctl

`atlasctl` is published as a Python package and can be installed as a tool.

## pipx (recommended for CLI usage)

```bash
pipx install atlasctl
atlasctl --version
```

## uv tool

```bash
uv tool install atlasctl
atlasctl --version
```

## From local source (development)

```bash
python3 -m venv .venv
. .venv/bin/activate
pip install -e packages/atlasctl
atlasctl --version
python -m atlasctl --help
```
