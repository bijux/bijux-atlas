# Compatibility Matrix: bijux Umbrella <-> bijux-atlas

| bijux umbrella | bijux-atlas plugin | status | notes |
|---|---|---|---|
| `0.1.x` | `0.1.x` | supported | plugin advertises `compatible_umbrella: >=0.1.0,<0.2.0` |
| `0.2.x` | `0.1.x` | unsupported | plugin handshake must fail compatibility check |

## Validation Rule

The umbrella validates plugin metadata range against umbrella semver before dispatch.
If incompatible, the umbrella returns a structured machine error and does not execute plugin commands.
