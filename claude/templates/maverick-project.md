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
packs: none            <!-- e.g., go -->
frontend: none         <!-- e.g., react-nextjs | vue | none -->

## Commands

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
