---
description: "Resolve PR review comments interactively with AI suggestions and agent execution"
arguments:
  - name: pr
    description: "PR number or URL (e.g., 123 or https://github.com/org/repo/pull/123)"
    required: true
  - name: repo
    description: "Repository path relative to project root (e.g., frontend, svc-payments). Auto-detected from PR URL if omitted."
    required: false
user_invocable: true
---

# Review Resolver - Interactive PR Review Comment Handler

Resolve review comments for: **$ARGUMENTS.pr**

---

## Phase 1: Fetch PR Review Comments

### Step 1.1: Detect Repository

If `$ARGUMENTS.repo` is provided, use it. Otherwise, detect from current directory or PR URL.

```bash
# If repo argument provided
REPO_PATH="$(git rev-parse --show-toplevel)/$ARGUMENTS.repo"

# Otherwise, detect from current git remote
REPO_PATH=$(pwd)
```

### Step 1.2: Fetch All Review Comments

```bash
# Get PR details
gh pr view $ARGUMENTS.pr --json title,body,state,reviewDecision,url,headRefName,baseRefName

# Get all review comments (conversations)
gh api repos/{owner}/{repo}/pulls/{pr_number}/comments --paginate

# Get review threads with resolution status
gh pr view $ARGUMENTS.pr --json reviewThreads
```

### Step 1.3: Filter Unresolved Comments

Only process comments that are:
- **NOT resolved** (unresolved threads)
- **NOT outdated** (still relevant to current code)

Group comments by:
1. **File** - group all comments on the same file together
2. **Thread** - keep conversation threads together
3. **Reviewer** - identify who asked what

---

## Phase 2: Present Comments to User

For each unresolved comment thread, present:

```markdown
---
### Comment #N | File: `path/to/file.ext:line` | By: @reviewer_name

> Original reviewer comment here
> Could be multiple lines

**Code Context:**
```diff
- old code line
+ current code line
```

**Thread History** (if replies exist):
- @author: previous reply...
- @reviewer: follow-up...

---
```

Present ALL unresolved comments first so the user has the full picture before proceeding.

---

## Phase 3: Generate Suggestions for Each Comment

For each comment, analyze:
1. **The reviewer's intent** - What are they asking/suggesting?
2. **The current code** - Read the actual file and surrounding context
3. **Best practice** - What would be the ideal approach?

Then generate **2-3 options** per comment:

```markdown
### Comment #N: [Brief description of the issue]

**Reviewer asks:** [summary of the question/suggestion]

**Option A - Accept & Fix** (recommended if applicable)
- Description: [what this changes]
- Code change: [brief preview of the fix]
- Response to reviewer: "Done! [explanation of the change]"

**Option B - Explain & Keep**
- Description: [why the current approach is valid]
- Response to reviewer: "[explanation of why the current code is correct/preferred]"

**Option C - Alternative Approach**
- Description: [a different solution than what the reviewer suggested]
- Code change: [brief preview]
- Response to reviewer: "[explanation of the alternative]"

👉 Your choice for Comment #N: [A / B / C / custom]
```

**IMPORTANT**: Wait for user selection on ALL comments before proceeding. Collect all choices first.

---

## Phase 4: User Selection

Present a summary of all comments and ask the user to select options:

```markdown
## Selection Summary

| # | File | Reviewer | Issue | Your Choice |
|---|------|----------|-------|-------------|
| 1 | `file.tsx:42` | @alice | Missing error handling | ? |
| 2 | `file.tsx:87` | @bob | Naming convention | ? |
| 3 | `api.go:15` | @alice | Missing test | ? |

Please select your choices (e.g., "1A, 2B, 3A") or type "all-A" to accept all recommended fixes.
```

Wait for user input. Accept formats:
- `1A, 2B, 3C` - individual selections
- `all-A` - accept all Option A (recommended fixes)
- `all-B` - explain and keep all
- Custom per-comment instructions

---

## Phase 5: Execute Selected Changes

### Step 5.1: Categorize Changes

