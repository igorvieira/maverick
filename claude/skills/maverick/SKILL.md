---
name: maverick
description: >
  Autonomous end-to-end development workflow. Orchestrates senior agents (architect, frontend, qa)
  and declarative language packs (planner, implementer, and specialist reviewers) to complete tasks from planning to PR and review hand-off. Project facts
  come from a per-repo adapter file, so the workflow itself stays project-agnostic. Supports Linear
  tickets, multiple parallel tickets with git worktrees, or local mode (--local) for tasks without
  Linear. Only ONE approval checkpoint (after planning), then fully autonomous with progress reports.
user_invocable: true
arguments:
  - name: tickets
    description: "Ticket(s) (TICKET-123 or TICKET-123,TICKET-124) OR --local \"task description\" for local tasks"
    required: true
---

# Maverick Workflow

Autonomous development workflow for **$ARGUMENTS.tickets** using coordinated agents. The workflow
is project-agnostic and language-agnostic: project facts come from the **project adapter** and
language expertise comes from **agent packs**.

## The Three Layers

| Layer | Where | What it provides |
|---|---|---|
| Core workflow | this skill | phases, single approval checkpoint, worktrees, QA gates, delivery, PR + review window |
| Language packs | `.claude/agents/` + manifest in `.claude/maverick/packs/<pack>.json` | planner / red-team / implementer / specialist reviewers for a language |
| Project adapter | `.claude/maverick/project.md` | THIS repo's facts: build/test commands, layout, migrations, conventions, ticket prefix |

**Never hardcode a project fact that belongs in the adapter.** If the adapter is missing, discover
what you can (Phase 0) and confirm the rest at the approval checkpoint — don't guess silently.

---

## Phase 0: Project Adapter

Before anything else, read `.claude/maverick/project.md`. It defines:

- **packs** to use, and the **frontend** stack (if any)
- **commands**: build, test, lint, typecheck
- **layout & conventions**: where code lives, API contract style
- **migrations & codegen** workflow
- **error handling / logging / testing** conventions
- **tickets & branches**: ticket prefix, branch source (tracker-suggested vs slug), default branch
- **PR & review** conventions, and **docs pointers** for deeper context

Read an optional `.claude/maverick/project.json` companion for explicit pack selection
and command overrides. If it exists, it is authoritative for machine-readable fields;
`project.md` remains the home for repository context and conventions not in JSON.
Without the companion, read the existing Markdown adapter normally.

Selection order is explicit adapter packs, then each installed JSON manifest's detection
rules, then generic fallback. Detection requires all `files` and, when nonempty, at
least one `any_files`; empty rules do not auto-select. Multiple matches are allowed.
Detection is a hint to confirm during planning, not proof of language or available tools.
An explicitly selected missing/invalid pack is an error, not a silent fallback.

Resolve each command independently: project adapter override → pack default → absent.
Within the adapter, per-pack overrides take precedence over shared commands. For multiple
packs, retain separate command sets; do not merge conflicting defaults. Check configured
scripts/tools before running them. Report absent commands; never invent a fallback.

Record what you resolved and what you assumed. **Assumed facts are surfaced at the approval
checkpoint (Phase 2.3)** — that's the one chance to correct them.

---

## Execution Modes

### Single Ticket
```bash
/maverick TICKET-123
# or with Ralph Loop:
/ralph-loop:ralph-loop "/maverick TICKET-123" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"
```

### Multiple Tickets (Parallel Worktrees)
```bash
/maverick TICKET-123,TICKET-124,TICKET-125
```

When multiple tickets are provided, Maverick will:
1. Create a git worktree for each ticket
2. Launch parallel Ralph loops (each running `/maverick-single`)
3. Each ticket executes independently — no conflicts between tickets

### Local Mode (No Tracker)
```bash
# Single task
/maverick --local "Add dark mode toggle to settings page"

# Multiple tasks (sequential)
/maverick --local "Fix login validation" "Add loading spinner" "Refactor auth hook"
```

When `--local` is used:
- No Linear API calls are made
- Task description is parsed directly from the input
- Branch names are generated from the description (e.g., `feature/add-dark-mode-toggle`)
- Everything else (plan, implement, QA, deliver, PR) works the same

---

## Workflow Overview

