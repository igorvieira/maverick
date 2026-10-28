# Language pack contract (schema version 1)

A language pack implements a generic protocol. Six packs ship today: Go, TypeScript
(Node.js backend and frontend), Rust, Python, Rails, and Elixir. None defines the
runtime's internal model. The protocol shares slots, capabilities, command data, and
structured results. Backend review content is shared through language-neutral
principles (below); each pack adds only its language's idioms and libraries.

## Boundaries

| Layer | Owns |
|---|---|
| Runtime core | Lifecycle, state machine, artifacts, validated capabilities, routing, structured review aggregation |
| Pack loader | JSON loading, schema validation, agent file checks, file-based detection |
| Language pack | Detection signals, default commands, prompt references, reviewer definitions, capability mappings, semantic instructions |
| Shared principles | Language-neutral backend rules applied by pack reviewers (`claude/principles/`) |
| Project adapter | Selected packs, repository commands, conventions, layout, accepted variants, project-specific overrides |

The Rust runtime has no conditional branches for particular languages or agent names.
The core has no filesystem dependencies. `maverick-packs` supplies loading and detection;
`maverick pack` exposes inspection/routing through the CLI. Existing lifecycle commands
remain separate. No providers, scheduler, parallel execution, shell execution, or
agent execution are implemented in this slice.

## Authoritative files and compatibility

Source layout:

```text
claude/agents/<pack-id>/
  pack.json
  pack.md          # compatibility pointer only, no routing or policy definitions
  <agent-id>.md
```

Installed layout:

```text
.claude/
  agents/<agent-id>.md
  maverick/packs/<pack-id>.json
  maverick/packs/<pack-id>.md
  maverick/project.md
  maverick/project.json         # optional, created by the consuming project
```

`pack.json` is the only source of manifest truth. Both Markdown entry points consume
it. The retained `pack.md` only points readers to JSON, so older links and installer
file-presence checks remain valid. It is not another editable manifest. Re-run the
project installer to update both consumers and manifests together. The installer does
not require Rust and never overwrites either project adapter.

Prompts remain authoritative for semantic guidance. Existing Go reviewers keep their
DDD, Clean Architecture, eventing, transaction, concurrency, and idiom instructions.
Their added output protocol supplies structured results alongside the existing report.
The Go planner emits extensible capabilities and consults the manifest for routing.

## Shared principles

`claude/principles/` holds one file per backend review area: `domain.md`,
`application-flow.md`, `adapters.md`, `eventing.md`, `architecture.md`, and
`red-team.md`. The installer copies them to `.claude/maverick/principles/` whenever at
least one pack is installed. Pack prompts reference them by path and add a short list of
language-specific checks, so a rule changes in one place for every language.

The Go pack predates the directory and keeps the same rules inline; it is the reference
the principles were extracted from.

Every principles file inherits `README.md`'s override rules:

1. Variants listed in the adapter's **Accepted Variants** section are reported as
   `low`/`info`, never blocking.
2. Reviews grade the diff; pre-existing violations are listed once as existing debt.
3. Data-loss rules (lost events, non-idempotent consumers of at-least-once delivery,
   unbounded retries, missing timeouts, unowned background work) stay blocking in new
   code unless the adapter lists them as accepted risks with an owner.
4. Existing project abstractions always win over new ones.

## Pack catalog

| Pack | Detection | Default commands | Always | Conditional |
|---|---|---|---|---|
| `go` | `go.mod` | gofmt, go build, go test | idiom | domain, application_flow, adapter, eventing, architecture |
| `typescript` | `tsconfig.json` or `package.json` | npm build/test/lint/typecheck | type_safety | frontend, contracts, domain, application_flow, adapter, eventing, architecture |
| `rust` | `Cargo.toml` | cargo fmt/build/test/clippy | idiom | domain, application_flow, adapter, eventing, architecture |
| `python` | `pyproject.toml`, `requirements.txt` or `setup.py` | ruff format/check, pytest | idiom | domain, application_flow, adapter, eventing, architecture |
| `rails` | `Gemfile` and `config/application.rb` | bin/rails test, rubocop | idiom | domain, application_flow, adapter, eventing, architecture |
| `elixir` | `mix.exs` | mix format/compile/test | idiom | domain, application_flow, adapter, eventing, architecture |

Default commands are hints: planners confirm them against the repository, and the
adapter overrides them (for example `bundle exec rspec` or `uv run pytest`).

