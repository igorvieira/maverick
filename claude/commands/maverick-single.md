---
description: "Single ticket execution for Maverick parallel worktrees"
arguments:
  - name: ticket
    description: "Single ticket ID (e.g., TICKET-123)"
    required: true
user_invocable: true
---

# Maverick Single - Worktree Execution

Execute development cycle for **$ARGUMENTS.ticket** in current worktree.

> This command is used by `/maverick` when running parallel worktrees.
> It assumes you're already in the correct worktree directory.
> Project facts come from the adapter (`.claude/maverick/project.md`); language expertise comes
> from the installed pack (manifest: `.claude/maverick/packs/<pack>.json`).

## Workflow

```
ADAPTER → TASK → PLAN (manifest planner) → IMPLEMENT (manifest implementer → routed reviewers) → QA → COMMIT → PUSH → OPEN PR → REVIEW WAIT
```

### Phase 0: Read the Adapter

Read `.claude/maverick/project.md` (commands, layout, conventions, branch/ticket rules). If it's
missing, discover from repo signals and note assumptions in the plan.

### Phase 1: Fetch Task

```
mcp__linear__get_issue with id: "$ARGUMENTS.ticket"
```

Extract:
- Title, description, acceptance criteria
- Figma links (if any)
- Extensible capability classification, saved as `classification.json`

### Phase 2: Planning

Use the selected manifests' planner slots and the core skill's Language Packs protocol.
Read `.claude/maverick/project.json` when present for explicit packs and command
overrides; otherwise retain the Markdown adapter. Selected packs override detection.
JSON manifests are authoritative; legacy Markdown manifest files are pointers only.

Classify capabilities with evidence. Invoke a red-team slot only when its manifest gate
matches. Resolve blockers before approval and fold warning mitigations into the plan.
No matching pack → generic senior-architect planning. Invalid explicit packs → error.

**Present plan (with any Open Questions) and wait for approval.**

### Phase 3: Implement

Current worktree already has the feature branch. Use manifest implementer slots with
the approved plan, capabilities, and adapter. Retain senior-frontend/Figma support for
UI work; without a matching pack, follow the existing generic implementation fallback.

Resolve commands per pack: adapter override → pack default → absent. Run only configured
commands after checking their scripts/tools. Never invent a default when one is missing.

Select reviewers in manifest order: `always` or any capability intersection, once each.
Unknown capabilities select no additional reviewers. Keep reviewer IDs scoped by pack.
The existing Markdown panel may run in parallel; Rust does not dispatch execution.
Consume structured results, mapping legacy labels only through manifest `legacy_verdicts`.
Fail or blocker/high findings always block. Missing required results remain unresolved.
Apply `review_policy`: resolve warnings, rerun blocked reviewers when configured, and
stop at the declared maximum fix rounds. Record accepted trade-offs and final results.

### Phase 4: QA Review (senior-qa)

**UI work, when applicable:**
- [ ] Figma compliance (tokens, spacing, typography)
- [ ] Design system components used
- [ ] Typecheck passes
- [ ] All criteria met

**General checks:**
- [ ] Tests pass
- [ ] Migrations work (if any)
- [ ] Error handling correct
- [ ] Events published (if any)
- [ ] Pack reviewer panel converged: zero blocking verdicts, trade-offs recorded

**REGRESSION CHECK (mandatory):** existing tests pass, nothing deleted/skipped, no unintended
behavior changes. Regression detected → STOP, revert, ask.

### Phase 5: Deliver

```bash
git add <specific-files>
git commit -m "feat($ARGUMENTS.ticket): <description>"
git push -u origin <branch-name>
```

### Phase 6: Open PR

Open the PR and **capture the PR number**:

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

Record `$PR_URL` and `$PR_NUMBER` — both are required for Phase 7.

### Phase 7: Review Wait + Resolver Hand-off

This phase is **mandatory** after `gh pr create`. It gives human + automated reviewers a
10-minute window, then either escalates to `/review-resolver` or declares the PR clean.

1. **Schedule the check 10 minutes out** via `ScheduleWakeup` (yield, don't block):
   ```
   ScheduleWakeup({
     delaySeconds: 600,
     reason: "10-min review window for PR <PR_NUMBER>",
     prompt: "/maverick-single:resume-review <PR_NUMBER>"
   })
   ```
   If `ScheduleWakeup` is unavailable, fall back to `/loop 10m` once with the same payload.

2. **When the timer fires, gather PR feedback** from all three sources:
   ```bash
   gh pr view $PR_NUMBER --json state,reviews,reviewDecision,statusCheckRollup
   gh api repos/:owner/:repo/pulls/$PR_NUMBER/comments --paginate
   gh api repos/:owner/:repo/issues/$PR_NUMBER/comments --paginate
   ```

3. **Classify and act**:
   - `reviewDecision == "CHANGES_REQUESTED"` **OR** any unresolved thread **OR** any actionable
     comment → invoke `/review-resolver $PR_NUMBER`.
   - Failing required checks → invoke `/review-resolver $PR_NUMBER` so the failures surface
     alongside comments.
   - Otherwise → output `PR <PR_NUMBER> is clear after the 10-minute review window.` and end.

4. **Do not re-schedule Phase 7 from inside itself.** Hand off once; the next iteration is driven
   by the user or by `/review-resolver`.

### Phase 8: Summary

Generate in English:

```markdown
## $ARGUMENTS.ticket Complete

### Branch
<branch-name>

### Changes
- <bullet 1>
- <bullet 2>
- <bullet 3>

### How to Test
1. Step 1
2. Step 2
```

## Completion

After Phase 7's review window has been honored (either resolver hand-off or clean), output exactly:

```
MAVERICK_COMPLETE

Ticket: $ARGUMENTS.ticket
Branch: <branch-name>
PR: <PR_URL>
Review: <"clear after 10-min window" | "handed off to /review-resolver">
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
10. Project facts come from the adapter - never hardcode them
