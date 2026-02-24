---
name: maverick
description: >
  Autonomous end-to-end development workflow. Orchestrates senior agents (architect, frontend, backend, qa)
  to complete Linear tasks from planning to PR. Supports single ticket or multiple parallel tickets with
  git worktrees. Integrates with Linear MCP, Figma MCP, and ralph-loop plugin.
  Only ONE approval checkpoint (after architect planning), then fully autonomous with progress reports.
user_invocable: true
arguments:
  - name: tickets
    description: "Linear ticket(s) - single (AP-552) or multiple comma-separated (AP-552,AP-553,AP-554)"
    required: true
---

# Maverick Workflow

Autonomous development workflow for **$ARGUMENTS.tickets** using coordinated senior agents.

## Execution Modes

### Single Ticket
```bash
/maverick AP-552
# or with Ralph Loop:
/ralph-loop:ralph-loop "/maverick AP-552" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"
```

### Multiple Tickets (Parallel Worktrees)
```bash
/maverick AP-552,AP-553,AP-554
```

When multiple tickets are provided, Maverick will:
1. Create a git worktree for each ticket
2. Launch parallel Ralph loops
3. Each ticket executes independently
4. No conflicts between tickets

## Workflow Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      MAVERICK WORKFLOW                          │
├─────────────────────────────────────────────────────────────────┤
│  1. LINEAR       → Fetch task, extract requirements             │
│  2. ARCHITECT    → Analyze scope, plan implementation           │
│  3. BRANCH       → Create feature branch (or worktree)          │
│  4. IMPLEMENT    → Frontend OR Backend (based on task type)     │
│  5. QA REVIEW    → Validate implementation                      │
│  6. DELIVER      → Commit, push, summarize                      │
└─────────────────────────────────────────────────────────────────┘
```

---

## Parallel Worktrees Mode

When executing multiple tickets (`AP-552,AP-553,AP-554`):

### Setup Worktrees

```bash
# Configuration
REPO_BASE="<project-root>/<repo>"  # or backend repo
WORKTREE_BASE="<project-root>/worktrees"

mkdir -p $WORKTREE_BASE

# For each ticket
for TICKET in AP-552 AP-553 AP-554; do
  BRANCH="feature/$(echo $TICKET | tr '[:upper:]' '[:lower:]')"
  git -C $REPO_BASE worktree add $WORKTREE_BASE/$TICKET -b $BRANCH
done
```

### Launch Parallel Loops

Each worktree runs independently:

```bash
# Terminal 1
cd <project-root>/worktrees/AP-552
/ralph-loop:ralph-loop "/maverick-single AP-552" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"

# Terminal 2
cd <project-root>/worktrees/AP-553
/ralph-loop:ralph-loop "/maverick-single AP-553" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"

# Terminal 3
cd <project-root>/worktrees/AP-554
/ralph-loop:ralph-loop "/maverick-single AP-554" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"
```

### Cleanup After Completion

```bash
for TICKET in AP-552 AP-553 AP-554; do
  git -C $REPO_BASE worktree remove $WORKTREE_BASE/$TICKET
done
rmdir $WORKTREE_BASE  # if empty
```

### Parallel Benefits

- **No conflicts**: Each ticket has its own directory
- **Independent branches**: No merge issues during development
- **True parallelism**: Multiple Claude instances working simultaneously
- **Faster delivery**: N tickets in ~1x time instead of Nx time

---

## Approval & Progress Reports

### Single Approval Checkpoint

**ONLY ONE interruption allowed**: After senior-architect completes the implementation plan.

```
┌─────────────────────────────────────────────────────────────┐
│  LINEAR → ARCHITECT → [APPROVAL] → IMPLEMENT → QA → DELIVER │
│                           ↑                                  │
│                     ONLY CHECKPOINT                          │
│                  (no more interruptions)                     │
└─────────────────────────────────────────────────────────────┘
```

After user approval, execution is **FULLY AUTONOMOUS** - no more questions or confirmations.

### Progress Report Schedule

During autonomous execution, output progress reports at these intervals:

| Elapsed Time | Report Frequency | Action |
|--------------|------------------|--------|
| 0-5 min | No reports | Work silently |
| 5-20 min | Every 2 min | Brief status update |
| 20-30 min | Every 5 min | Status + current phase |
| 30+ min | Every 15 min | Detailed progress report |

### Report Format

**Brief Report (2 min intervals)**
```
📍 [HH:MM] Phase: IMPLEMENT | Working on: <current file/task>
```

**Standard Report (5 min intervals)**
```
📊 Progress Report [HH:MM]
├── Phase: IMPLEMENT (3/6)
├── Current: Writing component X
├── Completed: 2 files modified
└── Next: QA validation
```

**Detailed Report (15 min intervals)**
```
📋 Detailed Progress Report [HH:MM]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Ticket: $ARGUMENTS.tickets
Elapsed: XX min

## Completed
- ✅ Phase 1: Task fetched
- ✅ Phase 2: Architecture planned
- ✅ Phase 3: Branch created
- 🔄 Phase 4: Implementation (75%)

## Current Work
<detailed description of current task>