```
┌────────────────────────────────────────────────────────────────────┐
│                        MAVERICK WORKFLOW                           │
├────────────────────────────────────────────────────────────────────┤
│  0. ADAPTER    → Read .claude/maverick/project.md (or discover)    │
│  1. TASK       → Fetch ticket (Linear) or parse --local input      │
│  2. PLAN       → Manifest planner (+ optional red team) [APPROVAL] │
│  3. BRANCH     → Create feature branch (or worktree)               │
│  4. IMPLEMENT  → Manifest implementer → reviewers → fix loop       │
│  5. QA         → senior-qa validation + regression check           │
│  6. DELIVER    → Commit, push, summary                             │
│  7. PR         → Open PR, capture number                           │
│  8. REVIEW     → 10-min review window → /review-resolver or clear  │
└────────────────────────────────────────────────────────────────────┘
```

---

## Language Packs

Packs are independent implementations of the same protocol. The authoritative manifest
is `.claude/maverick/packs/<pack>.json`; prompts remain in `.claude/agents/`. A legacy
`<pack>.md` is only a compatibility pointer. Do not derive routing from prompt names or
keep a second table of language-specific rules in this workflow.

Read and validate schema version 1, unique IDs, required planner/implementer slots,
optional red-team slot and matching gate, reviewer definitions, commands, and agent files.
Read the selected agents' prompts for language semantics. Invalid manifests must surface
clear errors before implementation. When the experimental CLI is available, use:

```bash
maverick pack --manifest .claude/maverick/packs/<pack>.json \
  --agents-dir .claude/agents --classification ./classification.json
```

The CLI returns a deterministic selection and resolved command data; it does not execute
agents or commands. Without it, the Markdown workflow reads the same JSON protocol:

1. Classify changes as a set of extensible snake_case capabilities.
2. Select each reviewer with `always: true` or at least one matching capability.
3. Preserve manifest order; select each reviewer once. Unknown capabilities are harmless.
4. Select the red-team slot using the manifest gate's `always`/capabilities rules.
5. Consume structured ReviewResult status/findings. Transitional prose labels are mapped
   only by the manifest's `legacy_verdicts`; never guess their meaning from phrases.
6. Block on fail or blocker/high findings, plus any stricter declared blocking policy.
   Missing or malformed required results are unresolved reviews, never a pass.

For multiple packs, reviewer identity is `(pack.id, reviewer.id)`. Do not conflate equally
named reviewers from different packs. No pack is a base class for another.

### Generic fallback (no matching pack)

- **PLAN**: apply senior-architect thinking and read adapter documentation.
- **IMPLEMENT**: follow adapter conventions and existing patterns. For a `frontend`
  capability without a pack, retain the senior-frontend playbook and Figma support.
- **VERIFY**: run only configured adapter commands.
- **REVIEW**: self-review and state that no specialist reviewer panel ran.

Degrade gracefully when no pack matches; explicit invalid selections must be corrected.

---

## Parallel Worktrees Mode

When executing multiple tickets (`TICKET-123,TICKET-124,TICKET-125`):

### Setup Worktrees

```bash
REPO_BASE="<project-root>/<repo>"
WORKTREE_BASE="<project-root>/worktrees"

mkdir -p $WORKTREE_BASE

for TICKET in TICKET-123 TICKET-124 TICKET-125; do
  BRANCH="feature/$(echo $TICKET | tr '[:upper:]' '[:lower:]')"
  git -C $REPO_BASE worktree add $WORKTREE_BASE/$TICKET -b $BRANCH
done
```

### Launch Parallel Loops

Each worktree runs independently:

```bash
# Terminal N
cd $WORKTREE_BASE/TICKET-123
/ralph-loop:ralph-loop "/maverick-single TICKET-123" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"
```

### Cleanup After Completion

```bash
for TICKET in TICKET-123 TICKET-124 TICKET-125; do
  git -C $REPO_BASE worktree remove $WORKTREE_BASE/$TICKET
done
rmdir $WORKTREE_BASE  # if empty
```

### Parallel Benefits

- **No conflicts**: each ticket has its own directory
- **Independent branches**: no merge issues during development
- **True parallelism**: multiple Claude instances working simultaneously

---

## Approval & Progress Reports

### Single Approval Checkpoint

**ONLY ONE interruption allowed**: after the implementation plan is complete (Phase 2.3).

```
┌──────────────────────────────────────────────────────────────────────┐
│  TASK → PLAN → [APPROVAL] → IMPLEMENT → QA → DELIVER → PR → REVIEW  │
│                    ↑                                                 │
│              ONLY CHECKPOINT                                         │
│           (no more interruptions)                                    │
└──────────────────────────────────────────────────────────────────────┘
```

