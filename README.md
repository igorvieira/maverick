# Maverick
My personal AI coding configuration with MCPs, skills, commands, agent packs, and workflows for Claude Code and Codex.

[![Test Setup](https://github.com/igorvieira/maverick/actions/workflows/test.yml/badge.svg)](https://github.com/igorvieira/maverick/actions/workflows/test.yml)
![License](https://img.shields.io/badge/license-MIT-blue)

<img width="702" height="300" alt="image-removebg-preview (1)" src="https://github.com/user-attachments/assets/f42be722-0cdb-4961-9a70-624bb87a0a4f" />

## The Three Layers

Maverick is **project-agnostic and language-agnostic** by construction. It separates:

| Layer | Lives in | Provides |
|---|---|---|
| **Core workflow** | `claude/skills/maverick/SKILL.md` | phases, single approval checkpoint, parallel worktrees, QA gates, delivery, PR + review window |
| **Language packs** | `claude/agents/<pack>/` | specialist agents for a language (planner, red team, implementer, reviewer panel) |
| **Project adapter** | `.claude/maverick/project.md` *in your repo* | YOUR repo's facts: build/test commands, layout, migrations, conventions, ticket prefix |

Project-specific knowledge never lives in this repository — it lives in each consuming repo's
adapter file. That keeps Maverick reusable across companies and codebases without leaking
anything private. A CI deny-list enforces it.

## Structure

```
maverick/
├── claude/                       # Claude Code configuration assets
│   ├── mcp-servers/              # Claude MCP server JSON snippets
│   │   ├── global.json           # Global MCPs (serena, figma)
│   │   └── project.json          # Per-project MCPs
│   ├── skills/
│   │   └── maverick/
│   │       └── SKILL.md          # Core workflow (single source of truth)
│   ├── agents/
│   │   ├── typescript/           # Minimal TypeScript pack
│   │   └── go/                   # Go language pack (DDD + Clean Architecture)
│   │       ├── pack.json         # Authoritative structured manifest
│   │       ├── pack.md           # Compatibility pointer only
│   │       ├── go-task-scope-planner.md
│   │       ├── go-adversarial-architect.md
│   │       ├── go-implementer.md
│   │       ├── go-domain-model-reviewer.md
│   │       ├── go-application-flow-reviewer.md
│   │       ├── go-adapter-reviewer.md
│   │       ├── go-eventing-reviewer.md
│   │       ├── go-idiom-reviewer.md
│   │       └── go-arch-reviewer.md
│   ├── commands/                 # Claude Code slash commands
│   │   ├── maverick.md           # Thin /maverick entry point (delegates to the skill)
│   │   ├── maverick-single.md    # Worktree unit of execution
│   │   ├── review-resolver.md
│   │   ├── senior-architect.md
│   │   ├── senior-frontend.md
│   │   ├── senior-security.md
│   │   └── senior-qa.md
│   └── templates/
│       ├── maverick-project.md   # Project adapter template (fill per repo)
│       └── linear-figma.md
├── codex/                        # Codex configuration assets
│   ├── AGENTS.md
│   ├── config/config.toml.example
│   └── skills/maverick/SKILL.md
├── setup.sh                      # Installer (global, codex, and per-project)
└── test_setup.sh                 # Test suite for setup scripts
```

## Experimental Rust Runtime

The Rust runtime is **experimental**. This first vertical slice provides a local,
typed execution kernel: run identity, an explicit state machine, validated transitions,
SQLite persistence, append-only transition history, versioned JSON artifacts, and
structured review results. The CLI records lifecycle steps explicitly; it does not
perform the work described by those steps.

**Markdown skills remain the current workflow interface.** The existing commands,
language packs, project adapters, and installers retain their existing workflow. They
do not drive the Rust lifecycle automatically; pack inspection is available separately. This foundation does not make the Maverick workflow
independent of Claude Code or Codex.

Agent execution, runtime-managed worktrees, scheduler, policy engine, and LLM
completions are **not implemented**. Model setup only discovers local CLIs and
persists a provider list. Approval authorization, required-artifact gates and automatic fix loops are also future integration work. A typed review's
`is_blocking()` returns true for `fail` status or a `blocker`/`high` finding; the CLI
validates review JSON but does not yet enforce review results as transition gates.
No agent semantics or project knowledge have moved into Rust.

### Crates and dependencies

| Crate | Responsibility |
|---|---|
| `crates/maverick-core` | Run model, transition validation, artifact/review/pack/model-setup contracts, capability routing and aggregation; independent of SQLite and CLI |
| `crates/maverick-store` | Local SQLite index, transactional state/history, artifact files and restart recovery |
| `crates/maverick-cli` | Clap JSON commands, pack inspection, Ratatui model setup, local CLI discovery |
| `crates/maverick-packs` | Declarative pack loading, agent-reference checks, file detection and explicit selection |

Dependencies are limited to Serde/JSON/TOML (contracts), UUID v4 (run IDs), thiserror
(typed library errors), rusqlite with bundled SQLite (no database service or system
SQLite requirement), tempfile (atomic file publication and test isolation), clap
(arguments), anyhow (CLI error context), and Ratatui/Crossterm (interactive setup).
`Cargo.lock` is committed. The runtime makes no network calls and does not invoke
LLM completions. Discovery may run local `grok models` / `ollama list` when those
binaries exist; it never stores secrets, only env var names.

### Build and test

Install a stable Rust toolchain with `rustfmt` and `clippy`, plus a C compiler for
bundled SQLite. Building for the first time needs access to crates.io; subsequent
CLI operations are local. The existing setup tests also require Bash and jq.

```bash
cargo build --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
bash test_setup.sh
```

Use `target/debug/maverick`, or install the binary with
`cargo install --path crates/maverick-cli --locked` to use `maverick` on your PATH.
The existing `setup.sh` does not install the experimental runtime.

### Model setup TUI

`maverick` with no subcommand opens a [Ratatui](https://ratatui.rs/) setup when stdout
is a TTY. `maverick setup` is the same. Non-TTY sessions print a hint and exit 2;
use `maverick models` for JSON. The TUI discovers Grok/Claude/Codex/Ollama on PATH,
lets you set a default, mark a **planner panel** of one or more models, and add an
OpenAI-compatible or xAI API provider. Grok is a provider implementation, not the
runtime's internal model. Config is written to `$MAVERICK_CONFIG` or
`~/.config/maverick/config.toml`; an optional `.maverick/config.toml` overlay lives in
the consuming project (already gitignored). Secrets stay in the environment.

```bash
maverick              # TUI on a TTY
maverick setup
maverick models       # resolved JSON; no run storage, no completions
```

Planning-with-every-model in the panel is the next slice. This one only records the
list.

### Basic CLI flow

All commands accept `--root <directory>` before or after the subcommand.
It names the storage directory itself and defaults to `.maverick` in the current
directory. Successful commands print JSON; errors go to stderr with a nonzero exit.

```bash
maverick start --task "Implement a sample feature"
# Copy the returned id into RUN_ID:
RUN_ID="<run-id>"
maverick transition "$RUN_ID" --to context-resolved
maverick transition "$RUN_ID" --to planned
maverick status "$RUN_ID"
maverick history "$RUN_ID"

printf '%s\n' '{"steps":["Implement the sample feature","Run tests"]}' > /tmp/plan-payload.json
maverick artifact "$RUN_ID" --kind plan --name plan.json --file /tmp/plan-payload.json

# Each invocation is a new process. These commands also work after reopening a terminal:
maverick status "<run-id>"
maverick history "<run-id>"
cat ".maverick/runs/<run-id>/plan.json"

# Alternative storage directory:
maverick start --task "Another sample task" --root /tmp/maverick-example
```

The allowed path is:

```text
created -> context-resolved -> planned -> awaiting-approval -> implementing
        -> reviewing -> testing -> delivering -> completed
             |    ^
             v    |
             fixing
```

Every nonterminal state can transition to `failed`. Both `completed` and `failed`
are terminal. Invalid transitions leave both the saved state and history unchanged.
State updates compare the previous state **and revision**, preventing stale writers
from silently overwriting changes even after a `reviewing -> fixing -> reviewing` loop.

### Storage and artifact contracts

Files are created as artifacts are supplied; `start` creates `task.json` and `reviews/`.

```text
.maverick/
  maverick.db
  runs/<run-id>/
    task.json
    context.json
    plan.json
    classification.json
    reviews/<review-name>.json
    qa.json
    delivery.json
    .versions/<artifact-name>/<revision>.json
    reviews/.versions/<review-name>/<revision>.json
```

Artifact kinds are `task`, `context`, `plan`, `classification`, `review`, `qa`, and
`delivery`. Non-review names must match the kind (for example, `plan.json`). Review
names allow ASCII letters, digits, hyphens, and underscores followed by `.json`.
Paths, separators, encoded traversal, and symlinks below the selected root are rejected.
Use a storage directory you own; concurrent hostile filesystem replacement is outside
this local store's trust boundary.

Each JSON file wraps the supplied payload:

```json
{
  "schema_version": 1,
  "run_id": "<run-id>",
  "kind": "plan",
  "name": "plan.json",
  "revision": 1,
  "payload": {"steps": ["Implement the sample feature"]}
}
```

`schema_version` versions the envelope; `revision` increments for each write to the
same artifact. Previous revisions remain inspectable in `.versions/` and readable
through the store API. Payloads are generic JSON except for reviews, whose input is:

```json
{
  "reviewer": "sample-reviewer",
  "status": "fail",
  "findings": [{
    "severity": "high",
    "rule": "sample-rule",
    "file": "src/example.rs",
    "line": 12,
    "message": "Describe the finding",
    "suggested_fix": "Describe the fix"
  }]
}
```

Review statuses are `pass`, `warn`, `fail`; severities are `blocker`, `high`,
`medium`, `low`, `info`. File, line, and suggested fix may be omitted or null.

SQLite indexes runs and artifact revisions; JSON payloads live in files. State and
transition history commit together in a transaction, and database triggers reject
history updates/deletes. Artifact revision files are atomically published before the
index transaction commits. The named latest files (such as `plan.json`) are derived
views: reopening the store repairs a missing or stale view from its indexed revision.
An interrupted uncommitted write can leave an unindexed file, which is ignored;
a later write can reuse that uncommitted revision. SQLite and the filesystem do not
share a transaction. A publication error after commit explicitly reports the saved run
ID; fix the reported filesystem problem and reopen the store before retrying a write.
Opening the store currently checks all latest artifacts, so startup cost grows with
the local index. Back up the entire storage directory while no process is writing.
This experimental schema has no migration path to future versions yet.

## Installation

### Into a project (recommended)

```bash
./setup.sh project /path/to/your/repo --pack go
```

This installs into the repo's `.claude/`:
- the maverick skill + commands
- the go pack agents (flat in `.claude/agents/`) and its manifest (`.claude/maverick/packs/go.json`; `go.md` is a compatibility pointer)
- the **project adapter template** at `.claude/maverick/project.md` (never overwrites an existing one)

Then **fill the adapter** with your repo's facts — commands, layout, migration workflow,
conventions, ticket prefix. That file is the only place project-specific knowledge should live.
Never put secrets in it.

### Global Claude Code assets (MCPs)

```bash
./setup.sh claude
```

### Codex

```bash
./setup.sh codex
cp codex/AGENTS.md /path/to/your/project/AGENTS.md
```

Running `./setup.sh` without arguments opens an interactive selector; `./setup.sh all` installs
Claude + Codex global assets.

## The Maverick Workflow

Maverick coordinates agents to complete tasks end-to-end. Works with Linear tickets or standalone
local tasks.

```bash
# Single ticket
/maverick TICKET-123

# Multiple tickets (parallel worktrees)
/maverick TICKET-123,TICKET-124,TICKET-125

# Local mode - no tracker needed
/maverick --local "Add dark mode toggle to settings page"

# Multiple local tasks
/maverick --local "Fix login validation" "Add loading spinner"
```

```
0. ADAPTER    → Read .claude/maverick/project.md (or discover)
1. TASK       → Fetch ticket (Linear) or parse --local input
2. PLAN       → Manifest planner (+ optional red team)   [ONLY APPROVAL]
3. BRANCH     → Feature branch (or worktree)
4. IMPLEMENT  → Manifest implementer → capability-routed reviewers → fix loop
5. QA         → senior-qa validation + mandatory regression check
6. DELIVER    → Commit, push, summary
7. PR         → Open PR, capture number
8. REVIEW     → 10-min review window → /review-resolver hand-off or clear
```

<img width="700" height="620" alt="image" src="https://github.com/user-attachments/assets/e2bc6fe7-ae6e-4900-a06d-d76f3ea386bc" />

## Language Packs

Language packs implement a language-independent contract: detection, optional commands,
planner/implementer slots, optional red-team gates, and ordered capability-based reviewers.
**Go is the first implementation, not the runtime's internal model.** Its semantic
DDD, Clean Architecture, eventing, and idiom guidance stays in its prompts. TypeScript
proves the same protocol works with a different reviewer set and command set.

The authoritative manifests are [Go](claude/agents/go/pack.json) and
[TypeScript](claude/agents/typescript/pack.json). Markdown manifest files are compatibility
pointers, not editable policy copies. Both workflow entry points read JSON. The Rust
loader and router have no language-specific branches or hardcoded agent names.

| Pack | Review coverage |
|---|---|
| Go | Mandatory idiom; conditional domain, application flow, adapters, eventing and architecture |
| TypeScript | Mandatory type safety; conditional frontend/accessibility and API contracts/runtime validation |

Capabilities are extensible validated snake_case strings, not a closed enum. Reviewers
are selected once in manifest order by `always` or any capability intersection. Unknown
capabilities do not break routing. Structured fail or blocker/high findings block review;
Go's legacy prose verdict mappings live only in its JSON manifest.

```bash
./setup.sh project /path/to/your/repo --pack go --pack typescript

cargo run -p maverick-cli -- pack --manifest claude/agents/go/pack.json \
  --capability domain --capability eventing
# idiom, domain, eventing; red team required

cargo run -p maverick-cli -- pack --manifest claude/agents/typescript/pack.json \
  --capability frontend --capability api_contract
# type_safety, frontend, contracts; no red team
```

The same code path handles both packs. Pack inspection returns JSON and never runs
agents, shell commands, or lifecycle transitions. The existing Markdown workflow still
performs execution and applies each manifest's fix policy.

Project adapters can select multiple packs and override commands. Per command:
**project adapter override → pack default → absent**. Per-pack adapter overrides take
precedence over shared adapter commands. Resolve defaults separately per pack; never
invent missing commands. An optional consuming-project `project.json` companion supports
the Rust CLI; Markdown-only adapters continue working with the workflow.

See [the pack specification](docs/language-packs.md) for the schema, agent checks,
selection/detection rules, aggregation, adapter format, compatibility, and limitations.

### Senior Commands

| Command | Description |
|---------|-------------|
| `/senior-architect` | Architectural analysis and system design |
| `/senior-frontend` | Frontend development (design-system + Figma aware) |
| `/senior-security` | Vulnerability scanning, dependency audit, secrets detection |
| `/senior-qa` | Testing, visual QA (Figma + Chrome DevTools) |

### Review Tools

| Command | Description |
|---------|-------------|
| `/review-resolver` | Interactive PR review comment handler with regression protection |

## MCP Servers

### Global
- **serena** - Intelligent code agent
- **figma** - Figma integration (design)

### Per Project
- **linear** - Linear integration (tasks)
- **github** - GitHub API (PRs, issues, repos)
- **chrome-devtools** - Chrome DevTools
- **basic-memory** - Persistent memory

| MCP | Type | Use |
|-----|------|-----|
| Figma | HTTP | Design to code |
| Linear | HTTP | Task management |
| GitHub | stdio | GitHub API (PRs, issues, repos) |
| Serena | stdio | Code agent |
| Chrome DevTools | stdio | Browser debugging |
| Basic Memory | stdio | Persistent memory |

## Critical Rules

### No Regressions Policy

All agents follow a strict **NO REGRESSIONS** policy:

1. **Never remove or modify existing functionality** without explicit user authorization
2. **Always preserve existing tests** - never delete or skip tests
3. **Review changes for side effects** - check if changes affect other parts of the codebase
4. **QA phase must verify** - no regressions in existing features before delivery
5. **When in doubt, ask** - if a change might cause regression, stop and ask for authorization

### Org-Agnostic Guarantee

This repository must contain **zero** organization-specific data: no internal service names,
internal libraries, staging hosts, or real ticket IDs. All of that belongs in each consuming
repo's `.claude/maverick/project.md`. CI runs a deny-list grep on every push/PR to enforce it —
the pattern itself lives in the `ORG_DATA_DENYLIST` repo secret, so not even the deny-list names
anything internal.

## Ralph Loop (Plugin)

Maverick works best with the `ralph-loop` plugin for autonomous execution:

```bash
/ralph-loop:ralph-loop "/maverick TICKET-123" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"
```

The plugin is available in the official Claude Code marketplace.

## Requirements

- Claude Code CLI installed for Claude setup
- Codex CLI installed for Codex setup
- Node.js (for stdio MCPs)
- Python/uvx (for serena and basic-memory)

## License

MIT
