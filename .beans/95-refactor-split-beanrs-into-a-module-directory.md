---
id: '95'
title: 'refactor: Split bean.rs into a module directory'
slug: refactor-split-beanrs-into-a-module-directory
status: open
priority: 2
created_at: 2026-02-27T05:22:29.442936Z
updated_at: 2026-02-27T05:22:29.442936Z
verify: cargo test --lib bean
tokens: 13812
tokens_updated: 2026-02-27T05:22:29.444360Z
---

## Task
Split the monolithic `src/bean.rs` (1,625 lines) into a module directory separating types from behavior.

## Target Structure
- `src/bean/mod.rs` — `Bean` struct, its `impl Bean` block, serde helper functions (`default_priority`, `default_max_attempts`, `is_zero`, `is_default_max_attempts`, `is_false`, `default_bean_type`, `is_default_bean_type`), and `validate_priority`
- `src/bean/types.rs` — Supporting enums and structs: `Status` (+ Display impl), `RunResult`, `RunRecord`, `OnFailAction`, `OnCloseAction`, `AttemptOutcome`, `AttemptRecord`

## Steps
1. Create `src/bean/` directory
2. Move `src/bean.rs` to `src/bean/mod.rs`
3. Extract all supporting types into `types.rs`
4. Add `pub mod types;` to mod.rs and `pub use types::*;` to re-export everything
5. Move type-related tests to types.rs, keep Bean tests in mod.rs

## External API (must not break)
These are all used across the codebase via `crate::bean::`:
- `Bean`, `Status`, `RunResult`, `RunRecord`, `OnFailAction`, `OnCloseAction`, `AttemptOutcome`, `AttemptRecord`, `validate_priority`
- All must remain importable as `crate::bean::X` after the split

## Constraints
- Do NOT change any type definitions, derives, or function signatures
- Do NOT modify files outside src/bean/
- Use `pub use types::*;` in mod.rs so all external imports keep working unchanged
- `lib.rs` already has `pub mod bean;` — works for both file and directory modules