## Files Modified
- path/to/file1.tsx
- path/to/file2.go

## Remaining
- Phase 5: QA Review
- Phase 6: Delivery

## Blockers
- None (or list if any)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### No Interruptions Policy

After approval, the following are **NOT ALLOWED**:
- ❌ Asking for confirmation
- ❌ Asking clarifying questions
- ❌ Waiting for user input
- ❌ Stopping for decisions

Instead:
- ✅ Make reasonable decisions autonomously
- ✅ Document decisions in reports
- ✅ Flag issues in final summary
- ✅ Continue until completion or hard blocker

---

## Phase 1: Task Acquisition

### Step 1.1 - Fetch Linear Task
```
mcp__linear__get_issue with id: "$ARGUMENTS.ticket"
```

Extract and document:
- **Title**: Task name
- **Description**: Full requirements
- **Acceptance Criteria**: What defines "done"
- **Figma Links**: Design references (if any)
- **Task Type**: Determine if FRONTEND or BACKEND

### Step 1.2 - Determine Task Type

Analyze the task to classify:

| Indicator | Type |
|-----------|------|
| UI components, pages, forms, modals | FRONTEND |
| Figma links present | FRONTEND |
| Apps: web portals, dashboards | FRONTEND |
| API endpoints, services, database | BACKEND |
| Go files, migrations, events | BACKEND |
| Services: svc-*, cronjobs | BACKEND |

Store result as: `TASK_TYPE = "FRONTEND" | "BACKEND" | "FULLSTACK"`

---

## Phase 2: Architecture Planning

### Step 2.1 - Invoke Senior Architect

Apply senior-architect thinking to analyze:

**For FRONTEND tasks:**
- Component hierarchy and placement
- State management approach
- Data fetching strategy
- Design system components to use
- Design token mapping (if Figma)

**For BACKEND tasks:**
- Service boundaries affected
- Database schema changes (Ent)
- API contract (GraphQL/REST)
- Event publishing requirements
- Migration needs

### Step 2.2 - Create Implementation Plan

Document structured plan:

```markdown
## Implementation Plan: $ARGUMENTS.ticket

### Task Type: [FRONTEND/BACKEND/FULLSTACK]

### Files to Create/Modify
1. `path/to/file` - Purpose

### Dependencies
- External: [packages needed]
- Internal: [other services/components]

### Implementation Steps
1. Step 1
2. Step 2
...

### Risks & Mitigations
- Risk: ... | Mitigation: ...
```

### Step 2.3 - Wait for Approval

Present plan to user. **DO NOT proceed without explicit approval.**

---

## Phase 3: Branch Setup

### Step 3.1 - Get Branch Name from Linear

**ALWAYS use the branch name suggested by Linear.** When viewing a task in Linear,
it shows the suggested branch name (e.g., `igor/ap-567-refactor-line-items-section`).

To get the branch name:
```
mcp__linear__get_issue with id: "$ARGUMENTS.ticket"
```

Look for the `branchName` field in the response.

### Step 3.2 - Create Feature Branch

```bash
# Determine correct repository
# Frontend: <frontend-app>
# Backend: appropriate svc-* directory

git checkout main
git pull origin main

# ALWAYS use Linear's suggested branch name
git checkout -b <linear-suggested-branch-name>
```

Example: If Linear suggests `igor/ap-567-refactor-line-items-section`, use exactly that:
```bash
git checkout -b igor/ap-567-refactor-line-items-section
```

---

## Phase 4: Implementation

### FRONTEND Implementation

Apply senior-frontend approach:

**Step 4.1 - Figma Analysis** (if link exists)

Extract `fileKey` and `nodeId` from URL, then:
```
mcp__figma__get_design_context with fileKey and nodeId
mcp__figma__get_variable_defs with fileKey and nodeId
mcp__figma__get_screenshot with fileKey and nodeId
```

Document:
- Component structure from Figma
- Color tokens → CSS variables mapping
- Typography → Text component props
- Spacing → Tailwind classes
- Icons needed

**Step 4.2 - Write Code**

Rules:
- ALWAYS use `the project's design system` components
- Use `cn()` for class merging
- Use `tv()` for variants
- Follow existing patterns in codebase
- TypeScript strict (no `any`)

**Step 4.3 - Verify TypeScript**
```bash
cd apps/<app> && npx tsc --noEmit
```

---

### BACKEND Implementation

Apply senior-backend approach:

**Step 4.1 - Schema Changes** (if needed)

Update Ent schema in `internal/generated/ent/schema/`

**Step 4.2 - Implement Command/Query**

Follow patterns:
- Commands in `internal/features/commands/`
- Queries in `internal/features/queries/`
- Proper error handling with context
- Event publishing for cross-service communication

**Step 4.3 - Write Tests**
```go
// Unit tests for business logic
// Integration tests for full flows
```

**Step 4.4 - Verify Build**
```bash
go build ./...
go test ./...
```

---

## Phase 5: QA Review

Apply senior-qa approach for validation:

### FRONTEND QA Checklist

- [ ] **Figma Compliance** (if applicable)
  - Compare implementation with screenshot
  - Verify all design tokens used correctly
  - Check responsive behavior
  - Validate spacing and typography

