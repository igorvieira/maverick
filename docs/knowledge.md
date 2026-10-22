# Personal knowledge (optional)

Maverick can feed your own engineering playbooks to the planner, implementer and reviewers.
The harness ships only the convention. The data is yours: it lives in `.knowledge/` at the
root of your Maverick checkout, which is git-ignored, so nothing you write there is ever
committed or published with the harness.

## Layout

```text
maverick/
  .knowledge/            # git-ignored, yours
    index.json           # what exists and when it applies
    <anything>.md        # playbooks, checklists, notes; organize folders however you like
```

To keep it somewhere else (a notes vault, a private repo), set `MAVERICK_KNOWLEDGE=/path/to/dir`
before running `./setup.sh project`.

## index.json

The only required file. Each entry points to a Markdown file and lists the capabilities
it applies to, using the same extensible snake_case values the planner writes to
`classification.json`:

```json
{
  "schema_version": 1,
  "entries": [
    {
      "id": "transactional-email-outbox",
      "title": "Transactional email that is sent exactly once",
      "file": "reliability/transactional-email-outbox.md",
      "capabilities": ["email_delivery", "background_jobs"],
      "summary": "Outbox in the same transaction, deduped worker, capped retries, delivery webhooks."
    }
  ]
}
```

Write the Markdown however you prefer. Short sections work best for agents: the problem,
the approach, and a checklist a reviewer can verify against a diff.

## How a project uses it

`./setup.sh project <repo>` links `<repo>/.claude/maverick/knowledge` to your knowledge
directory when it exists. It is a symlink, not a copy: no knowledge is written into the
project. Add `.claude/maverick/knowledge` to that repo's ignore rules if `.claude/` is
committed there.

During a run, after classification, the workflow reads `index.json`, loads the entries
whose capabilities intersect the task's, and passes them to the implementer and to each
selected reviewer as additional checks. No knowledge directory means this step is skipped.
