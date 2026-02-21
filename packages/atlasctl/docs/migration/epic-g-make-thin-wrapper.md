# Epic G Make Logic Inventory

Date: 2026-02-21
Source: `atlasctl make inventory-logic --json`

Total non-delegating recipe lines: 1548

Top targets with direct logic:
- `ops-local-full`: 46
- `stack-full`: 38
- `RUN_ID`: 30
- `ops-full`: 26
- `ops-k8s-tests`: 23
- `culprits-max_loc`: 19
- `culprits-max_loc-py`: 19
- `ci-security-advisory-render`: 17
- `governance-check`: 17
- `ops-release-update`: 17
- `ops-report`: 17
- `ops-ref-grade-local`: 16
- `ops-minio-ready`: 15
- `print-env`: 14
- `ops-deploy`: 13
- `ops-ci`: 13
- `culprits-file-max_rs_files_per_dir`: 13
- `culprits-file-max_modules_per_dir`: 13
- `ops-reset`: 12
- `docker-build`: 12

Migration rule

- New/updated targets should delegate to `atlasctl make run <target>` or direct `atlasctl` command entrypoints.
- Direct script/python/artifact-write exceptions are tracked in `configs/make/delegation-exceptions.json` and should trend down over time.
