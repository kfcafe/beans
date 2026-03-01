---
id: '96'
title: 'refactor: Split commands/create.rs into a module directory'
slug: refactor-split-commandscreaters-into-a-module-dire
status: open
priority: 2
created_at: 2026-02-27T05:22:37.867090Z
updated_at: 2026-02-27T05:22:37.867090Z
verify: cargo test --lib commands::create
tokens: 15677
tokens_updated: 2026-02-27T05:22:37.869045Z
---

## Task
Split `src/commands/create.rs` (1,958 lines) into a module directory. This file has ~400 lines of code and ~1,560 lines of tests — the main win is separating tests from implementation.

## Target Structure
- `src/commands/create/mod.rs` — `CreateArgs` struct, `assign_child_id`, `parse_on_fail`, `cmd_create`, `cmd_create_next`
- `src/commands/create/tests.rs` — All test code (the `mod tests` block, ~1,560 lines)

## Steps
1. Create `src/commands/create/` directory
2. Move `src/commands/create.rs` to `src/commands/create/mod.rs`
3. Extract the entire `mod tests { ... }` block into `tests.rs`
4. Replace the inline `mod tests` in mod.rs with `#[cfg(test)] mod tests;`
5. In tests.rs, add any necessary `use super::*;` and imports from the original test module
6. Re-export all public items from mod.rs

## External API (must not break)
These are imported by other modules:
- `commands::create::CreateArgs`
- `commands::create::assign_child_id`
- `commands::create::parse_on_fail`
- `commands::create::cmd_create` and `cmd_create_next` (re-exported via commands/mod.rs)

## Constraints
- Do NOT change any function signatures or behavior
- Do NOT modify files outside src/commands/create/
- All public items must remain importable at the same paths
