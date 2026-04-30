# Maverick Project Guide

This project uses Maverick as an AI-assisted development workflow. Follow this guide when working in a repository configured with Maverick.

## Operating Rules

- Preserve existing behavior unless the user explicitly asks for a change.
- Read the local codebase before proposing implementation details.
- Prefer the repository's existing patterns, scripts, frameworks, and test conventions.
- Keep edits scoped to the requested task.
- Never delete, skip, or weaken tests to make a change pass.
- Before final delivery, run the most relevant available validation and report anything that could not be run.

## Workflow

1. Understand the task source:
   - Linear ticket: fetch the full issue details when a Linear MCP is available.
   - Local task: derive requirements from the user's description and the codebase.
   - Figma task: inspect design context and screenshots when a Figma MCP is available.
2. Plan the change:
   - Identify files to create or modify.
   - Call out data model, API, UI, migration, or rollout risks.
   - For broad or risky work, get user approval before editing.
3. Implement:
   - Make conservative, idiomatic changes.
   - Update task progress as meaningful milestones are completed.
   - Ask only when a decision cannot be inferred safely from local context.
4. Verify:
   - Run targeted tests first.
   - Run broader checks when shared behavior or public contracts changed.
   - For UI work, verify responsive states and visual regressions when tooling is available.
5. Deliver:
   - Summarize changed files and behavior.
   - Include tests/checks run.
   - Mention unresolved risks or manual follow-up.

## Linear And Figma

- Linear issue IDs usually look like `AP-552`.
- If a task includes a Figma URL, extract `fileKey` and `nodeId` and inspect the design before coding.
- Do not update Linear statuses unless the user asks for it.
- When finishing a Linear task, prepare a concise implementation note with files changed and test steps.

## Git

- Do not rewrite user changes.
- Do not use destructive Git commands unless the user explicitly asks.
- Use short, direct commit messages when commits are requested.
- Do not add AI co-author footers unless the user explicitly asks.

## Optional Maverick Skill

If the `maverick` Codex skill is installed and the user asks for the Maverick workflow, use that skill for the detailed process.
