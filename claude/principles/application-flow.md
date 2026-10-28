# Application flow principles

**If the flow only works by accident, it is wrong.**
Read `README.md` in this directory for adapter overrides and diff scope.

## Orchestration
- Use cases (commands, services, interactors, contexts) coordinate; the domain decides.
- One use case = one business capability with explicit input and output types.
- Commands change state; queries never change state. Queries may bypass aggregates
  and read projections directly.

## Transaction ownership (non-negotiable)
- The use case owns the transaction boundary: not the handler, not the repository,
  not an ORM callback.
- Domain operations run inside it; the use case commits or rolls back.
- One transaction per use case; touching several aggregates needs an explicit decision
  (merge, shared invariant with strong consistency, or saga/process manager).

## Side-effect ordering (non-negotiable)
1. Load state.
2. Run domain behavior (events collected in the aggregate).
3. Persist state.
4. Store outbox rows in the same transaction (when the project has an outbox).
5. Commit.
6. Clear harvested events only after a successful commit.
7. Trigger async side effects (emails, HTTP calls, publishes without outbox) only after commit.

Events, emails, and external calls are never emitted before commit. A side effect that
fails after commit must be retried or recorded; logging and dropping it is data loss.

## Validation split
- Application validates shape: presence, format, type coercion.
- Domain validates business rules: uniqueness, limits, allowed transitions.

## Errors
- Never swallow domain errors. Translate them to transport errors at the edge only.
- Distinguish "nothing happened" from "partially happened" in error paths.

## Idempotency
- Non-idempotent commands reachable by retries (HTTP clients, queues, webhooks, jobs)
  accept an idempotency key or rely on a natural key enforced by a unique constraint.

## Concurrency
- Concurrent updates to the same aggregate are handled explicitly: optimistic version
  check, row lock, or serialized processing. Retries on serialization failures are bounded.

## Severity guide
- `blocker`: publish/side effect before commit, transaction opened in handler and
  repository simultaneously, lost update possible on money or balances.
- `high`: business rule in the use case instead of the domain, swallowed error,
  missing idempotency on a retried command.