Separate selected options into:
- **Code changes needed** - Options that require modifying files
- **Response only** - Options that only need a reply comment

### Step 5.2: Regression Baseline (MANDATORY - run BEFORE any code change)

**CRITICAL**: Capture the current state of all tests BEFORE making any changes. This is the regression baseline.

```bash
# Frontend - capture baseline test results
cd $REPO_PATH
pnpm test:unit --reporter=verbose 2>&1 | tee /tmp/review-resolver-baseline-frontend.txt
echo "Frontend baseline exit code: $?" >> /tmp/review-resolver-baseline-frontend.txt

# Backend - capture baseline test results
go test ./... -v 2>&1 | tee /tmp/review-resolver-baseline-backend.txt
echo "Backend baseline exit code: $?" >> /tmp/review-resolver-baseline-backend.txt

# Capture list of all existing test files (to detect deletions later)
find . -name "*_test.go" -o -name "*.test.ts" -o -name "*.test.tsx" -o -name "*.spec.ts" -o -name "*.spec.tsx" | sort > /tmp/review-resolver-baseline-testfiles.txt

# Capture current TypeScript compilation status
npx tsc --noEmit 2>&1 | tee /tmp/review-resolver-baseline-tsc.txt
```

**Store the baseline counts:**
- Total tests passing
- Total tests failing (if any pre-existing failures)
- Total test files
- TypeScript errors (if any pre-existing)

**If baseline tests are already failing**: Document which tests fail BEFORE changes. These are pre-existing failures, not regressions.

### Step 5.3: Implement Code Changes

For each code change, determine the type and use the appropriate agent approach:

**Frontend changes** (senior-frontend patterns):
- Use the project's design system components
- Follow TypeScript strict mode
- Maintain existing patterns

**Backend changes** (senior-backend patterns):
- Follow Go conventions
- Implement proper error handling
- Add tests if requested

Execute all code changes:
1. Read the target file **and all files that import/depend on it**
2. Understand the context around the reviewer's comment
3. Identify all **callers and consumers** of any function/component being changed
4. Apply the selected fix
5. **After EACH individual file change**, run a quick verification:
   - Frontend: `npx tsc --noEmit` (catch type errors immediately)
   - Backend: `go build ./...` (catch compilation errors immediately)
6. If verification fails, **revert the change and try a different approach** - do NOT proceed with broken code

**Per-change regression micro-check:**
After each file modification, run the tests for that specific file/package:
```bash
# Frontend - run tests related to the changed file
pnpm vitest run --reporter=verbose <changed-file-path>

# Backend - run tests in the changed package
go test -v ./<changed-package>/...
```

If a micro-check reveals a new test failure, **STOP and fix it before proceeding** to the next change.

### Step 5.4: Full Regression Check (MANDATORY - run AFTER all code changes)

```bash
# Frontend - full test suite
pnpm test:unit --reporter=verbose 2>&1 | tee /tmp/review-resolver-after-frontend.txt

# Backend - full test suite
go test ./... -v 2>&1 | tee /tmp/review-resolver-after-backend.txt

# Verify no test files were deleted
find . -name "*_test.go" -o -name "*.test.ts" -o -name "*.test.tsx" -o -name "*.spec.ts" -o -name "*.spec.tsx" | sort > /tmp/review-resolver-after-testfiles.txt
diff /tmp/review-resolver-baseline-testfiles.txt /tmp/review-resolver-after-testfiles.txt

# TypeScript compilation
npx tsc --noEmit 2>&1 | tee /tmp/review-resolver-after-tsc.txt
```

**Compare baseline vs after:**
- **New test failures** = REGRESSION. Must fix before proceeding.
- **Deleted test files** = REGRESSION. Must restore.
- **New TypeScript errors** = REGRESSION. Must fix.
- **Fewer passing tests** = REGRESSION. Investigate and fix.

**If ANY regression is detected:**
1. Identify which change caused it
2. Fix the regression OR revert the problematic change
3. Re-run the full regression check
4. Repeat until zero regressions
5. **NEVER proceed to commit with regressions**

