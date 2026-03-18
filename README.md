# beans

[![CI](https://github.com/kfcafe/beans/actions/workflows/ci.yml/badge.svg)](https://github.com/kfcafe/beans/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/beans-cli)](https://crates.io/crates/beans-cli)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![dependency status](https://deps.rs/repo/github/kfcafe/beans/status.svg)](https://deps.rs/repo/github/kfcafe/beans)

Task tracker for AI coding agents.

Markdown tasks with verify gates, dependency-aware scheduling, and built-in agent orchestration. Every task has a shell command that must pass to close. `bn run` dispatches work to agents automatically, tracks failures, and re-dispatches as dependencies resolve.

Plain markdown files. Any agent that can read files and run shell commands is fluent in beans.

```bash
bn create "Add CSV export" --verify "cargo test csv::export"
bn run                                   # Dispatches to an agent
bn run --loop-mode -j 8                  # Or run everything: 8 agents, continuous
```

## Contents

- [Install](#install)
- [Quick Start](#quick-start)
- [How It Works](#how-it-works)
- [Fail-First: Enforced TDD](#fail-first-enforced-tdd)
- [Failure History](#failure-history)
- [Hierarchical Tasks](#hierarchical-tasks)
- [Dependencies](#dependencies)
- [Agent Orchestration](#agent-orchestration)
- [Agent Context](#agent-context)
- [Memory System](#memory-system)
- [Commands](#commands)
- [Configuration](#configuration)

## Install

```bash
cargo install beans-cli
```

<details>
<summary>Build from source</summary>

```bash
git clone https://github.com/kfcafe/beans && cd beans
cargo build --release
cp target/release/bn ~/.local/bin/
```

</details>

## Quick Start

```bash
bn init --agent claude                               # Set up agent config
bn create "Fix CSV export" --verify "cargo test csv" # Define work with a verify gate
bn create "Add pagination" --verify "cargo test page" # Add more work
bn run                                               # Dispatch ready beans to agents
bn agents                                            # Monitor running agents
bn logs 3                                            # View agent output for bean 3
```

Manual workflow:

```bash
bn quick "Fix CSV export" --verify "cargo test csv"  # Create + claim task
bn status                                            # See what's claimed/ready/blocked
bn close 1                                           # Run verify, close if passes
```

## How It Works

Tasks are Markdown files with YAML frontmatter stored in `.beans/`:

```
.beans/
├── 1-add-csv-export.md
├── 2-add-tests.md
├── 2.1-unit-tests.md       # Child of 2 (dot notation)
└── archive/2026/01/        # Closed tasks auto-archive
```

A bean:

```yaml
---
id: "1"
title: Add CSV export
status: in_progress
verify: cargo test csv::export
attempts: 0
---

Add a `--format csv` flag to the export command.

**Files:** src/export.rs, tests/export_test.rs
```

When you run `bn close 1`:

1. Beans runs `cargo test csv::export`
2. Exit 0 → task closes, moves to archive
3. Exit non-zero → task stays open, failure appended to notes, ready for retry

## Fail-First: Enforced TDD

**On by default.** Before creating a bean, the verify command runs and must **fail**:

1. If it **passes** → bean rejected ("test doesn't test anything new")
2. If it **fails** → bean created (test is real)
3. After implementation, `bn close` runs verify → must **pass**

```bash
# REJECTED (cheating test):
bn quick "..." --verify "python -c 'assert True'"
# error: Cannot create bean: verify command already passes!

# ACCEPTED (real test):
bn quick "..." --verify "pytest test_unicode.py"
# ✓ Verify failed as expected - test is real
# Created bean 5
```

Use `--pass-ok` / `-p` to skip fail-first for refactoring or builds where verify should already pass:

```bash
bn quick "extract helper" --verify "cargo test" -p
bn quick "remove secrets" --verify "! grep 'api_key' src/" -p
```

## Failure History

When verify fails, beans appends error output to the bean's notes:

```yaml
---
id: "3"
title: Fix date parsing for ISO 8601
verify: pytest test_dates.py
attempts: 2
---

## Attempt 1 — 2024-01-15T14:32:00Z
Exit code: 1
```
FAILED test_dates.py::test_timezone_offset
  AssertionError: Expected '2024-01-15T14:32+05:30' but got '2024-01-15T09:02Z'
```

## Attempt 2 — 2024-01-15T15:10:00Z
Exit code: 1
```
FAILED test_dates.py::test_timezone_offset
  ValueError: unconverted data remains: +05:30
```
```

When Agent A times out, Agent B sees exactly what failed. Output is truncated to first 50 + last 50 lines.

## Hierarchical Tasks

Parent-child via dot notation:

```bash
bn create "Search feature" --verify "make test-search"
#> Created: 1

bn create "Index builder" --parent 1 --verify "cargo test index::build"
#> Created: 1.1

bn create "Query parser" --parent 1 --verify "cargo test query::parse"
#> Created: 1.2

bn tree 1
#> [ ] 1. Search feature
#>   [ ] 1.1 Index builder
#>   [ ] 1.2 Query parser
```

Parents auto-close when all children close.

## Dependencies

Auto-inferred from `produces`/`requires`:

```bash
bn create "Define schema types" --parent 1 \
  --produces "Schema,FieldType" \
  --verify "cargo test schema::types"

bn create "Build query engine" --parent 1 \
  --requires "Schema" \
  --verify "cargo test query::engine"
```

The query engine is automatically blocked until schema types closes. No explicit `bn dep add` needed.

```bash
bn status
#> ## Ready (1)
#>   1.1 [ ] Define schema types   # ready (no requires)

bn close 1.1

bn status
#> ## Ready (1)
#>   1.2 [ ] Build query engine    # now ready (producer closed)
```

### Sequential Chaining

```bash
bn create "Step 1: scaffold" --verify "cargo build"
bn create next "Step 2: implement" --verify "cargo test"
bn create next "Step 3: docs" --verify "grep -q 'API' README.md"
```

Each `next` bean automatically depends on the previous one.

## Agent Orchestration

Configure your agent once, then dispatch:

```bash
bn init --agent claude        # Or: pi, aider
# Or set directly:
bn config set run "claude -p 'read bean {id}, implement it, then run bn close {id}'"
```

`{id}` is replaced with the bean ID. The agent reads the bean, does the work, and runs `bn close`.

### Dispatching

```bash
bn run                    # Dispatch all ready beans
bn run 3                  # Dispatch a specific bean
bn run -j 8              # Up to 8 parallel agents
bn run --loop-mode        # Keep dispatching until all work is done
bn run --auto-plan        # Auto-split large beans before dispatch
bn run --review           # Adversarial review after each close
bn run --dry-run          # Preview what would be dispatched
```

### Monitoring

```bash
bn agents                 # Show running/completed agents
bn logs 3                 # View agent output for bean 3
```

### Failure Handling

```bash
bn create "fix parser" --verify "cargo test" --on-fail "retry:3"
bn create "fix data loss" --verify "make ci" --on-fail "escalate:P0"
bn close --failed 5 --reason "needs upstream API change"
```

### Planning

```bash
bn plan 3                 # Interactively split a large bean into subtasks
bn plan --auto            # Autonomous planning
bn plan --dry-run         # Preview without creating
```

## Agent Context

`bn context <id>` outputs everything an agent needs to implement a bean:

1. **Bean spec** — ID, title, verify, description, acceptance criteria
2. **Previous attempts** — what was tried and why it failed
3. **Project rules** — from `.beans/RULES.md`
4. **Dependency context** — sibling beans that produce required artifacts
5. **File structure** — signatures and imports
6. **File contents** — full source of referenced files

```bash
bn context 5                   # Complete agent briefing
bn context 5 --structure-only  # Signatures only (smaller)
bn context 5 --json            # Machine-readable
bn context                     # No ID: project-wide memory context
```

File paths come from the bean's `paths` field (`--paths` on create) and paths extracted from the description text.

## Memory System

Facts are verified project truths with TTL and staleness detection:

```bash
bn fact "DB is PostgreSQL" --verify "grep -q 'postgres' docker-compose.yml" -p
bn fact "Tests require Docker" --verify "docker info >/dev/null 2>&1" --ttl 90
bn verify-facts                    # Re-verify all facts
bn context                         # Memory context includes stale facts
bn recall "database"               # Search across all beans
```

## Commands

```bash
# Task lifecycle
bn create "title" --verify "cmd"    # Create (fail-first by default)
bn create "title" -p                # Skip fail-first (--pass-ok)
bn create next "title" --verify "cmd"  # Chain: auto-depends on last bean
bn create                           # Interactive wizard (TTY only)
bn quick "title" --verify "cmd"     # Create + claim
bn claim <id>                       # Claim existing task
bn verify <id>                      # Test without closing
bn close <id>                       # Run verify, close if passes
bn close --failed <id>              # Mark failed, release claim

# Orchestration
bn run [id] [-j N]                  # Dispatch ready beans to agents
bn run --loop-mode                  # Continuous dispatch
bn run --auto-plan                  # Auto-split large beans
bn run --review                     # Adversarial review after close
bn plan <id>                        # Decompose a large bean
bn review <id>                      # Review implementation
bn agents                           # Show running/completed agents
bn logs <id>                        # View agent output

# Querying
bn status                           # Overview: claimed, ready, blocked
bn show <id>                        # Full task details (--json, --short)
bn list                             # List with filters (--json, --ids, --format)
bn tree [id]                        # Hierarchy view
bn graph                            # Dependency graph (ASCII, Mermaid, DOT)
bn trace <id>                       # Lineage, deps, artifacts, attempts
bn recall "query"                   # Search beans by keyword
bn context [id]                     # Agent context (with ID) or memory context (without)

# Memory
bn fact "title" --verify "cmd"      # Create a verified fact
bn verify-facts                     # Re-verify all facts

# Dependencies
bn dep add <id> <dep-id>            # Add dependency
bn dep remove <id> <dep-id>        # Remove dependency

# Housekeeping
bn tidy                             # Archive closed, release stale, rebuild index
bn doctor [--fix]                   # Health check
bn sync                             # Rebuild index
bn edit <id>                        # Edit in $EDITOR
bn update <id>                      # Update fields
bn delete <id>                      # Delete a bean
bn reopen <id>                      # Reopen closed bean
bn unarchive <id>                   # Restore archived bean
bn locks [--clear]                  # View/clear file locks
bn config get/set <key> [value]     # Project configuration
bn mcp serve                        # MCP server for IDE integration
bn completions <shell>              # Shell completions (bash, zsh, fish, powershell)
```

### Pipe-Friendly

```bash
bn create "fix parser" --verify "cargo test" -p --json | jq -r '.id'
bn list --json | jq '.[] | select(.priority == 0)'
bn list --ids | bn close --stdin --force
cat spec.md | bn create "task" --description - --verify "cmd"
bn list --format '{id}\t{status}\t{title}'
```

## Configuration

Stored in `.beans/config.yaml`, checked into git.

```bash
bn config set run "claude -p 'read bean {id}, implement it, then run bn close {id}'"
bn config set plan "claude -p 'read bean {id} and split it into subtasks'"
bn config set max_concurrent 4
```

| Key | Default | Description |
|-----|---------|-------------|
| `run` | — | Command template for agent dispatch. `{id}` = bean ID. |
| `plan` | — | Command template to split large beans. |
| `max_concurrent` | `4` | Max parallel agents. |
| `max_loops` | `10` | Max agent loops before stopping (0 = unlimited). |
| `poll_interval` | `30` | Seconds between loop mode cycles. |
| `auto_close_parent` | `true` | Close parent when all children close. |
| `verify_timeout` | — | Default verify timeout in seconds. Per-bean `--verify-timeout` overrides. |
| `rules_file` | — | Path to rules file injected into `bn context`. |
| `file_locking` | `false` | Lock bean `paths` files during concurrent work. |
| `extends` | `[]` | Parent config files to inherit from. |
| `on_close` | — | Hook after close. Vars: `{id}`, `{title}`, `{status}`, `{branch}`. |
| `on_fail` | — | Hook after verify failure. Vars: `{id}`, `{title}`, `{attempt}`, `{output}`, `{branch}`. |
| `post_plan` | — | Hook after `bn plan` creates children. |
| `review.run` | — | Review agent command. Falls back to `run`. |
| `review.max_reopens` | `2` | Max review reopen cycles. |

### Config Inheritance

```yaml
# .beans/config.yaml
extends:
  - ~/.beans/global-config.yaml
project: my-app
run: "claude -p 'read bean {id}, implement it, then run bn close {id}'"
```

Child values override parent. Multiple parents applied in order (last wins).

## Documentation

- [Agent Skill](docs/SKILL.md) — Quick reference for AI agents
- [Best Practices](docs/BEST_PRACTICES.md) — Writing effective beans for agents
- `bn --help` — Full command reference

## Contributing

Contributions welcome. Fork the repo, create a feature branch, open a pull request.

## License

[Apache 2.0](LICENSE)
