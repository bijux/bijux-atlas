# Dev Control Plane Exit Codes

## Process Exit Codes

- `0`: success
- `1`: command execution error / contract failure / internal error
- `2`: usage error (argument parsing) or wrapped make target policy failure when invoked through `make`

## Check Report Exit Mapping (check engine)

- `ok` -> `0`
- `fail` -> stable non-zero check failure exit (rendered through `exit_code_for_report`)
- `error` -> stable non-zero execution error exit (distinct from `fail`)
- `skip` -> represented in results and counts; does not fail the process by itself

## Notes

- Command-level JSON payloads may also include domain-specific `error_code` values (for example docs/configs/ops contracts).
- Prefer machine-readable payload `status` and `error_code` fields over shell exit codes when integrating in CI dashboards.

