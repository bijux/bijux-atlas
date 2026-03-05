# CLI Usage Guide

## Discover

```bash
python3 tools/cli/discover_subcommands.py --format text
```

## Configure defaults

```bash
python3 tools/cli/atlas_cli_runner.py --print-effective-config
```

## Run commands with wrapper

```bash
python3 tools/cli/atlas_cli_runner.py -- api list
python3 tools/cli/atlas_cli_runner.py -- observe metrics list
```
