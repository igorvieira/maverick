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
│   │   └── go/                   # Go language pack (DDD + Clean Architecture)
│   │       ├── pack.md           # Manifest: slot → agent → blocking verdict
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

## Installation

### Into a project (recommended)

```bash
./setup.sh project /path/to/your/repo --pack go
```

This installs into the repo's `.claude/`:
- the maverick skill + commands
- the go pack agents (flat in `.claude/agents/`) and its manifest (`.claude/maverick/packs/go.md`)
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
2. PLAN       → FE: senior-architect | BE: pack planner (+ red team)   [ONLY APPROVAL]
3. BRANCH     → Feature branch (or worktree)
4. IMPLEMENT  → FE: senior-frontend | BE: pack implementer → reviewer panel → fix loop
5. QA         → senior-qa validation + mandatory regression check
6. DELIVER    → Commit, push, summary
7. PR         → Open PR, capture number
8. REVIEW     → 10-min review window → /review-resolver hand-off or clear
```

<img width="700" height="620" alt="image" src="https://github.com/user-attachments/assets/e2bc6fe7-ae6e-4900-a06d-d76f3ea386bc" />

## Language Packs

A pack is a suite of specialist agents for one language, with a manifest mapping workflow slots
(planner / red team / implementer / reviewers) to agents and their **blocking verdicts**. The core
skill selects the pack from the adapter (or repo signals like `go.mod`) and degrades gracefully to
generic roles when no pack is installed.

### Go pack

Backend suite for Go services following DDD + Clean Architecture:

| Agent | Role | Blocking verdict |
|---|---|---|
| `go-task-scope-planner` | Scope analysis, ambiguity detection, Change Classification | — |
| `go-adversarial-architect` | Red Team for significant designs | critical flaws |
| `go-implementer` | Writes code across all layers | — |
| `go-domain-model-reviewer` | Aggregates, VOs, invariants, state machines | Invalid Model |
| `go-application-flow-reviewer` | Use cases, orchestration, transactions | Broken Flow |
| `go-adapter-reviewer` | Repos, HTTP/gRPC handlers, consumers, clients | Broken Adapter |
| `go-eventing-reviewer` | Event contracts, publishing, idempotent consumption | Dangerous Events |
| `go-idiom-reviewer` | Idiomatic Go, naming, error handling | Non-Idiomatic |
| `go-arch-reviewer` | Layer boundaries, transactions, concurrency | Reject / Blocker |

The reviewer panel is selected by the planner's Change Classification and runs **in parallel**;
blocking verdicts feed a fix loop (max 3 rounds, then surface to the user).

Future packs (e.g., `typescript/`, `python/`) plug into the same slots without touching the core
skill.

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
