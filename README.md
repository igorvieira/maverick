# Maverick
My personal AI coding configuration with MCPs, skills, commands, and workflows for Claude Code and Codex.

[![Test Setup](https://github.com/igorvieira/maverick/actions/workflows/test.yml/badge.svg)](https://github.com/igorvieira/maverick/actions/workflows/test.yml)
![License](https://img.shields.io/badge/license-MIT-blue)

<img width="702" height="300" alt="image-removebg-preview (1)" src="https://github.com/user-attachments/assets/f42be722-0cdb-4961-9a70-624bb87a0a4f" />

## Structure

```
maverick/
├── mcp-servers/              # MCP server configurations
│   ├── global.json           # Global MCPs (serena, figma)
│   └── project.json          # Per-project MCPs
├── skills/                   # Claude Code custom skills
│   └── maverick/             # Autonomous development workflow
│       └── SKILL.md
├── commands/                 # Claude Code commands (slash commands)
│   ├── maverick.md           # /maverick - autonomous development
│   ├── maverick-single.md    # /maverick-single - for worktrees
│   ├── review-resolver.md    # /review-resolver - PR review handler
│   ├── senior-architect.md   # /senior-architect - architecture
│   ├── senior-frontend.md    # /senior-frontend - React/Next.js
│   ├── senior-backend.md     # /senior-backend - Go/microservices
│   ├── senior-security.md    # /senior-security - vulnerability scanning
│   └── senior-qa.md          # /senior-qa - testing and quality
├── codex/                    # Codex configuration assets
│   ├── AGENTS.md             # Project instruction template
│   ├── config/
│   │   └── config.toml.example
│   └── skills/
│       └── maverick/
│           └── SKILL.md
├── templates/                # Project instruction templates
│   └── linear-figma.md       # Linear + Figma workflow
├── setup.sh                  # Claude Code/Codex installation script
└── test_setup.sh             # Test suite for setup scripts
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

### Claude Code

```bash
./setup.sh claude
```

### Codex

```bash
./setup.sh codex
```

This installs the Maverick skill into `~/.codex/skills/maverick`, creates `~/.codex/config.toml` if needed, and points you to the MCP snippets in `codex/config/config.toml.example`.

To install both:

```bash
./setup.sh all
```

Running `./setup.sh` without arguments opens an interactive selector.

For each project that should use Maverick with Codex:

```bash
cp codex/AGENTS.md /path/to/your/project/AGENTS.md
```

### Manual Claude Code Setup

1. Copy global MCPs:
```bash
# Add to your ~/.claude/settings.json under "mcpServers"
cat mcp-servers/global.json
```

2. For specific projects, add to project settings:
```bash
cat mcp-servers/project.json
```

3. Copy the project instruction template:
```bash
cp templates/linear-figma.md /path/to/your/project/CLAUDE.md
```

### Manual Codex Setup

1. Copy the Codex skill:
```bash
mkdir -p ~/.codex/skills
cp -r codex/skills/maverick ~/.codex/skills/
```

2. Copy or merge the MCP snippets you need:
```bash
cat codex/config/config.toml.example
```

3. Copy the project guide:
```bash
cp codex/AGENTS.md /path/to/your/project/AGENTS.md
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

## Flow

<img width="700" height="620" alt="image" src="https://github.com/user-attachments/assets/e2bc6fe7-ae6e-4900-a06d-d76f3ea386bc" />



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

For Claude Code, copy the `commands/` and `skills/` folders to your project's `.claude/`:

```bash
cp -r commands/ /path/to/project/.claude/
cp -r skills/ /path/to/project/.claude/
```

For Codex, install the skill globally and add `AGENTS.md` to each project:

```bash
./setup.sh codex
cp codex/AGENTS.md /path/to/project/AGENTS.md
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

- Claude Code CLI installed for Claude setup
- Codex CLI installed for Codex setup
- Node.js (for stdio MCPs)
- Python/uvx (for serena and basic-memory)

## License

MIT
