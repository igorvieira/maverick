# Development Workflow with Linear + Figma

This project uses Linear and Figma MCPs for a structured development flow.

## Workflow

### 1. Task Reading (Linear)

When receiving a Linear task:
- Use `mcp__linear__get_issue` to get complete details
- Identify Figma links in the description
- Extract requirements and acceptance criteria
- Present summary to user for confirmation

### 2. Design Analysis (Figma)

If there's a Figma link:
- Extract `fileKey` and `nodeId` from the URL
- Use `mcp__figma__get_design_context` to get code and structure
- Use `mcp__figma__get_variable_defs` for design tokens (colors, spacing, typography)
- Use `mcp__figma__get_screenshot` for design visualization
- Document found tokens for use in implementation

### 3. Planning

ALWAYS enter planning mode before implementing:
- Use `EnterPlanMode` for non-trivial tasks
- List all files to be created/modified
- Detail necessary components
- Map Figma tokens to CSS variables/code
- Wait for user approval before proceeding

### 4. Implementation

During implementation:
- Create tasks with `TaskCreate` for tracking
- Follow Figma tokens and specifications
- Keep code clean without over-engineering
- Update task status as progress is made

### 5. Review

After implementation:
- Review generated code
- Verify adherence to Figma design
- Ensure all requirements were met
- Present result to user

### 6. Final Documentation (Linear)

When finishing the task:
- Use `mcp__linear__create_comment` to add to the issue:
  - PR link (if any)
  - Step-by-step testing instructions
  - Screenshots or relevant observations
- Use `mcp__linear__update_issue` to update status if needed

## Useful Commands

### Get my tasks
```
mcp__linear__list_issues with assignee: "me"
```

### Get specific task
```
mcp__linear__get_issue with id: "ISSUE-123"
```

### Get Figma context
```
URL: https://figma.com/design/{fileKey}/{fileName}?node-id={nodeId}
mcp__figma__get_design_context with extracted fileKey and nodeId
```

## Test Comment Format (Linear)

When finishing, add comment in this format:

```markdown
## Implementation Complete

### Modified Files
- `path/to/file.tsx` - Change description

### How to Test
1. Step 1
2. Step 2
3. Step 3

### Notes
- Relevant notes about the implementation
```

## Git and Commits

- **NEVER** include "Co-Authored-By" in commits - only the user's author
- Commits should be simple, without extra signatures
- Commit format: just the message, no co-authorship footer

## Rules

- ALWAYS wait for user approval before implementing
- ALWAYS read the complete task before starting
- ALWAYS check Figma when available
- NEVER skip the planning phase for complex tasks
- NEVER update Linear status without user confirmation
