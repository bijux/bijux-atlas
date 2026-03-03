# Warmup Lock Model

- Owner: `bijux-atlas-operations`
- Review cadence: `quarterly`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@a10951a3e4e65b3b9be3bb67b16b4dc16a6d5287`
- Last changed: `2026-03-03`
- Reason to exist: explain the startup warmup lock used to coordinate Redis-backed dataset warmup.

## Purpose

The warmup lock prevents multiple pods from prewarming the same dataset at the same time during
startup.

## Behavior

- Lock acquisition uses one atomic Redis `SET key value NX EX <ttl>` command.
- The lock value is unique per process.
- Release only deletes the key when the stored value still matches the original owner.
- If a process crashes after acquiring the lock, the TTL allows a later pod to recover.
- Retries use bounded jitter to avoid synchronized contention.

## Failure Modes

- If Redis is unavailable, startup falls back to local warmup behavior.
- If the lock expires before release, the owner logs the expiry and avoids deleting a newer owner.