All packs share the capability vocabulary `domain`, `aggregate_design`,
`application_flow`, `adapters`, `api_contract`, `database`, `eventing`,
`eventing_strategy`, `architecture`, `transaction_boundaries`,
`cross_service_consistency`, and `concurrency`, plus pack-specific values (for
example `frontend`, `runtime_validation`, `unsafe_code`, `otp_processes`). The red team
runs for `domain`, `eventing`, `aggregate_design`, `transaction_boundaries`,
`cross_service_consistency`, and `eventing_strategy` in every pack.

To add a language, create `claude/agents/<id>/` with a `pack.json` that uses the same
slots and capabilities, a planner, an implementer, an always-on idiom reviewer, and
reviewers that reference the shared principles. Then add the ID to the pack lists in
`crates/maverick-packs/tests/contract.rs` and `crates/maverick-cli/tests/packs.rs`.

## Manifest fields

All objects reject unknown fields. JSON is used because the workspace already supports
Serde JSON; this change adds no external Rust dependency. `PackManifest::from_json`
validates the complete contract. Public Rust values can be assembled directly, so
routing, aggregation, and loader entry points validate them again before use.

| Field | Contract |
|---|---|
| `schema_version` | Required integer, currently exactly `1` |
| `id` | Unique pack ID in the loaded collection |
| `display_name` | Nonblank display name |
| `detection.files` | Exact relative file paths, all required; defaults to `[]` |
| `detection.any_files` | Exact relative paths, at least one required when nonempty; defaults to `[]` |
| `commands` | Object with optional `format`, `build`, `test`, `lint`, `typecheck` strings |
| `agents.planner`, `agents.implementer` | Required prompt IDs |
| `agents.red_team` | Optional prompt ID; must be paired with a `red_team` gate |
| `reviewers` | Ordered array, with unique reviewer IDs and agent references |
| `red_team` | Optional gate with `always`, `capabilities`, `blocking`, and optional `legacy_verdicts` |
| `review_policy` | Positive `max_fix_rounds`, `rerun_blocked_only`, `resolve_warnings` |

Pack, reviewer, agent, and project-command IDs use lowercase ASCII segments separated
by `-` or `_`, start with a letter, and are at most 64 characters. Empty segments,
paths, whitespace, and traversal are rejected. Agent IDs resolve to `<id>.md` inside
the supplied agent directory. Missing, empty, non-UTF-8, and escaping agent files fail
loading. The loader checks file references; it does not parse provider-specific prompt
frontmatter or execute prompt content.

Detection combines both rule groups with AND. Empty rules never auto-select a pack.
Paths cannot be absolute, contain traversal, or use globs; matching is exact and relative
to the project root. Files resolving outside that root are not detection matches.
TypeScript's `package.json` signal is deliberately broad and is only a hint: confirm the
language and configured scripts during planning or select packs explicitly in the adapter.

## Capabilities and deterministic routing

`Capability` is a serializable value object with equality and hashing. It validates
`[a-z][a-z0-9]*(_[a-z0-9]+)*`, up to 64 characters, on construction and deserialization.
There is no closed capability enum. Packs and projects may introduce new values without
changing Rust. Classification payloads contain a set:

```json
{"capabilities": ["frontend", "api_contract"]}
```

Each reviewer declares:

```json
{
  "id": "example",
  "agent": "example-reviewer",
  "always": false,
  "capabilities": ["example_capability"],
  "blocking": {
    "statuses": ["fail"],
    "severities": ["blocker", "high"]
  }
}
```

The router selects `always` reviewers and any reviewer whose capabilities intersect
the classification. It filters the manifest array once, preserving declaration order
and returning each reviewer only once. Unknown capabilities are valid and harmless.
Conditional reviewers and gates must declare at least one capability.

`./setup.sh project` without `--pack` applies the same detection rules (using `jq`) and
installs every matching pack.

Multi-pack selection follows explicit adapter order. Without an adapter, detection
preserves the caller's manifest order. Duplicate pack IDs are rejected. Reviewer
identity is scoped by pack; two packs can both have a reviewer named `contracts`.
They are routed and aggregated separately, never deduplicated across packs.

## Reviews, gates, and preserved Go policy

Results follow the existing `ReviewResult` contract: `reviewer` is the manifest's
reviewer ID (within one pack), `status` is `pass`, `warn`, or `fail`, and `findings`
contain typed severity, rule, message, and optional source location/fix.

