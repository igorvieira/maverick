---
description: "Single ticket execution for Maverick parallel worktrees"
arguments:
  - name: ticket
    description: "Single Linear ticket ID (e.g., AP-552)"
    required: true
user_invocable: true
---

# Maverick Single - Worktree Execution

Execute development cycle for **$ARGUMENTS.ticket** in current worktree.

> This command is used by `/maverick` when running parallel worktrees.
> It assumes you're already in the correct worktree directory.

## Workflow

```
LINEAR → ARCHITECT → IMPLEMENT → QA → COMMIT → PUSH
```

### Phase 1: Fetch Task

```
mcp__linear__get_issue with id: "$ARGUMENTS.ticket"
```

Extract:
- Title, description, acceptance criteria
- Figma links (if any)
- Task type: FRONTEND or BACKEND

### Phase 2: Architecture (senior-architect)

Plan implementation:
- Files to create/modify
- Components/services needed
- Implementation steps

**Present plan and wait for approval.**

### Phase 3: Implement

Current worktree already has the feature branch. Proceed with implementation.

**FRONTEND** (senior-frontend):
```
mcp__figma__get_design_context (if Figma link)
mcp__figma__get_variable_defs (if Figma link)
```
- Use the project's design system components
- Verify: `npx tsc --noEmit`

**BACKEND** (senior-backend):
- Commands/queries in `internal/features/`
- Tests in `tests/`
- Verify: `go build ./... && go test ./...`

### Phase 4: QA Review (senior-qa)

**FRONTEND:**
- [ ] Figma compliance (tokens, spacing, typography)
- [ ] Design system components used
- [ ] TypeScript passes
- [ ] All criteria met

**BACKEND:**
- [ ] Tests pass
- [ ] Migrations work (if any)
- [ ] Error handling correct
- [ ] Events published (if any)

### Phase 5: Deliver

```bash
git add <specific-files>
git commit -m "feat($ARGUMENTS.ticket): <description>"
git push -u origin feature/$(echo "$ARGUMENTS.ticket" | tr '[:upper:]' '[:lower:]')
```

### Phase 6: Summary

Generate in English:

```markdown
## $ARGUMENTS.ticket Complete

### Branch
feature/<ticket-lowercase>

### Changes
- <bullet 1>
- <bullet 2>
- <bullet 3>

### How to Test
1. Step 1
2. Step 2
```

## Completion

Output exactly:

```
MAVERICK_COMPLETE

Ticket: $ARGUMENTS.ticket
Branch: feature/<ticket>
Status: Pushed and ready for PR
```

## Approval & Progress Reports

### Single Approval
- **ONE checkpoint only**: After presenting the implementation plan
- After approval: **FULLY AUTONOMOUS** - no interruptions

### Progress Report Schedule
| Elapsed | Frequency | Format |
|---------|-----------|--------|
| 0-5 min | Silent | - |
| 5-20 min | Every 2 min | `📍 [HH:MM] Phase: X \| Working on: <task>` |
| 20-30 min | Every 5 min | Status + current phase |
| 30+ min | Every 15 min | Detailed report |

---

## Rules

1. **ONE approval only** - then fully autonomous
2. **No interruptions** - never ask after approval
3. **Progress reports** - follow time schedule
4. Assume worktree is already set up
5. Branch already exists in worktree
6. Work independently of other tickets
7. No Co-Authored-By in commits
8. Summary in English with bullets
9. Make decisions autonomously - document, don't ask
