---
name: maverick
description: Use this skill when the user asks for Maverick, autonomous end-to-end development, Linear-ticket implementation, Figma-to-code work, senior frontend/backend/security/QA review, or a no-regressions implementation workflow in Codex.
---

# Maverick For Codex

Maverick is a disciplined end-to-end development workflow for Codex. It supports Linear tickets, local tasks, Figma-backed UI work, and review/QA passes.

## Core Policy

- No regressions: preserve existing behavior unless the user explicitly authorizes a change.
- Read before editing: inspect the codebase, tests, and local patterns first.
- Keep changes scoped: avoid opportunistic refactors.
- Validate honestly: run relevant checks and report anything skipped or blocked.
- Use Codex subagents only when the user explicitly asks for parallel agents or delegation.

## Modes

### Linear Ticket

Use when the request includes one or more issue IDs, such as `TICKET-123`.

1. Fetch the full ticket with Linear MCP when available.
2. Extract goal, acceptance criteria, links, affected surfaces, and ambiguity.
3. Inspect linked Figma designs when present.
4. Plan implementation and validation.
5. Implement, verify, and prepare delivery notes.

### Local Task

Use when the request includes `--local` or a natural-language task.

1. Convert the request into goal, scope, likely files, and acceptance criteria.
2. Search the codebase for nearby patterns.
3. Implement the smallest coherent change.
4. Verify with targeted checks.

### Multi-Task

For multiple independent tickets or tasks:

- If the user explicitly authorized parallel agents, split work by repository, service, or file ownership and use disjoint write scopes.
- Otherwise, execute sequentially.
- Use worktrees only when multiple branches need to progress independently.

## Declarative Language Packs

When packs are installed, read `.claude/maverick/packs/<id>.json` and the project
adapter. JSON is the authoritative manifest; `<id>.md` is a compatibility pointer.
Use an optional `.claude/maverick/project.json` for explicit pack selection and command
overrides; keep Markdown-only adapters working. Explicit selection precedes manifest
file detection. Missing or invalid explicit packs must be reported as errors.

Use planner/implementer slots from the manifest. Route reviewers in declaration order
using `always` or any intersection with the extensible `classification.json` capabilities.
Do not classify tasks into a closed frontend/backend enum or select by agent name.
Keep each reviewer's semantics in its prompt and scope reviewer IDs by pack.

Evaluate red-team gates and fix-loop policies from JSON. Structured fail or blocker/high
findings block; use the manifest's legacy verdict mapping only for transitional prose.
Missing or malformed required reviews remain unresolved. Record warning resolutions.
Resolve commands per pack: adapter override → pack default → absent; per-pack adapter
overrides win over shared ones. Never invent missing commands. The experimental
`maverick pack` CLI loads and routes but does not execute commands or agents.

If no pack matches, retain the role playbooks below and explicitly report that no
specialist panel ran. Do not change the existing rule requiring user authorization
before using parallel subagents.

## Role Playbooks

### Architect

- Map requirements to code areas.
- Identify contracts, migrations, data flow, UI states, and risk.
- Produce a short implementation plan with validation.

### Frontend

- Prefer the project's design system and existing component patterns.
- Check responsive behavior, loading/error/empty states, accessibility, and visual parity.
- Use Figma MCP context and screenshots when available.

### Backend

- Preserve API contracts and database compatibility.
- Check validation, idempotency, auth, observability, and failure modes.
- Add or update focused tests around changed behavior.

### Security

- Look for auth bypasses, injection, secret exposure, unsafe deserialization, dependency risk, and logging leaks.
- Treat security findings as blockers unless the user explicitly accepts the risk.

### QA

- Run targeted tests first, then broader suites when shared behavior changed.
- For UI work, verify important viewport states when browser tooling is available.
- Compare intended behavior against acceptance criteria.

## Delivery Format

End with:

- Summary of behavior changed.
- Files changed.
- Tests or checks run.
- Remaining risks, blockers, or manual verification needed.

For Linear tasks, also prepare a comment:

```markdown
## Implementation Complete

### Modified Files
- `path/to/file` - Change description

### How to Test
1. Step
2. Step

### Notes
- Relevant implementation notes
```
