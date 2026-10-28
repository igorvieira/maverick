# Shared backend principles

Language-neutral review rules shared by every Maverick language pack. Pack prompts
point here and add only language-specific idioms, libraries, and failure modes.
The Go pack predates this directory and carries the same rules inline.

| File | Applies to reviewer slot |
|---|---|
| `domain.md` | `domain` |
| `application-flow.md` | `application_flow` |
| `adapters.md` | `adapter` |
| `eventing.md` | `eventing` |
| `architecture.md` | `architecture` |
| `red-team.md` | red-team gate (`agents.red_team`) |

Installed path: `.claude/maverick/principles/`. Source path: `claude/principles/`.

## Adapter overrides (applies to every file)

Principles describe the target design, not a rewrite mandate. Before grading, read the
project adapter (`.claude/maverick/project.md`, and `project.json` when present):

1. **Accepted variants.** If the adapter declares an accepted variant (for example
   "ORM entities are used directly inside commands", "IDs are plain UUID strings",
   "events are published from an after-commit hook"), a rule that conflicts with it is
   reported as `low`/`info` with the rule name, never `high` or `blocker`.
2. **Diff scope.** Grade only code the change adds or modifies. Pre-existing violations
   in untouched code are listed once under "Existing debt" as `info`.
3. **New code still meets the floor.** Variants never excuse data loss: events lost on
   failed publish, non-idempotent consumers of at-least-once queues, unbounded retries,
   external calls without timeouts, and untracked background work stay `high` or above
   in new code unless the adapter explicitly lists them as accepted risks with an owner.
4. **Existing abstractions win.** Use the project's transaction helper, outbox, clock,
   error type, and test harness. Never introduce a parallel one.