`BlockingPolicy` must include fail, blocker, and high. Packs may add stricter status
or severity blockers but cannot weaken that floor. Aggregation returns blocked IDs,
missing IDs, and warnings in manifest order. Missing selected reviews block completion;
duplicate or unselected results are errors. It does not infer a verdict from prose.

Go's `legacy_verdicts` maps its existing exact labels to typed statuses for the Markdown
consumer and prompts. No such phrases appear in runtime logic. TypeScript emits typed
results directly and needs no label mapping. MUST-FIX findings carry high or blocker
severity even when the overall status is warn. Other warnings require documented
resolution or accepted trade-offs when `resolve_warnings` is enabled.

Go retains six reviewers and the red-team gate for domain/eventing and significant
design signals. The planner's Go-specific instructions map new aggregates, transaction
boundaries, cross-service consistency, and eventing strategies to declared capabilities.
Architecture review is selected by its own significant-boundary/concurrency signals.
The manifest preserves the three-round fix limit and rerunning only blocked reviewers.

Rust computes red-team selection and review aggregation. The Markdown consumer still
executes approval, red-team review, warning resolution, and fix-loop policies. This
slice does not automatically gate lifecycle transitions or run review/fix loops.

## Project adapter and command precedence

An optional consuming-project `.claude/maverick/project.json` provides structured data:

```json
{
  "schema_version": 1,
  "packs": ["go", "typescript"],
  "commands": {},
  "pack_commands": {
    "typescript": {"test": "npm run test:unit"}
  },
  "project_commands": {"check_docs": "npm run docs:check"},
  "conventions": ["Follow existing repository conventions"],
  "layout": {"source": "src/"}
}
```

These are illustrative consumer settings, not Maverick repository facts. The companion
is not installed automatically. When present, it owns the machine-readable fields;
do not duplicate them in `project.md`. The Markdown adapter continues to hold context
not in JSON. Existing Markdown-only adapters remain supported by the workflow; Rust
only loads explicitly supplied JSON adapters and never guesses how to parse prose.

Per command, precedence is project adapter override → pack default → absent. A per-pack
adapter override wins over a shared adapter command. Commands resolve separately for
each selected pack, avoiding accidental mixing of defaults. Omitted/null overrides
inherit; empty or NUL-containing strings are invalid. Disabling an inherited command
is not modeled in version 1. Additional named `project_commands` remain separate data.

Explicit `packs: []` disables auto-selection. Unknown or duplicate selected packs and
overrides for unselected packs are rejected. Without an adapter, detection can select
multiple packs. No language or tool name is hardcoded in the resolver.

**Command execution policy in this slice is no execution.** CLI output explicitly says
`"execution": "disabled"`. Loading a command does not authorize it. The existing Markdown
workflow must check configured scripts/tools and honor its approval rules. Missing
commands are reported, never replaced with invented commands. Automatic command policy
and execution belong to a later slice.

## Build and inspect

```bash
cargo build --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
bash test_setup.sh

target/debug/maverick pack --manifest claude/agents/go/pack.json \
  --capability domain --capability eventing
# Reviewer IDs: idiom, domain, eventing; red_team_required: true

target/debug/maverick pack --manifest claude/agents/typescript/pack.json \
  --capability frontend --capability api_contract
# Reviewer IDs: type_safety, frontend, contracts; red_team_required: false
```

Both commands execute the same loader, validator, and routing function. `--classification`
accepts a JSON file with a capabilities array and merges it with repeated `--capability`
values. `--project` enables detection; without it all supplied manifests are inspected.
An explicit `--adapter` overrides detection. Repeat `--manifest` for multiple packs.
For installed files, pass `--agents-dir .claude/agents`. Pack inspection never creates
run storage; the global `--root` applies only to lifecycle commands.

Tests run the same contract against both packs and against renamed/synthetic manifests
with novel capabilities. They also verify installer compatibility, installed prompt
resolution, error handling, override precedence, and absence of command execution.

## Current limits

Non-Go packs keep their prompts short and depend on the shared principles being
installed; a hand-copied agent without `.claude/maverick/principles/` is incomplete.
Detection uses exact paths,
not globs or package-content analysis. Prompt frontmatter validation, provider dispatch,
command execution, automatic lifecycle gates, scheduler, parallel execution, and future
schema migrations remain outside this slice. Filesystem loading assumes a trusted local
pack/project directory, not hostile concurrent filesystem mutation.
