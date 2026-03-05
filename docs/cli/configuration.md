# CLI Configuration

Default config file path:

- `$XDG_CONFIG_HOME/bijux-dev-atlas/config.toml`
- fallback: `~/.config/bijux-dev-atlas/config.toml`

Supported fields:

- `repo_root`
- `output_format` (`human|json|both`)
- `quiet`
- `profile`

Use dev-atlas directly:

```bash
bijux-dev-atlas configs print --format json
bijux-dev-atlas api list --format json
```
