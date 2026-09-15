# Maverick Project Adapter

Per-repo facts the Maverick workflow and agent packs must follow. This file lives in the
**consuming repository** at `.claude/maverick/project.md` and is read at the start of every
Maverick run (Phase 0). It is the only place project-specific knowledge should live — the
maverick skill, commands, and packs stay generic.

Rules for this file:
- Facts here **override** any default in the maverick skill or packs.
- Keep it factual and short — agents read it verbatim.
- **NEVER put secrets here** (tokens, passwords, connection strings). Internal names/hosts
  belong here only if this repository is private.

---

## Stack & Packs

<!-- Agent packs installed in .claude/agents/ that Maverick should use -->
packs: none            <!-- ordered list, e.g., go, typescript -->
frontend: none         <!-- e.g., react-nextjs | vue | none -->

## Structured Companion (optional)

You may create `.claude/maverick/project.json` using schema version 1. When present,
it owns selected packs and command overrides; do not duplicate those fields here.
It may also hold conventions and layout; keep each fact in one place. This Markdown
file remains the context source for facts not in the companion. Existing Markdown-only
adapters continue to work with the workflow. The Rust CLI reads the explicit JSON file.

```json
{
  "schema_version": 1,
  "packs": [],
  "commands": {},
  "pack_commands": {},
  "project_commands": {},
  "conventions": [],
  "layout": {}
}
```

`packs` is an ordered array of manifest IDs. An explicit empty array disables automatic
selection. Use `pack_commands` for overrides scoped by pack ID, `commands` for shared
overrides, and `project_commands` for named project-only commands (for example migrations).

## Commands

Precedence, per command: project adapter override → language pack default → absent.
Within the adapter, per-pack overrides win over shared commands. Resolve each pack
separately. Omitted/null fields inherit; blank commands are invalid. Missing commands
stay absent: report the gap, never invent or execute a substitute. Commands are trusted
project configuration and must be checked before execution; the Rust CLI only reports them.


build: <e.g., go build ./... | pnpm build>
test: <e.g., go test ./... | pnpm test>
lint: <e.g., make lint | pnpm lint>
typecheck: <e.g., npx tsc --noEmit | n/a>

## Repository Layout & Conventions

- <where commands/queries/handlers/components live, e.g., "commands in internal/commands/, queries in internal/queries/ (CQRS)">
- <API contract style, e.g., "gRPC between services, GraphQL at the gateway">

## Migrations & Codegen

- <workflow, e.g., "edit the ORM schema, then run `make generate`; create migrations with the migration tool — never hand-write them">

## Error Handling & Logging

- <error-wrapping library and conventions>
- <logging conventions>

## Testing Strategy

- <unit vs integration approach; required local tooling, e.g., "integration tests spin up embedded postgres — full `postgres` binary must be on PATH">

## Tickets & Branches

ticket_prefix: TICKET      <!-- your tracker's prefix -->
branch_source: slug        <!-- linear = always use the tracker's suggested branch name | slug = generate from description -->
default_branch: main

## PR & Review

- <PR body conventions, required status checks, review bots to expect during the review window>
- <commit message conventions>

## Docs Pointers

<!-- Files agents should read for deeper context before planning/implementing -->
- docs/ARCHITECTURE.md
- docs/CONVENTIONS.md

## Environment Notes (optional, non-secret)

- <how to run the app locally, feature-flag conventions, seeded test accounts by NAME only, etc.>
