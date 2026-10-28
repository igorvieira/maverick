# Eventing principles

**If an event cannot be safely replayed, versioned, or understood without reading the
producer's code, it is wrong.**
Read `README.md` in this directory for adapter overrides and diff scope.

## What an event is
- A fact in past tense (`InvoicePaid`), never a command (`ChargeCustomer`).
- Immutable: corrections are new events.
- Owned by one bounded context. Domain events are internal by default; integration
  events are public contracts.

## Payload
- Minimal: IDs, timestamps, changed values consumers need.
- No internal domain structures, ORM fields, or database surrogate keys.
- Money carries currency; times carry a timezone (UTC).
- Include an event ID, type, version, occurred-at, and correlation/causation IDs in the
  envelope (or the project's existing envelope).

## Versioning
- Integration events are versioned explicitly (v1, v2).
- Compatible changes only: add optional fields. Removing or renaming fields, or
  changing meaning, is a new event type or version.

## Delivery
- Assume at-least-once delivery and out-of-order arrival.
- Producers publish after commit via outbox, or an after-commit hook whose failures are
  retried or recorded, never only logged.
- The outbox has a relay and monitoring for stuck rows.
- Consumers are idempotent and tolerate duplicates and reordering (version checks,
  last-write-wins with timestamps, or state guards).
- Retry and dead-letter policies are defined per consumer.

## Severity guide
- `blocker`: event published before commit, breaking change to a public event in place,
  consumer that double-applies money movements on redelivery.
- `high`: command disguised as event, full aggregate in payload, publish failure only
  logged, no version on an integration event.