### Step 5.5: Create a Single Commit

Only after zero regressions confirmed:

```bash
git add <modified-files>
git commit -m "fix(review): resolve PR review comments

- <brief description of each change made>
"
```

---

## Phase 6: QA Review (senior-qa) - Regression-First Validation

Before submitting, the senior-qa agent runs a **regression-first** validation. Regressions are a HARD BLOCKER - nothing ships with regressions.

### Step 6.1: Independent Regression Verification

QA must independently verify (not trust Phase 5 results):

```bash
# Run full test suite fresh
pnpm test:unit --reporter=verbose 2>&1
go test ./... -v 2>&1

# Compare against Phase 5 baseline files
# Any NEW failure that wasn't in baseline = BLOCKER
```

### Step 6.2: Behavioral Regression Analysis

Beyond test results, QA must verify:

1. **API contracts unchanged** - If a function signature changed, verify all callers still work
2. **Component props unchanged** - If a React component's props changed, verify all usages
3. **Type exports unchanged** - If types were modified, verify downstream consumers
4. **Event/message contracts unchanged** - If events were modified, verify publishers and subscribers
5. **Database queries unchanged** - If queries were modified, verify no N+1 or missing data

```bash
# Check git diff for dangerous patterns
git diff --stat HEAD~1  # Verify only expected files changed

# Look for unintended changes
git diff HEAD~1 -- "*.test.*" "*.spec.*" "*_test.go"  # Should show ONLY additions, never deletions or modifications to existing assertions
```

### Step 6.3: Scope Creep Detection

Verify that changes are **strictly limited** to what the review comments requested:

```bash
# List all changed files
git diff --name-only HEAD~1

# For each changed file, verify it was referenced in a review comment
# If a file was changed but NOT referenced in any comment = SCOPE CREEP = BLOCKER
```

**Exception**: If fixing a review comment naturally requires updating an import or a related type definition, that's acceptable. But changing unrelated files is NOT.

### Step 6.4: Full QA Checklist

- [ ] **ZERO new test failures** (compared to baseline)
- [ ] **ZERO deleted test files**
- [ ] **ZERO deleted test cases** (existing assertions preserved)
- [ ] **ZERO new TypeScript errors**
- [ ] **ZERO new Go compilation errors**
- [ ] All modified files compile without errors
- [ ] New tests added where reviewers requested
- [ ] Code style consistent with surrounding code
- [ ] Changes are minimal and focused (no scope creep)
- [ ] No files changed that weren't referenced in review comments
- [ ] Function/component interfaces remain backward-compatible (or all callers updated)

### Step 6.5: QA Report Format

```markdown
## QA Review: PR Review Resolutions

### Regression Report (CRITICAL)
| Metric | Baseline | After | Delta | Status |
|--------|----------|-------|-------|--------|
| Tests passing (FE) | 142 | 142 | 0 | PASS |
| Tests passing (BE) | 87 | 88 | +1 | PASS |
| Tests failing (FE) | 0 | 0 | 0 | PASS |
| Tests failing (BE) | 0 | 0 | 0 | PASS |
| Test files | 34 | 34 | 0 | PASS |
| TSC errors | 0 | 0 | 0 | PASS |

### Scope Verification
| File Changed | Referenced in Comment | Justified |
|-------------|----------------------|-----------|
| `file.tsx` | Comment #1 | YES |
| `file.test.tsx` | Comment #1 (test added) | YES |

### Changes Reviewed
| File | Change | Regression Risk | Status |
|------|--------|-----------------|--------|
| `file.tsx:42` | Added error handling | Low | PASS |
| `api.go:15` | Added unit test | None | PASS |

### Test Results
- Frontend: `pnpm test:unit` → PASS/FAIL (X tests, Y suites)
- Backend: `go test ./...` → PASS/FAIL (X tests)

### Verdict
READY FOR SUBMISSION / BLOCKED - REGRESSIONS FOUND
```

