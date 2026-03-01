# Exit Codes

`bijux-atlas-cli` uses stable process exit codes from `bijux-atlas-core::ExitCode`.

- `0` `Success`: command completed successfully.
- `2` `Usage`: invalid arguments, unsupported command shape, or contract violation reported to caller.
- `3` `DependencyFailure`: dependent component failed.
- `4` `Internal`: unexpected internal error.

Machine-readable failures with `--json` emit a stable error payload with a `code` and `message`.
