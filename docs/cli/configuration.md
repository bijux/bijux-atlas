# CLI Configuration

Default config file path:

- `$XDG_CONFIG_HOME/bijux-dev-atlas/config.toml`
- fallback: `~/.config/bijux-dev-atlas/config.toml`

Supported fields:

- `repo_root`
- `output_format` (`human|json|both`)
- `quiet`
- `profile`

Use wrapper:

```bash
python3 tools/cli/atlas_cli_runner.py --print-effective-config
python3 tools/cli/atlas_cli_runner.py -- api list
```
