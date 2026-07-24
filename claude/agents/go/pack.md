# Maverick Language Pack: Go

Backend agent suite for Go services following DDD + Clean Architecture. The agents are
**project-agnostic**: everything repo-specific (layout, migrations, error-wrapping library,
test tooling) comes from the project adapter at `.claude/maverick/project.md`.

## Selection signals

Use this pack when:
- the project adapter declares `packs: go`, or
- the repository has a `go.mod` and no adapter says otherwise.

## Installation

`setup.sh project <repo> --pack go` copies the nine agent files flat into the project's
`.claude/agents/` and this manifest to `.claude/maverick/packs/go.md`.

## Workflow slots

| Slot | Agent | When | Verdict scale (blocking in **bold**) |
|---|---|---|---|
| Planner | `go-task-scope-planner` | every backend task, before planning is presented | structured plan + Change Classification + clarifying questions |
| Red team | `go-adversarial-architect` | Change Classification marks Domain or Eventing = Yes; new aggregates, transaction boundaries, cross-service consistency, new eventing strategy | holds / holds with risks / **critical flaws** |
| Implementer | `go-implementer` | after plan approval; also for non-trivial fix-loop fixes | n/a |
| Reviewer (always) | `go-idiom-reviewer` | every diff | Idiomatic / Mostly Idiomatic / **Non-Idiomatic** |
| Reviewer | `go-domain-model-reviewer` | Domain = Yes (aggregates, VOs, invariants, state machines) | Strong Model / Needs Refinement / **Invalid Model** |
| Reviewer | `go-application-flow-reviewer` | Application Flow = Yes (use cases, handlers, orchestration) | Clean Flow / Needs Refinement / **Broken Flow** |
| Reviewer | `go-adapter-reviewer` | Adapters = Yes (repos, gRPC/HTTP handlers, consumers, clients) | Clean Adapter / Needs Refinement / **Broken Adapter** |
| Reviewer | `go-eventing-reviewer` | Eventing = Yes (event structs, publish/consume) | Safe Events / Needs Refinement / **Dangerous Events** |
| Reviewer | `go-arch-reviewer` | new boundaries, transactions, concurrency (significant changes) | Acceptable / Needs Work / **Reject** (+ Blocker/High/Medium/Low findings) |

## The Change Classification

`go-task-scope-planner` returns a Change Classification — Domain / Application Flow / Eventing /
Adapters / Pure Implementation (Yes/No each). **Save it**: it selects the reviewer panel above.
Diff inspection is the fallback selector when a classification is missing.

## Red-team gate

- **"Architecture holds under scrutiny"** → proceed
- **"holds with identified risks"** → fold P0/P1 attacks into the plan's Risks & Mitigations with
  explicit mitigations; answer the "Questions the Blue Team Must Answer" in the plan
- **"critical flaws"** → revise the design and re-run before presenting for approval; never present
  a critically-flawed plan

Skip the red team for Pure Implementation / Adapters-only changes.

## Fix loop

- Any **blocking verdict** or `go-arch-reviewer` **Blocker/High** finding → MUST fix
- Middle verdicts ("Needs Refinement" / "Mostly Idiomatic" / "Needs Work") → apply the listed
  MUST-FIX items; remaining suggestions are judgment calls — apply or record as Accepted Trade-offs
- Top verdicts → done

Apply fixes (spawn `go-implementer` again for non-trivial ones), re-run verify commands, then
re-run **only the reviewers that blocked**. After 3 rounds without convergence, stop and surface
the disagreement to the user instead of looping.

## Verify commands (defaults — the project adapter overrides)

```bash
go build ./...
go test ./...
```

Never send code that fails build or tests to the reviewer panel.