After user approval, execution is **FULLY AUTONOMOUS** — no more questions or confirmations.

### Progress Report Schedule

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
├── Phase: IMPLEMENT (4/8)
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
- ✅ Phase 2: Plan approved
- ✅ Phase 3: Branch created
- 🔄 Phase 4: Implementation (75%)

## Current Work
<detailed description of current task>

## Files Modified
- path/to/file1
- path/to/file2

## Remaining
- Phase 5: QA Review
- Phase 6-8: Delivery, PR, review window

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

### Step 1.1a - Tracker Mode: Fetch Task
```
mcp__linear__get_issue with id: "$ARGUMENTS.ticket"
```

### Step 1.1b - Local Mode: Parse Description

When `--local` is detected, skip the tracker entirely:
- Parse the task description provided by the user
- Search the codebase for related files and patterns to build context
- Derive acceptance criteria from the goal

Extract and document:
- **Title**: task name (from tracker or user description)
- **Description**: full requirements
- **Acceptance Criteria**: what defines "done"
- **Figma Links**: design references (if any)
- **Change Classification**: extensible capability set, with evidence for each signal

### Step 1.2 - Classify Capabilities

Save `classification.json` as a payload such as `{"capabilities": ["frontend", "api_contract"]}`.
These are examples, not a closed taxonomy. Use the selected pack planner's semantic
instructions and the actual diff. Capabilities may span multiple concerns and packs;
there is no FRONTEND/BACKEND/FULLSTACK discriminator. Reclassify if implementation
reveals additional affected surfaces, then recompute the panel from the manifest.

---

## Phase 2: Planning

### Step 2.1 - Plan with Manifest Slots

Invoke each selected pack's `agents.planner` with the task, adapter, and docs pointers.
It returns understanding, ambiguities/questions, capability classification, an
implementation plan, and acceptance criteria. Preserve open questions for approval.
If no pack matches, use the generic planning fallback. For UI changes, retain design
system, component, state/data-flow, and Figma analysis where applicable.

Evaluate each manifest's `red_team` gate against the classification. If selected, invoke
`agents.red_team` with the plan and its language-specific prompt. A blocking result
requires revision and re-review before approval. For a warning, record mitigations and
answer outstanding design questions. Do not substitute a language-specific gate here.

### Step 2.2 - Create Implementation Plan

```markdown
## Implementation Plan: $ARGUMENTS.ticket

### Selected Packs
- [manifest IDs]

### Change Classification
- Capabilities: [extensible identifiers and evidence]

### Files to Create/Modify
1. `path/to/file` - Purpose

### Dependencies
- External: [packages needed]
- Internal: [other services/components]

### Implementation Steps
1. Step 1
2. Step 2
...

### Acceptance Criteria
- [from planner + ticket]

### Risks & Mitigations
- Risk: ... | Mitigation: ...
- [Red-team findings when selected, each with its mitigation]

### Adapter Facts Assumed (only if the adapter was missing/incomplete)
- [facts discovered in Phase 0 that need confirmation]

### Open Questions (require user answer)
- [Clarifying Questions from the pack planner, if any]
```

### Step 2.3 - Wait for Approval

Present plan to user. **DO NOT proceed without explicit approval.**

If the plan has **Open Questions** or **Adapter Facts Assumed**, the approval message must surface
them; answers feed back into the plan before implementation starts. If a task can be implemented in
more than one reasonable way and the ticket doesn't specify which, ask — don't guess.

---

## Phase 3: Branch Setup

### Step 3.1 - Get Branch Name

Per the adapter's `branch_source`:

**Tracker mode (`branch_source: linear`):** ALWAYS use the branch name suggested by the tracker
(Linear: `branchName` field on the issue).

**Slug mode / local mode:** generate from the task description.
```bash
# "Add dark mode toggle" → feature/add-dark-mode-toggle
# Slugify: lowercase, replace spaces with hyphens, remove special chars, max 50 chars
```

### Step 3.2 - Create Feature Branch

```bash
git checkout <default_branch>   # from adapter, usually main
git pull origin <default_branch>
git checkout -b <branch-name>
```

---

## Phase 4: Implementation

### UI Implementation Support (when applicable)

Supplement the selected pack implementer with the senior-frontend playbook when UI
work is involved; without a pack, use that playbook as the implementation fallback:

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
- Typography → text component props
- Spacing → utility classes
- Icons needed

