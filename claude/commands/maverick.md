---
description: "Autonomous end-to-end development workflow (delegates to the maverick skill)"
arguments:
  - name: tickets
    description: "Ticket(s) (TICKET-123 or TICKET-123,TICKET-124) OR --local \"task description\""
    required: true
user_invocable: true
---

# Maverick

Invoke the `maverick` **skill** with: **$ARGUMENTS.tickets**

Use the Skill tool now: `skill: maverick`, `args: "$ARGUMENTS.tickets"`.

The full workflow definition lives in `.claude/skills/maverick/SKILL.md` — that file is the single
source of truth (this command is just the `/maverick` entry point). If the skill is not installed,
install it from the maverick repo: `./setup.sh project <repo-path> [--pack go]`.
