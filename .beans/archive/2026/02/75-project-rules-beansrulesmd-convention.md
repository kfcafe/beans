---
id: '75'
title: 'Project rules: .beans/RULES.md convention'
slug: project-rules-beansrulesmd-convention
status: closed
priority: 2
created_at: 2026-02-23T12:13:10.047614Z
updated_at: 2026-02-27T06:16:24.821385Z
closed_at: 2026-02-27T06:16:24.821385Z
verify: cargo test rules && test -f .beans/RULES.md
claimed_by: pi-agent
claimed_at: 2026-02-27T06:12:00.530997Z
is_archived: true
tokens: 14043
tokens_updated: 2026-02-23T12:13:10.050608Z
history:
- attempt: 1
  started_at: 2026-02-27T06:16:24.823656Z
  finished_at: 2026-02-27T06:16:24.945033Z
  duration_secs: 0.121
  result: pass
  exit_code: 0
outputs:
  text: |-
    running 9 tests
    test commands::context::tests::format_rules_section_wraps_with_delimiters ... ok
    test commands::plan::tests::build_prompt_includes_decomposition_rules ... ok
    test commands::context::tests::load_rules_returns_none_when_file_missing ... ok
    test commands::context::tests::load_rules_returns_none_when_file_empty ... ok
    test commands::context::tests::load_rules_uses_custom_rules_file_path ... ok
    test commands::init::tests::init_creates_rules_md_stub ... ok
    test commands::init::tests::init_does_not_overwrite_existing_rules_md ... ok
    test commands::context::tests::load_rules_returns_content_when_present ... ok
    test commands::run::ready_queue::tests::assemble_bean_context_includes_rules ... ok

    test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 863 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.00s
---

## Task

Add support for a .beans/RULES.md file that gets automatically injected into agent context for every bean.

## Motivation
Shrimp Task Manager's 'project rules' feature lets you define coding standards that persist across all agent sessions. Agents working on any bean in the project automatically see the rules — coding conventions, architectural decisions, forbidden patterns, etc.

## What to implement

1. When `bn context <id>` is called, check for `.beans/RULES.md` and prepend its contents as a 'Project Rules' section before the bean-specific context
2. `bn init` should create a stub `.beans/RULES.md` with example content (commented out or with placeholder sections)
3. `bn config set rules_file <path>` to allow overriding the default path
4. Rules content should be injected with a clear delimiter so agents know it's project-level, not task-level
5. Token budget: rules get a separate budget (default ~1000 tokens) that doesn't compete with bean context

## Files
- src/commands/context.rs (modify — inject rules before bean context)
- src/commands/init.rs (modify — create stub RULES.md)
- src/config.rs (modify — add rules_file config key)
- tests/ (add rules integration tests)

## Context format
```
═══ PROJECT RULES ═══════════════════════════════════════════
<contents of RULES.md>
═════════════════════════════════════════════════════════════

═══ BEANS CONTEXT ═══════════════════════════════════════════
<existing bean context>
```

## Edge Cases
- Missing RULES.md → skip silently (not an error)
- Empty RULES.md → skip section entirely
- Very large RULES.md → warn but still inject (user's responsibility to keep it reasonable)

## Acceptance
- [ ] `bn context <id>` includes RULES.md content when file exists
- [ ] `bn init` creates a stub RULES.md
- [ ] Missing RULES.md doesn't break anything
- [ ] cargo test passes
