---
description: "Autonomous end-to-end development workflow with senior agents"
arguments:
  - name: tickets
    description: "Linear ticket IDs (AP-552 or AP-552,AP-553) OR --local with task descriptions"
    required: true
user_invocable: true
---

# Maverick - Autonomous Development

Execute complete development cycle for: **$ARGUMENTS.tickets**

## Mode Detection

Parse input to determine mode:
- **Linear ticket**: `AP-552` → Fetch from Linear, standard execution
- **Multiple Linear tickets**: `AP-552,AP-553,AP-554` → Parallel execution with worktrees
- **Local mode**: `--local "task description"` → No Linear, work from provided description
- **Local multi**: `--local "task1" "task2" "task3"` → Multiple local tasks, sequential execution

---

## Local Mode (`--local`)

When `--local` is detected, skip Linear entirely. The user provides the task description directly.

### Single Local Task

```
/maverick --local "Add dark mode toggle to the settings page"
```

### Multiple Local Tasks

```
/maverick --local "Fix login form validation" "Add loading spinner to submit button" "Refactor auth hook"
```

### Local Workflow

```
DESCRIBE → ARCHITECT → BRANCH → IMPLEMENT → QA → DELIVER
```

### Phase 1: Parse Task Description

Extract from the provided text:
- **Goal**: What needs to be done
- **Scope**: Which parts of the codebase are affected
- **Task type**: FRONTEND, BACKEND, or FULLSTACK (infer from description)
- **Acceptance criteria**: Derive from the goal

If the description is vague, use the codebase to infer context:
- Search for related files and patterns
- Identify the area of the codebase affected
- Build a clear task definition from the description + code context

### Phase 2: Architecture (senior-architect)

Same as Linear mode - analyze and plan:
- Files to create/modify
- Components/services needed
- Implementation steps

**Wait for user approval before proceeding.**

### Phase 3: Create Branch

```bash
# Generate branch name from task description
# e.g., "Add dark mode toggle" → feature/add-dark-mode-toggle
git checkout main
git pull origin main
git checkout -b feature/<slugified-description>
```

### Phase 4-6: Implement → QA → Deliver

Same as Linear mode (see below).

### Local Completion

```
MAVERICK_COMPLETE

Task: <task description summary>
Branch: feature/<branch-name>
Summary: <bullets>
Status: Ready for PR
```

### Multiple Local Tasks

For multiple `--local` tasks, execute **sequentially** (each on its own branch):

```
Task 1: feature/fix-login-validation → DONE
Task 2: feature/add-loading-spinner → DONE
Task 3: feature/refactor-auth-hook → DONE
```

Output at the end:

```
MAVERICK_LOCAL_COMPLETE

Completed Tasks:
- "Fix login form validation" → feature/fix-login-validation
- "Add loading spinner" → feature/add-loading-spinner
- "Refactor auth hook" → feature/refactor-auth-hook

All branches pushed and ready for PR.
```

---

## Linear Mode (Single Ticket)

For one ticket, execute standard workflow:

```
/ralph-loop:ralph-loop "/maverick $ARGUMENTS.tickets" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"
```

---

## Linear Mode (Multiple Tickets - Parallel with Worktrees)

When multiple tickets are provided (comma-separated), execute in parallel using git worktrees.

### Step 1: Parse Tickets

```bash
# Split tickets into array
TICKETS=($ARGUMENTS.tickets)  # e.g., AP-552,AP-553,AP-554
```

### Step 2: Determine Repository

Identify the target repository based on task types:
- **Frontend tasks**: `<project-root>/<frontend-app>`
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
cd $WORKTREE_BASE/AP-552
/ralph-loop:ralph-loop "/maverick-single AP-552" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"

# Terminal 2 - Ticket 2
cd $WORKTREE_BASE/AP-553
/ralph-loop:ralph-loop "/maverick-single AP-553" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"

# Terminal 3 - Ticket 3
cd $WORKTREE_BASE/AP-554
/ralph-loop:ralph-loop "/maverick-single AP-554" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"
```

### Step 5: Cleanup Worktrees (after completion)

```bash
for TICKET in ${TICKETS[@]}; do
  git -C $REPO_BASE worktree remove $WORKTREE_BASE/$TICKET
done
```

---

## Shared Workflow (Linear & Local)

```
[TASK SOURCE] → ARCHITECT → BRANCH → IMPLEMENT → QA → DELIVER
```

### Phase 1: Fetch Task

**Linear mode:**
```
mcp__linear__get_issue with id: "<ticket>"
```

**Local mode:**
Parse the provided task description directly.

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

**Linear mode:**
```bash
# Use branch name from Linear if available
git checkout -b <linear-suggested-branch-name>
```

**Local mode:**
```bash
# Generate from task description
git checkout -b feature/<slugified-description>
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
git commit -m "feat(<ticket-or-scope>): <description>"
git push -u origin <branch-name>
```

Generate summary in English with bullets:
- Branch name
- What was done
- How to test

Create PR using `gh pr create`.

### Phase 7: Schedule PR Review Check

After the PR is created, schedule a check after 10 minutes to verify if there are any review comments:

1. Use `/loop 10m` or `CronCreate` to schedule a single check
2. The check should:
   - Fetch PR review comments using `gh api repos/{owner}/{repo}/pulls/{pr_number}/comments`
   - Fetch PR reviews using `gh pr view {pr_number} --json reviews`
   - If there are unresolved comments or CHANGES_REQUESTED, notify the user
   - If no comments, report that the PR is clear
3. Use the `/review-resolver` skill if actionable comments are found, presenting them to the user for resolution

---

## Completion

### Single Ticket (Linear)
```
MAVERICK_COMPLETE

Branch: feature/<ticket>
Summary: <bullets>
Status: Ready for PR
```

### Multiple Tickets (Linear)
```
MAVERICK_PARALLEL_COMPLETE

Completed Tickets:
- AP-552: feature/ap-552
- AP-553: feature/ap-553
- AP-554: feature/ap-554

All branches pushed and ready for PR.
```

### Local Task
```
MAVERICK_COMPLETE

Task: <description>
Branch: feature/<branch-name>
Summary: <bullets>
Status: Ready for PR
```

---

## Quick Reference

| Command | Description |
|---------|-------------|
| `/maverick AP-552` | Single Linear ticket |
| `/maverick AP-552,AP-553,AP-554` | Multiple Linear tickets, parallel worktrees |
| `/maverick --local "description"` | Single local task, no Linear |
| `/maverick --local "task1" "task2"` | Multiple local tasks, sequential |

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
[HH:MM] Phase: X | Working on: <current task>
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
8. Each ticket/task = independent branch
9. Make decisions autonomously - document, don't ask
10. **Local mode**: no Linear calls, derive everything from description + codebase
