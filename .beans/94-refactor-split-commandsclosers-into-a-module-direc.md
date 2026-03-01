---
id: '94'
title: 'refactor: Split commands/close.rs into a module directory'
slug: refactor-split-commandsclosers-into-a-module-direc
status: open
priority: 2
created_at: 2026-02-27T05:22:12.571827Z
updated_at: 2026-02-27T05:22:12.571827Z
verify: cargo test --lib commands::close
tokens: 28471
tokens_updated: 2026-02-27T05:22:12.573929Z
---

## Task
Split the monolithic `src/commands/close.rs` (2,869 lines) into a module directory with focused files.

## Target Structure
- `src/commands/close/mod.rs` — The `cmd_close` handler and parent-chain logic: `all_children_closed`, `auto_close_parent`, `find_root_parent`, `truncate_to_char_boundary`
- `src/commands/close/verify.rs` — Verification execution: `VerifyResult` struct, `run_verify`, `truncate_output`, `format_failure_note`

## Steps
1. Create `src/commands/close/` directory
2. Move `src/commands/close.rs` to `src/commands/close/mod.rs`
3. Extract verify-related types and functions into `verify.rs`
4. Add `mod verify;` to mod.rs, use `verify::{VerifyResult, run_verify, ...}` internally
5. Re-export all public items from mod.rs so external callers don't change
6. Move tests to their relevant files

## External API (must not break)
- `commands::close::cmd_close` (re-exported via commands/mod.rs)
- No other symbols are imported externally

## Constraints
- Do NOT change any function signatures or behavior
- Do NOT modify files outside src/commands/close/
- Re-export everything public from mod.rs so no external imports break
