# Suite Coverage Markers

`packages/atlasctl/tests/goldens/check/check-suite-coverage.markers.txt` is the suite-marker SSOT for repo-check coverage hints.

Rules:

- File must exist and be non-empty.
- One check id per line.
- Lines are sorted lexicographically and unique.
- Every marker must map to a registered canonical `checks_<domain>_<area>_<name>` check id.

The `checks_repo_suite_marker_rules` and `checks_repo_check_test_coverage` checks enforce these rules.
