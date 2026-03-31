# Maverick
My personal Claude Code configuration with MCPs, skills, commands, and workflows.

[![Test Setup](https://github.com/igorvieira/maverick/actions/workflows/test.yml/badge.svg)](https://github.com/igorvieira/maverick/actions/workflows/test.yml)
![License](https://img.shields.io/badge/license-MIT-blue)

<img width="702" height="300" alt="image-removebg-preview (1)" src="https://github.com/user-attachments/assets/f42be722-0cdb-4961-9a70-624bb87a0a4f" />

```
                    ╔══════════════════════════════════════╗
                    ║           M A V E R I C K            ║
                    ║      Autonomous Dev Workflow          ║
                    ╚═══════════════╤══════════════════════╝
                                    │
                    ┌───────────────┴───────────────┐
                    │         TASK SOURCE            │
                    │   Linear Ticket  or  --local   │
                    └───────────────┬───────────────┘
                                    │
                    ┌───────────────▼───────────────┐
                    │      SENIOR ARCHITECT          │
                    │  Analyze scope, plan files,    │
                    │  define implementation steps    │
                    └───────────────┬───────────────┘
                                    │
                          ┌─────────▼─────────┐
                          │    USER APPROVAL   │
                          │  (only checkpoint) │
                          └─────────┬─────────┘
                                    │
              ┌─────────────────────┼─────────────────────┐
              │                     │                     │
  ┌───────────▼──────────┐ ┌───────▼────────┐ ┌─────────▼──────────┐
  │  SENIOR FRONTEND     │ │ SENIOR BACKEND │ │  SENIOR SECURITY   │
  │  React, Next.js,     │ │ Go, gRPC,      │ │  Vuln scan, deps,  │
  │  design system       │ │ microservices  │ │  secrets, OWASP    │
  └───────────┬──────────┘ └───────┬────────┘ └─────────┬──────────┘
              │                     │                     │
              └─────────────────────┼─────────────────────┘
                                    │
                    ┌───────────────▼───────────────┐
                    │          SENIOR QA             │
                    │  Tests, regression check,      │
                    │  Figma compliance, coverage     │
                    └───────────────┬───────────────┘
                                    │
                    ┌───────────────▼───────────────┐
                    │           DELIVER              │
                    │  Commit, push, create PR       │
                    └───────────────┬───────────────┘
                                    │
                              ┌─────▼─────┐
                              │   DONE    │
                              └───────────┘
```

## Structure

```
maverick/
├── mcp-servers/              # MCP server configurations
│   ├── global.json           # Global MCPs (serena, figma)
│   └── project.json          # Per-project MCPs
├── skills/                   # Custom skills
│   └── maverick/             # Autonomous development workflow
│       └── SKILL.md
├── commands/                 # Commands (slash commands)
│   ├── maverick.md           # /maverick - autonomous development
│   ├── maverick-single.md    # /maverick-single - for worktrees
│   ├── review-resolver.md    # /review-resolver - PR review handler
│   ├── senior-architect.md   # /senior-architect - architecture
│   ├── senior-frontend.md    # /senior-frontend - React/Next.js
│   ├── senior-backend.md     # /senior-backend - Go/microservices
│   ├── senior-security.md    # /senior-security - vulnerability scanning
│   └── senior-qa.md          # /senior-qa - testing and quality
├── templates/                # CLAUDE.md templates
│   └── linear-figma.md       # Linear + Figma workflow
├── setup.sh                  # Installation script
└── test_setup.sh             # Test suite for setup.sh
```

## MCP Servers

### Global
- **serena** - Intelligent code agent
- **figma** - Figma integration (design)

### Per Project
- **linear** - Linear integration (tasks)
- **github** - GitHub API (PRs, issues, repos)
- **chrome-devtools** - Chrome DevTools
- **basic-memory** - Persistent memory

## Installation

### Quick
```bash
./setup.sh
```

### Manual

1. Copy global MCPs:
```bash
# Add to your ~/.claude/settings.json under "mcpServers"
cat mcp-servers/global.json
```

2. For specific projects, add to project settings:
```bash
cat mcp-servers/project.json
```

3. Copy the CLAUDE.md template to your project:
```bash
cp templates/linear-figma.md /path/to/your/project/CLAUDE.md
```

## MCPs Used

| MCP | Type | Use |
|-----|------|-----|
| Figma | HTTP | Design to code |
| Linear | HTTP | Task management |
| GitHub | stdio | GitHub API (PRs, issues, repos) |
| Serena | stdio | Code agent |
| Chrome DevTools | stdio | Browser debugging |
| Basic Memory | stdio | Persistent memory |

## Skills & Commands

### Maverick - Autonomous Development

Maverick is a workflow that coordinates senior agents to complete tasks end-to-end. Works with Linear tickets or standalone local tasks.

```bash
# Single Linear ticket
/maverick AP-552

# Multiple Linear tickets (parallel worktrees)
/maverick AP-552,AP-553,AP-554

# Local mode - no Linear needed
/maverick --local "Add dark mode toggle to settings page"

# Multiple local tasks
/maverick --local "Fix login validation" "Add loading spinner"
```

**See the flow diagram at the top of this README.**

### Senior Agents

| Command | Description |
|---------|-------------|
| `/senior-architect` | Architectural analysis and system design |
| `/senior-frontend` | React/Next.js frontend development |
| `/senior-backend` | Go/microservices development |
| `/senior-security` | Vulnerability scanning, dependency audit, secrets detection |
| `/senior-qa` | Testing, visual QA (Figma + Chrome DevTools) |

### Review Tools

| Command | Description |
|---------|-------------|
| `/review-resolver` | Interactive PR review comment handler with regression protection |

### Installing Commands

Copy the `commands/` and `skills/` folders to your project's `.claude/`:

```bash
cp -r commands/ /path/to/project/.claude/
cp -r skills/ /path/to/project/.claude/
```

## Critical Rules

### No Regressions Policy

All agents follow a strict **NO REGRESSIONS** policy:

1. **Never remove or modify existing functionality** without explicit user authorization
2. **Always preserve existing tests** - never delete or skip tests
3. **Review changes for side effects** - check if changes affect other parts of the codebase
4. **QA phase must verify** - no regressions in existing features before delivery
5. **When in doubt, ask** - if a change might cause regression, stop and ask for authorization

### Before Any Delivery

The QA phase includes mandatory regression checks:
- Run existing test suites
- Verify unchanged functionality still works
- Check for unintended side effects
- Compare before/after behavior

## Ralph Loop (Plugin)

Maverick works best with the `ralph-loop` plugin for autonomous execution:

```bash
/ralph-loop:ralph-loop "/maverick AP-552" --max-iterations 30 --completion-promise "MAVERICK_COMPLETE"
```

The plugin is available in the official Claude Code marketplace.

## Requirements

- Claude Code CLI installed
- Node.js (for stdio MCPs)
- Python/uvx (for serena and basic-memory)

## License

MIT
