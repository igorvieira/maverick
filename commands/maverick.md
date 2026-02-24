---
description: "Autonomous end-to-end development workflow with senior agents"
arguments:
  - name: tickets
    description: "Linear ticket IDs - single (AP-552) or multiple (AP-552,AP-553,AP-554)"
    required: true
user_invocable: true
---

# Maverick - Autonomous Development

Execute complete development cycle for: **$ARGUMENTS.tickets**

## Mode Detection

Parse tickets to determine mode:
- **Single ticket**: `AP-552` → Standard execution
- **Multiple tickets**: `AP-552,AP-553,AP-554` → Parallel execution with worktrees

---

## Single Ticket Mode

For one ticket, execute standard workflow:

```
/ralph-loop:ralph-loop "/maverick $ARGUMENTS.tickets" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"
```

---

## Multiple Tickets Mode (Parallel with Worktrees)

When multiple tickets are provided (comma-separated), execute in parallel using git worktrees.

### Step 1: Parse Tickets

```bash
# Split tickets into array
TICKETS=($ARGUMENTS.tickets)  # e.g., AP-552,AP-553,AP-554
```

### Step 2: Determine Repository

Identify the target repository based on task types:
- **Frontend tasks**: `<project-root>/<repo>`
- **Backend tasks**: `<project-root>/svc-<service-name>`

### Step 3: Setup Worktrees

For each ticket, create a worktree:

```bash
# Base repository
REPO_BASE="<project-root>/<repo>"
WORKTREE_BASE="<project-root>/worktrees"

# Create worktree directory
mkdir -p $WORKTREE_BASE

# For each ticket
for TICKET in ${TICKETS[@]}; do
  BRANCH="feature/$(echo $TICKET | tr '[:upper:]' '[:lower:]')"
  WORKTREE_PATH="$WORKTREE_BASE/$TICKET"

  # Create worktree with new branch
  git -C $REPO_BASE worktree add $WORKTREE_PATH -b $BRANCH

  echo "Created worktree: $WORKTREE_PATH on branch $BRANCH"
done
```

### Step 4: Launch Parallel Ralph Loops

Each ticket runs in its own worktree with independent Ralph loop:

```bash
# Terminal 1 - Ticket 1
cd <project-root>/worktrees/AP-552
/ralph-loop:ralph-loop "/maverick-single AP-552" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"

# Terminal 2 - Ticket 2
cd <project-root>/worktrees/AP-553
/ralph-loop:ralph-loop "/maverick-single AP-553" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"

# Terminal 3 - Ticket 3
cd <project-root>/worktrees/AP-554
/ralph-loop:ralph-loop "/maverick-single AP-554" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"
```

### Step 5: Cleanup Worktrees (after completion)

```bash
for TICKET in ${TICKETS[@]}; do
  git -C $REPO_BASE worktree remove <project-root>/worktrees/$TICKET
done
```

---

## Single Ticket Workflow

```
LINEAR → ARCHITECT → BRANCH → IMPLEMENT → QA → DELIVER
```

### Phase 1: Fetch Task

```
mcp__linear__get_issue with id: "<ticket>"
```

Identify:
- Requirements and acceptance criteria
- Figma links (if any)
- Task type: **FRONTEND** or **BACKEND**

### Phase 2: Architecture (senior-architect)

Analyze and plan:
- Files to create/modify
- Components/services needed
- Implementation steps

**Wait for user approval before proceeding.**

### Phase 3: Create Branch (or use worktree branch)

```bash
git checkout -b feature/$(echo "<ticket>" | tr '[:upper:]' '[:lower:]')
```

### Phase 4: Implement

**FRONTEND** (senior-frontend):
- Get Figma context if available
- Use the project's design system components
- Verify: `npx tsc --noEmit`

**BACKEND** (senior-backend):
- Implement commands/queries
- Write tests
- Verify: `go build ./... && go test ./...`

### Phase 5: QA Review (senior-qa)

**FRONTEND checks:**
- [ ] Figma tokens matched
- [ ] Design system components used
- [ ] TypeScript passes
- [ ] All criteria met

**BACKEND checks:**
- [ ] Tests pass
- [ ] Migrations work
- [ ] Error handling correct
- [ ] Events published

### Phase 6: Deliver

```bash
git add <files>
git commit -m "feat(<ticket>): <description>"
git push -u origin feature/<ticket>
```

Generate summary in English with bullets:
- Branch name
- What was done
- How to test

---

## Completion

### Single Ticket
```
MAVERICK_COMPLETE

Branch: feature/<ticket>
Summary: <bullets>
Status: Ready for PR
```

### Multiple Tickets
```
MAVERICK_PARALLEL_COMPLETE

Completed Tickets:
- AP-552: feature/ap-552 ✅
- AP-553: feature/ap-553 ✅
- AP-554: feature/ap-554 ✅

All branches pushed and ready for PR.
```

---

## Quick Reference

| Command | Description |
|---------|-------------|
| `/maverick AP-552` | Single ticket, standard flow |
| `/maverick AP-552,AP-553,AP-554` | Multiple tickets, parallel worktrees |

---

## Approval & Progress Reports

### Single Approval
- **ONE checkpoint only**: After senior-architect presents the plan
- After approval: **FULLY AUTONOMOUS** - no more questions

### Progress Report Schedule
| Elapsed | Frequency |
|---------|-----------|
| 0-5 min | Silent |
| 5-20 min | Every 2 min |
| 20-30 min | Every 5 min |
| 30+ min | Every 15 min |

### Report Format
```
📍 [HH:MM] Phase: X | Working on: <current task>
```

---

## Rules

1. **ONE approval only** - after architect, then autonomous
2. **No interruptions** - never ask questions after approval
3. **Progress reports** - follow the time schedule
4. QA must pass before commit
5. No Co-Authored-By
6. Summary in English
7. Worktrees are isolated - no conflicts
8. Each ticket = independent branch
9. Make decisions autonomously - document, don't ask