**Step 4.2 - Write Code**

Rules:
- ALWAYS use the project's design system components (adapter names them)
- Use the project's class-merge / variant utilities per the adapter
- Follow existing patterns in the codebase
- Follow type-safety and framework rules from the selected pack and project adapter

**Step 4.3 - Verify**
Run the resolved typecheck command when configured. If absent, report that limitation;
do not invent a language-specific command.

---

### Implementation with a Pack

**Step 4.1 - Implement with the manifest slot**

Invoke `agents.implementer` with the approved plan, capabilities, answered questions,
adapter, and documentation. Layout, migrations/codegen, error handling, testing, and
language conventions come from those inputs and the prompt.

**Step 4.2 - Verify Build**

Run the resolved build/test commands when present. Missing commands must be reported;
failing configured checks must be fixed before review.

**Step 4.3 - Specialist Review Panel**

Route from the final capability set using the Language Packs protocol above. The
existing Markdown workflow may dispatch its selected panel in parallel, preserving
manifest order in reports; this does not add execution to the Rust runtime. Give each
reviewer the diff, adapter, and its own prompt. The runtime has no language-specific
reviewer names or semantic knowledge.

**Step 4.4 - Fix Loop**

Apply each reviewer's declared blocking policy to its structured result. Fix blockers;
when `review_policy.resolve_warnings` is true, resolve required warning findings and
record remaining suggestions as accepted trade-offs. MUST-FIX findings must carry high
or blocker severity, even when the overall status is warn.

Apply fixes through the selected implementer, re-run configured verification, and
re-run only blocked reviewers when `rerun_blocked_only` is true; otherwise re-run the
selected panel. Stop and surface disagreement at `max_fix_rounds` from the manifest.
Carry final results, warning resolutions, and trade-offs into QA. These are workflow
policies; the Rust slice selects and aggregates but does not execute this loop.

### Implementation without a Pack

- Follow adapter conventions and existing patterns.
- Add relevant tests and run configured commands.
- Self-review and state that no specialist panel ran.

---

## Phase 5: QA Review

Apply senior-qa approach for validation:

### UI QA Checklist (when applicable)

- [ ] **Figma Compliance** (if applicable)
  - Compare implementation with screenshot
  - Verify all design tokens used correctly
  - Check responsive behavior
  - Validate spacing and typography

- [ ] **Code Quality**
  - Design system components used (not raw HTML)
  - Typecheck passes without errors
  - No console.log statements
  - Proper error handling

- [ ] **Functionality**
  - All acceptance criteria met
  - Edge cases handled
  - Loading states present
  - Error states handled

### General QA Checklist

- [ ] **Code Quality**
  - Error handling per adapter conventions
  - Logging present
  - No hardcoded values
  - Follows project patterns

- [ ] **Testing**
  - Unit tests written
  - Tests pass (adapter test command)
  - Critical paths covered

- [ ] **Database** (if applicable)
  - Migrations created via the project's tooling, work up and down
  - Schema changes correct
  - Indexes added where needed

- [ ] **Events** (if applicable)
  - Events published correctly
  - Event handlers working

- [ ] **Specialist Reviews** (when a pack ran)
  - Reviewer panel (Step 4.3) includes every manifest match and every mandatory reviewer
  - Zero blocking verdicts remain; fix loop (Step 4.4) converged
  - Final verdicts and Accepted Trade-offs recorded in the QA report

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
- Clear, descriptive message (adapter commit conventions apply)

### Step 6.2 - Push Branch

```bash
git push -u origin <branch-name>
```

### Step 6.3 - Generate Summary

Create delivery summary in English with bullets:

```markdown
## Delivery Summary: $ARGUMENTS.ticket

### Branch
`<branch-name>`

### Title
<Task title>

### Changes Made
- Implemented <feature/fix description>
- Added <component/endpoint/test>
- Updated <file/config>

### Files Modified
- `path/to/file` - Description

### How to Test
1. Step 1
2. Step 2
3. Step 3

### Technical Notes
- <Any relevant implementation details>
- <Dependencies or considerations>
```

### Step 6.4 - Update Tracker (Optional)

```
mcp__linear__create_comment with:
- Branch name
- Summary in English
- Test instructions
```

---

## Phase 7: Open PR

Create the PR and **capture the PR number** from the output:

