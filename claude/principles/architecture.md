# Architecture principles

**Source code dependencies point inward.**
Read `README.md` in this directory for adapter overrides and diff scope.

```
transport (HTTP/RPC/queue/CLI) → application → domain ← infrastructure (DB, clients)
```

## Boundaries
- Domain has no infrastructure or framework dependencies.
- Application orchestrates; it defines the ports it needs (repositories, clients,
  publishers). Infrastructure implements them.
- Ports live in the application layer by default; in the domain only when justified.
- Read-heavy paths may use dedicated read models (CQRS) instead of aggregates.

## Consistency
- Declare per operation: strong or eventual consistency.
- Prefer single-aggregate transactions. Cross-aggregate or cross-service changes
  choose explicitly between merge, shared transaction with a documented invariant,
  or saga/process manager with compensations.
- Document the locking strategy (optimistic version or pessimistic lock) for contended data.

## Concurrency and lifecycle
- No unowned background work. Every task/thread/process has cancellation, error
  propagation, and shutdown handling.
- No blocking calls on event loops or async runtimes.
- Global mutable state and singletons holding request data are defects.
- No "background/empty" contexts or detached cancellation tokens on production request paths.

## Production reality
- Structured logs with correlation IDs; no secrets or full PII in logs.
- Errors are actionable: what failed, why, what the caller can do.
- Metrics on business actions, not only infrastructure.
- Migrations are backward compatible (expand → migrate → contract) and safe to run
  while the old version is serving traffic.
- Graceful degradation when a dependency fails; timeouts everywhere.
- Multi-tenant data is scoped at the query level, not only in the UI.

## Anti-slide rule
When asked for an "ideal structure", give a minimal baseline, two or three valid
variants, and trade-offs. Never claim one universal layout.

## Severity guide
- `blocker`: domain importing infrastructure in new code (unless an accepted variant),
  cross-service consistency without saga or compensation, tenant isolation gaps,
  non-backward-compatible migration on a live table.
- `high`: unowned background work, missing timeouts, business logic in transport,
  undocumented locking on contended money data.
