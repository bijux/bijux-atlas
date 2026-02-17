# Bijux Plugin Conformance Checklist

Use this checklist for each `bijux-<subsystem>` plugin release.

- [ ] Binary name follows `bijux-<subsystem>`.
- [ ] Plugin is discoverable via `$PATH` scan as `bijux-*` executable.
- [ ] `--bijux-plugin-metadata` returns valid JSON and exits `0`.
- [ ] Metadata includes required fields: `name`, `version`, `compatible_umbrella`, `build_hash`.
- [ ] Shared exit code policy is implemented.
- [ ] Global flags implemented: `--json`, `--quiet`, `--verbose`, `--trace`.
- [ ] Shared env vars honored: `BIJUX_LOG_LEVEL`, `BIJUX_CACHE_DIR`.
- [ ] Shared config path resolution follows `docs/plugin-spec.md`.
- [ ] Completion contract implemented via `completion <shell>`.
- [ ] Help output uses standard section order and template.
- [ ] Subcommand namespace avoids reserved umbrella verbs.
- [ ] Machine error contract is stable and emitted on stderr in `--json` mode.
- [ ] Unknown fields are rejected for machine contracts where required.
- [ ] Contract tests cover metadata, error schema, and exit codes.