- [ ] **Code Quality**
  - Design system components used (not raw HTML)
  - TypeScript passes without errors
  - No console.log statements
  - Proper error handling

- [ ] **Functionality**
  - All acceptance criteria met
  - Edge cases handled
  - Loading states present
  - Error states handled

### BACKEND QA Checklist

- [ ] **Code Quality**
  - Error handling with context
  - Logging present
  - No hardcoded values
  - Follows project patterns

- [ ] **Testing**
  - Unit tests written
  - Tests pass: `go test ./...`
  - Critical paths covered

- [ ] **Database** (if applicable)
  - Migrations work: up and down
  - Schema changes correct
  - Indexes added where needed

- [ ] **Events** (if applicable)
  - Events published correctly
  - Event handlers working

### REGRESSION CHECK (MANDATORY)

**Before ANY delivery, verify NO REGRESSIONS were introduced:**

- [ ] **Existing Tests**
  - All existing tests still pass
  - No tests were deleted or skipped
  - Test coverage not reduced

- [ ] **Existing Functionality**
  - Features unrelated to the task still work
  - No unintended side effects
  - API contracts preserved

- [ ] **Code Review for Regressions**
  - Review diff for accidental deletions
  - Check imports weren't broken
  - Verify no shared utilities were modified incorrectly

- [ ] **Authorization Check**
  - If ANY existing functionality needs to change: **STOP and ask user**
  - Never remove or modify existing behavior without explicit authorization
  - Document any intentional changes to existing code

**If regression is detected:**
1. **STOP immediately**
2. **DO NOT commit**
3. **Revert the regression**
4. **Ask user for authorization** if the change is intentional

### QA Verdict

If ANY check fails (including regression check):
- Document the failure
- Fix the issue
- Re-run verification
- Loop until all pass

---

## Phase 6: Delivery

### Step 6.1 - Commit Changes

```bash
git add <specific-files>
git commit -m "feat($ARGUMENTS.ticket): <concise description>"
```

Rules:
- NO Co-Authored-By
- Stage only relevant files
- Clear, descriptive message

### Step 6.2 - Push Branch

```bash
git push -u origin feature/$(echo "$ARGUMENTS.ticket" | tr '[:upper:]' '[:lower:]')
```

### Step 6.3 - Generate Summary

Create delivery summary in English with bullets:

```markdown
## Delivery Summary: $ARGUMENTS.ticket

### Branch
`feature/<ticket-lowercase>`

### Title
<Task title from Linear>

### Changes Made
- Implemented <feature/fix description>
- Added <component/endpoint/test>
- Updated <file/config>

### Files Modified
- `path/to/file.tsx` - Description
- `path/to/file.go` - Description

### How to Test
1. Step 1
2. Step 2
3. Step 3

### Technical Notes
- <Any relevant implementation details>
- <Dependencies or considerations>
```

### Step 6.4 - Update Linear (Optional)

```
mcp__linear__create_comment with:
- Branch name
- Summary in English
- Test instructions
```

---

## Completion

When ALL phases complete successfully, output:

```
MAVERICK_COMPLETE

## Task: $ARGUMENTS.ticket

### Branch
feature/<ticket-lowercase>

### Summary
<bullet points of what was done>

### Status
✅ Implementation complete
✅ QA validation passed
✅ Changes committed and pushed

Ready for PR creation.
```

---

## Iteration Tracking

Use TaskList to track progress:

| Checkpoint | Status |
|------------|--------|
| Linear task fetched | ⬜ |
| Task type determined | ⬜ |
| Architecture planned | ⬜ |
| Plan approved | ⬜ |
| Branch created | ⬜ |
| Implementation complete | ⬜ |
| QA validation passed | ⬜ |
| Committed | ⬜ |
| Pushed | ⬜ |
| Summary generated | ⬜ |

---

## Error Handling

### If blocked on implementation
- Document the blocker
- Ask user for help
- Do NOT output completion promise

### If QA fails
- Fix the issue
- Re-run validation
- Loop until pass

### If tests fail
- Debug and fix
- Do NOT proceed with failing tests

### If TypeScript fails
- Fix type errors
- Do NOT commit with errors

---

## Rules

1. **ONE approval only** - after architect planning, then fully autonomous
2. **No interruptions** - never ask questions after approval
3. **Progress reports** - follow the time-based schedule
4. **Use appropriate senior agent** for each phase
5. **QA must pass** before delivery
6. **No Co-Authored-By** in commits
7. **Summary in English** with bullet points
8. **Never skip phases** - follow the flow
9. **Make decisions autonomously** - document them, don't ask
10. **Track time** for report scheduling
11. **Always use Linear's branch name** - each task has a suggested branch name in Linear, always create branches using that exact name
12. **NO REGRESSIONS** - never remove or modify existing functionality without explicit user authorization
13. **Preserve all tests** - never delete, skip, or disable existing tests
14. **Regression check mandatory** - QA phase must verify no regressions before delivery
15. **When in doubt, STOP** - if a change might cause regression, stop and ask for authorization
