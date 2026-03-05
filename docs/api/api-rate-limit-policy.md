# API Rate Limit Policy

- Owner: `api-contracts`
- Type: `policy`
- Audience: `user`
- Stability: `stable`

## Policy

- Default request budgets are enforced per endpoint class.
- Rate-limited responses must return documented error shape.
- Clients should use bounded retries with backoff.
