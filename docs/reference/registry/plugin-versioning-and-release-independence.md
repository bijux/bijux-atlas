# Plugin Versioning And Release Independence

## Independent Versioning

- Umbrella (`bijux`) and plugins (`bijux-*`) version independently.
- Compatibility is enforced via plugin metadata field `compatible_umbrella`.
- Umbrella must refuse incompatible plugins at dispatch time.

## Release Cadence Independence

- Plugins may ship patch/minor releases without umbrella release.
- Umbrella may release without forcing plugin rebuilds.
- Only compatibility-range changes require coordinated announcement.
