# Suite Coverage Markers

`packages/atlasctl/tests/goldens/check-suite-coverage.markers.txt` is the suite-marker SSOT for repo-check coverage hints.

Rules:

- File must exist and be non-empty.
- One check id per line.
- Lines are sorted lexicographically and unique.
- Every marker must map to a registered `repo.*` check id.

The `repo.suite_marker_rules` and `repo.check_test_coverage` checks enforce these rules.
