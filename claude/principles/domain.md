# Domain model principles

**If the domain model allows invalid business states, it is wrong.**
Read `README.md` in this directory for adapter overrides and diff scope.

## Aggregates
- An aggregate is a consistency boundary; it enforces all of its invariants internally.
- Callers never coordinate rules or mutate state directly: no public setters, no
  writable fields, no exposing internal collections (return copies or read-only views).
- Expose intent-based operations (verbs such as `cancel`, `approve`), not CRUD.
- Reference other aggregates by ID only. Children are reached through the root.
- Keep aggregates small enough to load and save in one transaction.

## Entities and value objects
- Entities have identity and lifecycle; equality is by identity.
- Value objects are immutable, self-validating, and compared by value.
- Use value objects (or the language's nominal wrapper: newtype, branded type, struct)
  for IDs, money (amount + currency, integer minor units or an exact decimal type, never
  binary floating point), quantities, email, phone, country, currency, and dates or
  times with business meaning. Mixing two ID kinds must fail to compile or validate.

## Construction
- Factories validate preconditions and return an error/result instead of an invalid object.
- Reconstitution from persistence is a separate path that does not re-run creation rules
  and is not callable from application code.

## State machines
- States are an explicit closed set (enum, union, atom set), not free strings.
- Transitions are operations with guards; an invalid transition returns a domain error.

## Domain events
- Past-tense facts (`OrderPlaced`), immutable, with minimal payload (IDs, timestamps,
  changed values). Never commands, never whole aggregates.
- The aggregate records events internally; the application layer harvests them.

## Domain errors
- Expected business failures are typed or sentinel errors with stable identities that
  callers can match. No generic "failed"/"error" messages.

## Purity
- The domain imports no ORM, HTTP, serialization, queue, or framework code. Time and
  randomness come from parameters or an injected clock/ID source, not global calls.

## Severity guide
- `blocker`: invalid state reachable, invariant enforced only by callers, money in floats.
- `high`: primitive where a value object is mandatory (new code), public mutation of
  aggregate state, events carrying whole aggregates.
- `medium`: missing factory validation for an edge case, anemic model with logic in services.
