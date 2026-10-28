# Adapter principles

**If an adapter does more than translate and delegate, it is wrong.**
Read `README.md` in this directory for adapter overrides and diff scope.

## Repositories / data access
- Parameterized queries only; no string-built SQL with input.
- Select explicit columns in hand-written SQL; avoid N+1 (loops issuing queries).
- Handle "not found" explicitly and map it to a domain/application error.
- Propagate cancellation/deadlines to every database call.
- No auto-commit inside repository methods when a use case owns the transaction.
- Map rows to domain objects through the reconstitution path; handle nulls explicitly.

## HTTP / RPC / GraphQL handlers
- Parse → validate shape → map to command/query → call application → map response.
- Zero business logic and zero orchestration.
- Map application/domain errors to protocol errors deliberately (no blanket 500s,
  no leaking internal messages or stack traces).
- Never return domain or ORM objects directly as wire payloads.
- Propagate request/correlation IDs and the request's cancellation.
- Authenticate and authorize at the edge; tenant/owner scoping is never optional.

## Message consumers and jobs
- Check idempotency before doing work (inbox table, dedupe key, natural key).
- Parse defensively and validate the event version.
- Finish processing before ack; ack/nack/retry/dead-letter decisions are explicit.
- Classify errors: transient → bounded retry with backoff; permanent → dead-letter
  with context. Never dead-letter everything, never retry forever.
- No business logic in the consumer; it calls a use case.

## Webhooks (inbound)
- Verify the provider signature (HMAC or equivalent) with constant-time comparison
  before trusting the body; reject replays using timestamps or event IDs when provided.
- Persist or enqueue before acknowledging; do not do heavy work inline.

## External API clients
- Timeouts are mandatory (connect + overall). Retries are bounded, with backoff and
  jitter, only for idempotent or keyed requests.
- Respect cancellation. Translate SDK/HTTP errors into internal errors.
- Keep SDK types out of domain and application code.
- Secrets come from configuration, never source.

## Background work
- Every background task, thread, process, or goroutine has an owner, a cancellation
  path, and error reporting. Fire-and-forget work in request handlers is a defect.

## Severity guide
- `blocker`: SQL injection, missing webhook signature verification, unauthenticated
  tenant data access, ack before processing.
- `high`: missing timeout, unbounded retry, business logic in handler/consumer,
  domain object returned as JSON, fire-and-forget background work.
