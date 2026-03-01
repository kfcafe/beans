---
id: '98'
title: Fix indentation in spawner.rs test Config structs
slug: fix-indentation-in-spawnerrs-test-config-structs
status: closed
priority: 2
created_at: '2026-02-27T07:30:26.128648Z'
updated_at: '2026-02-27T07:34:35.750814Z'
closed_at: '2026-02-27T07:34:35.750814Z'
verify: cd /Users/asher/beans && ! grep -Pn '^\s{8}on_close:|^\s{8}on_fail:|^\s{8}post_plan:' src/spawner.rs
is_archived: true
tokens: 5678
tokens_updated: '2026-02-27T07:30:26.136585Z'
---

## Task
Fix inconsistent indentation in two test Config struct literals in `src/spawner.rs`.

The `on_close`, `on_fail`, and `post_plan` fields use 8-space indent instead of 12-space, misaligned with the other fields in the same struct literal.

## Steps
1. Open `src/spawner.rs`
2. Find the two Config struct literals in tests (~line 637 and ~664)
3. Change the 3 fields from 8-space indent to 12-space indent to match siblings
4. Verify `cargo test --lib spawner` passes

## Current (wrong):
```rust
            file_locking: false,
        on_close: None,
        on_fail: None,
        post_plan: None,
```

## Expected (fixed):
```rust
            file_locking: false,
            on_close: None,
            on_fail: None,
            post_plan: None,
```

## Files
- src/spawner.rs (modify — fix indentation only)
