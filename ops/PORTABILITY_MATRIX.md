# Ops Portability Matrix

- Owner: `bijux-atlas-operations`
- Purpose: define portability guarantees across profiles, environments, and toolchains.
- Consumers: `checks_ops_portability_environment_contract`

## Platform Matrix

- macos-runner
- minimal-linux-container
- air-gapped-simulation
- degraded-stack-mode
- partial-dataset-mode
- multi-registry
- alternate-storage-backend

## Environment Modes

- local-only
- remote-execution
- container-only-toolchain

## Resource Pressure and Fault Simulation

- cpu-limited
- memory-limited
- slow-network-simulation
- time-skew-simulation
- missing-dependency-simulation

## Path Portability Invariants

- portable-path validation
- all paths are repo-relative with forward slashes

## Enforcement Links

- checks_ops_portability_environment_contract

## Runtime Evidence Mapping

- ops/k8s/install-matrix.json
- ops/stack/profiles.json
- ops/load/scenarios/noisy-neighbor-cpu-throttle.json
- ops/observe/contracts/goldens/profiles.json