```bash
PR_URL=$(gh pr create --title "feat($ARGUMENTS.ticket): <description>" --body "$(cat <<'EOF'
## Summary
- <bullet 1>
- <bullet 2>

## Test plan
- [ ] <step 1>

Ticket: $ARGUMENTS.ticket
EOF
)")
PR_NUMBER=$(echo "$PR_URL" | grep -oE '[0-9]+$')
```

Follow the adapter's PR conventions (body format, linked ticket, labels). Record both `$PR_URL`
and `$PR_NUMBER` — they are required inputs for Phase 8.

---

## Phase 8: Review Window + Resolver Hand-off

This phase is **mandatory** after every `gh pr create`. It gives human + automated reviewers a
10-minute window to leave feedback, then either escalates to `/review-resolver` or declares the
PR clean.

1. **Schedule the check 10 minutes out** using `ScheduleWakeup` (yield, don't block-wait):
   ```
   ScheduleWakeup({
     delaySeconds: 600,
     reason: "10-min review window for PR <PR_NUMBER>",
     prompt: "/maverick:resume-review <PR_NUMBER>"   # or inline the resume steps below
   })
   ```
   If `ScheduleWakeup` is unavailable in the current harness, fall back to `/loop 10m` once with
   the same payload.

2. **When the timer fires, gather PR feedback** from all three sources:
   ```bash
   gh pr view $PR_NUMBER --json state,reviews,reviewDecision,statusCheckRollup
   gh api repos/:owner/:repo/pulls/$PR_NUMBER/comments --paginate
   gh api repos/:owner/:repo/issues/$PR_NUMBER/comments --paginate
   ```
   Inline-thread comments, top-level PR comments, and formal reviews all carry actionable items.

3. **Classify and act**:
   - `reviewDecision == "CHANGES_REQUESTED"` **OR** any unresolved review thread **OR** any comment
     requesting changes → invoke `/review-resolver $PR_NUMBER`
   - Failing required status checks → invoke `/review-resolver $PR_NUMBER` as well, so failures
     surface alongside human review comments
   - No actionable items and checks green → output
     `PR <PR_NUMBER> is clear after the 10-minute review window.` and end the run

4. **Loop guard**: do not re-schedule Phase 8 from inside itself. After one resolver hand-off the
   user drives the next iteration. Maverick's job ends when the 10-minute window has been honored
   once.

---

## Completion

Emit completion **only after Phase 8's review window has been honored** (one scheduled check, then
either a `/review-resolver` hand-off or a "clear" result):

```
MAVERICK_COMPLETE

## Task: $ARGUMENTS.ticket

### Branch
<branch-name>

### Summary
<bullet points of what was done>

### Status
✅ Implementation complete
✅ QA validation passed
✅ Changes committed and pushed
✅ PR: <PR_URL>
✅ Review: <"clear after 10-min window" | "handed off to /review-resolver">
```

---

## Iteration Tracking

Use TaskList to track progress:

| Checkpoint | Status |
|------------|--------|
| Adapter read (or discovery done) | ⬜ |
| Task fetched | ⬜ |
| Capabilities classified | ⬜ |
| Plan created | ⬜ |
| Plan approved | ⬜ |
| Branch created | ⬜ |
| Implementation complete | ⬜ |
| Reviewer panel converged (if pack) | ⬜ |
| QA validation passed | ⬜ |
| Committed & pushed | ⬜ |
| PR opened | ⬜ |
| Review window honored | ⬜ |

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

### If typecheck/build fails
- Fix errors
- Do NOT commit with errors

---

## Rules

1. **ONE approval only** - after planning, then fully autonomous
2. **No interruptions** - never ask questions after approval
3. **Progress reports** - follow the time-based schedule
4. **Use the right agents** - manifest slots for packs, senior playbooks for fallback/UI support/QA
5. **QA must pass** before delivery
6. **No Co-Authored-By** in commits
7. **Summary in English** with bullet points
8. **Never skip phases** - follow the flow, including the PR review window
9. **Make decisions autonomously** - document them, don't ask
10. **Track time** for report scheduling
11. **Branch names follow the adapter** - tracker-suggested names when `branch_source: linear`
12. **NO REGRESSIONS** - never remove or modify existing functionality without explicit user authorization
13. **Preserve all tests** - never delete, skip, or disable existing tests
14. **Regression check mandatory** - QA phase must verify no regressions before delivery
15. **When in doubt, STOP** - if a change might cause regression, stop and ask for authorization
16. **Project facts live in the adapter** - never hardcode them into this skill or the packs