### Step 6.6: Regression Found - Resolution Protocol

If QA finds ANY regression:

1. **STOP** - Do NOT proceed to Phase 7
2. **Identify** - Which specific change caused the regression
3. **Report to user** - Show exactly what broke and why
4. **Fix options**:
   - A) Fix the regression while preserving the review fix
   - B) Revert the problematic change and respond to reviewer with explanation
   - C) User decides a different approach
5. **Re-run QA** - Full regression check again after fix
6. **Loop until ZERO regressions** - No exceptions

---

## Phase 7: Submit Responses

### Step 7.1: Push Code Changes

```bash
git push
```

### Step 7.2: Post Review Responses

For each comment, post the selected response as a reply:

```bash
# Reply to each review comment thread
gh api repos/{owner}/{repo}/pulls/{pr_number}/comments/{comment_id}/replies \
  -f body="<selected response>"
```

### Step 7.3: Resolve Threads

For comments that were fully addressed with code changes:

```bash
# Mark threads as resolved (if the user chose to fix them)
gh api graphql -f query='
  mutation {
    resolveReviewThread(input: {threadId: "<thread_id>"}) {
      thread { isResolved }
    }
  }
'
```

### Step 7.4: Post Summary Comment

Add a summary comment on the PR:

```bash
gh pr comment $ARGUMENTS.pr --body "$(cat <<'EOF'
## Review Comments Resolved

### Changes Made
- <list of changes>

### Responses Posted
- <list of responses>

### QA Status
All changes verified - ready for re-review.
EOF
)"
```

---

## Phase 8: Final Report

Present to the user:

```markdown
## Review Resolution Complete

### PR: #<number> - <title>

### Comments Resolved: X/Y

| # | Comment | Action | Status |
|---|---------|--------|--------|
| 1 | Missing error handling | Fixed + Responded | Resolved |
| 2 | Naming convention | Explained | Responded |
| 3 | Missing test | Added test | Resolved |

### Code Changes
- Commit: `<hash>` - fix(review): resolve PR review comments
- Files modified: <list>

### Ready for Re-review
All comments addressed. Reviewer can now re-review.
```

---

## Interaction Rules

1. **ALWAYS show all comments first** - User needs the full picture before deciding
2. **ALWAYS wait for user selection** - Never auto-apply fixes without approval
3. **ALWAYS run QA before submitting** - No unreviewed changes go to the PR
4. **Group by file** - Makes it easier to understand related changes
5. **Preserve reviewer intent** - Responses should acknowledge the reviewer's point
6. **Minimal changes** - Only change what was requested, nothing more
7. **Professional tone** - Review responses should be respectful and constructive
8. **No Co-Authored-By** - Follow project convention for commits
9. **Explain trade-offs** - When suggesting Option B (keep), explain why clearly
10. **Mark resolved only when fixed** - Only resolve threads with actual code changes

---

## Regression Protection (ABSOLUTE PRIORITY)

These rules are **NON-NEGOTIABLE** and override all other considerations:

1. **ALWAYS capture test baseline BEFORE any code change** - No exceptions
2. **NEVER commit with failing tests that weren't failing before** - Hard blocker
3. **NEVER delete or modify existing test assertions** - Only add new ones
4. **NEVER delete test files** - If a reviewer asks to refactor tests, add new tests first, then modify
5. **NEVER proceed past Phase 5 with regressions** - Fix or revert first
6. **NEVER change files not referenced in review comments** - Scope creep = regression risk
7. **ALWAYS verify callers/consumers** when changing any function signature, component props, or type
8. **ALWAYS run per-change micro-checks** - Catch regressions at the earliest possible moment
9. **ALWAYS run full test suite comparison** - Micro-checks can miss cross-cutting regressions
10. **If in doubt, DON'T change it** - A review response explaining "this is intentional" is safer than a regression
11. **Revert > fix-forward when under pressure** - If a fix causes cascading issues, revert and try a different approach
12. **QA verdict is final** - If senior-qa says BLOCKED, the flow stops. No overriding.
