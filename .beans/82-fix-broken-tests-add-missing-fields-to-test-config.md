---
id: '82'
title: 'Fix broken tests: add missing fields to test Config and Bean constructors'
slug: fix-broken-tests-add-missing-fields-to-test-config
status: open
priority: 2
created_at: 2026-02-23T23:51:37.885382Z
updated_at: 2026-02-23T23:51:37.885382Z
verify: cargo test
tokens: 85187
tokens_updated: 2026-02-23T23:51:37.888570Z
---

## Task
Tests don't compile because recent additions to Config (rules_file) and Bean (bean_type, last_verified, stale_after, paths, attempt_log) weren't added to test code that constructs these structs directly.

## What to fix

### Config — add `rules_file: None` to every test constructor
Files with broken Config constructors (all need `rules_file: None`):
- src/commands/adopt.rs:226
- src/commands/close.rs:1390, 1500, 2458
- src/commands/create.rs:366
- src/commands/fact.rs:216
- src/commands/memory_context.rs:340
- src/commands/quick.rs:208
- src/spawner.rs:627, 649
- tests/cli_tests.rs:15
- tests/adopt_test.rs:20

### Bean — add memory fields to round_trip_full_bean test
- src/bean.rs:613 — the `round_trip_full_bean` test constructs a Bean literal missing: `bean_type`, `last_verified`, `stale_after`, `paths`, `attempt_log`

## Context

### Config struct (src/config.rs)
The `rules_file` field is `Option<String>` with `#[serde(default)]`. Set to `None` in tests.

### Bean struct memory fields (src/bean.rs)
```rust
pub bean_type: String,           // default "task"
pub last_verified: Option<DateTime<Utc>>,  // default None
pub stale_after: Option<DateTime<Utc>>,    // default None
pub paths: Vec<String>,          // default empty
pub attempt_log: Vec<AttemptRecord>,  // default empty
```

## Acceptance
- [ ] cargo test compiles and all tests pass
- [ ] No test logic changes — only add missing fields to struct literals
