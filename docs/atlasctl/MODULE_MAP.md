# Atlasctl Module Map

- `atlasctl/cli/main.py`: top-level CLI entrypoint, context setup, and command dispatch.
- `atlasctl/cli/constants.py`: parser registration tables and domain constants.
- `atlasctl/cli/output.py`: canonical CLI payload envelope + emission helpers.
- `atlasctl/commands/check/*`: check command parser/run split and compatibility bridge.
- `atlasctl/commands/docs/*`: docs command parser/run/generate/validate split entrypoints.
- `atlasctl/commands/ops/*`: ops command split entrypoints by operation intent.
- `atlasctl/checks/registry.py`: source of truth for registered checks/domains.
- `atlasctl/checks/runner.py`: native check execution orchestration.
- `atlasctl/checks/engine.py`: shared function/command check execution primitives.
- `atlasctl/checks/repo/*`: repo-domain checks (paths/effects/shell-shims/policies/module-size).
- `atlasctl/contracts/output.py`: JSON output validation contract entrypoint.
- `atlasctl/core/arg.py`: shared argparse helper builders.
- `atlasctl/core/clock.py`: timestamp formatting rules.
- `atlasctl/core/serialize.py`: stable JSON serialization rules.
- `atlasctl/core/types.py`: shared path/evidence dataclasses.
- `atlasctl/core/result.py`: internal `Ok/Err` result helpers.
- `atlasctl/legacy/command.py`: deprecated legacy audit/check command surface.
