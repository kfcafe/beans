---
id: '75'
title: 'Project rules: .beans/RULES.md convention'
slug: project-rules-beansrulesmd-convention
status: open
priority: 2
created_at: 2026-02-23T12:13:10.047614Z
updated_at: 2026-02-23T12:13:10.047614Z
verify: cargo test rules && test -f .beans/RULES.md
tokens: 14043
tokens_updated: 2026-02-23T12:13:10.050608Z
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
