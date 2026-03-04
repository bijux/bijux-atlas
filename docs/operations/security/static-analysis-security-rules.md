# Static Analysis Security Rules

Security-focused static checks must pass in CI:

- forbidden literal and secret pattern scanning for release evidence artifacts
- workflow pinning and exception expiry validation
- policy contract schema validation for security config surfaces
- shell network fetch guardrail checks for unreviewed `curl|bash` patterns

Primary command:

- `bijux-dev-atlas security validate`
