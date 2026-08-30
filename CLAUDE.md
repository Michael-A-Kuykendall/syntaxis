# Project Instructions for AI Agents

This file provides instructions and context for AI coding agents working on this project.

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:6cd5cc61 -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md for details and anti-patterns.

## Agent Context Profiles

The managed Beads block is task-tracking guidance, not permission to override repository, user, or orchestrator instructions.

- **Conservative (default)**: Use `bd` for task tracking. Do not run git commits, git pushes, or Dolt remote sync unless explicitly asked. At handoff, report changed files, validation, and suggested next commands.
- **Minimal**: Keep tool instruction files as pointers to `bd prime`; use the same conservative git policy unless active instructions say otherwise.
- **Team-maintainer**: Only when the repository explicitly opts in, agents may close beads, run quality gates, commit, and push as part of session close. A current "do not commit" or "do not push" instruction still wins.

## Session Completion

This protocol applies when ending a Beads implementation workflow. It is subordinate to explicit user, repository, and orchestrator instructions.

1. **File issues for remaining work** - Create beads for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **Handle git/sync by active profile**:
   ```bash
   # Conservative/minimal/default: report status and proposed commands; wait for approval.
   git status

   # Team-maintainer opt-in only, unless current instructions forbid it:
   git pull --rebase
   git push
   git status
   ```
5. **Hand off** - Summarize changes, validation, issue status, and any blocked sync/commit/push step

**Critical rules:**
- Explicit user or orchestrator instructions override this Beads block.
- Do not commit or push without clear authority from the active profile or the current user request.
- If a required sync or push is blocked, stop and report the exact command and error.
<!-- END BEADS INTEGRATION -->


## Build & Test

```bash
cargo build --offline               # builds all crates offline (zero deps)
cargo test --offline                # 80 tests, no network needed
cargo run -- "The cat are sleeping."                # canonical JSON analysis
cargo run --example demo                            # live tokenization / import / retraction
cargo clippy --all-targets -- -D warnings            # lint
cargo fmt --check                                   # formatting check
make gate                                            # full pre-release gate
```

## Architecture Overview

Deterministic, offline English structural analysis engine. Zero external
dependencies. A single `Analysis` snapshot per document, byte-stable canonical
JSON, provenance (support sets) on every fact, and cascading retraction.

| crate | contains |
| --- | --- |
| `syntaxis::parser_core` | data model, spans, provenance, fact graph, canonical JSON, SHA-256 |
| `syntaxis::english_rules` | rule-pack loader, segmentation, tokenization, analysis pipeline |
| `syntaxis::conllu` | strict UD import/export and the versioned Penn↔UD projection |
| `syntaxis` binary | command-line front end |

## Conventions & Patterns

- **Determinism is the top invariant.** No `HashMap`/`HashSet` in any path
  reaching serialized output; no clock, RNG, network, or env reads in the
  analysis path.
- **`parser_core` contains no English knowledge.** Language-specific rules and
  word lists belong in `english-rules`.
- **Provenance on every fact.** Every derived fact carries a `SupportSet`
  naming its rule, rule pack, and sources.
- **Field order in `parser_core::serialize` is the contract**, not an
  implementation detail.
- See [CONTRIBUTING.md](CONTRIBUTING.md) and [DEVELOPERS.md](DEVELOPERS.md) for
  the full governance model.
