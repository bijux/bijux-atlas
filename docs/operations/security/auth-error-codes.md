# Auth Error Codes

Atlas uses dedicated auth failures instead of generic policy rejections.

## Codes

- `AuthenticationRequired` (`401`): the request reached a protected route without valid
  credentials for the configured auth mode.
- `AccessForbidden` (`403`): the request authenticated or resolved to a known principal path, but
  the access policy denied the requested action.

## Remediation

- For `AuthenticationRequired`, provide the required API key or HMAC headers, or route the request
  through the configured ingress auth boundary.
- For `AccessForbidden`, use a principal that is allowed by the governed access policy or change
  the deployment boundary so the request reaches the intended trust zone.
