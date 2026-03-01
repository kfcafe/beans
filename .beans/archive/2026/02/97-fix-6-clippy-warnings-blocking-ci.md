---
id: '97'
title: Fix 6 clippy warnings blocking CI
slug: fix-6-clippy-warnings-blocking-ci
status: closed
priority: 1
created_at: '2026-02-27T07:30:26.069938Z'
updated_at: '2026-02-27T07:34:35.750814Z'
closed_at: '2026-02-27T07:34:35.750814Z'
verify: cd /Users/asher/beans && cargo clippy -- -D warnings 2>&1 | grep -c 'warning:' | grep '^0$'
is_archived: true
tokens: 12082
tokens_updated: '2026-02-27T07:30:26.089660Z'
---

## Task
Fix all 6 clippy warnings so `cargo clippy -- -D warnings` passes clean (required by `.github/workflows/ci.yml`).

## Current warnings (from `cargo clippy 2>&1 | grep 'warning:' | grep -v 'generated\|Finished'`):
1. `large size difference between variants` — likely in cli.rs Command enum
2. `this function has too many arguments (9/8)` — src/commands/run/ready_queue.rs:74
3. `calling push_str() using a single-character string literal` — use push() instead
4. `this function has too many arguments (9/8)` — src/commands/run/wave.rs:99
5. `this if statement can be collapsed` — combine nested ifs
6. `this map_or can be simplified` — src/mcp/resources.rs:68, use is_none_or

## Approach
- For `too many arguments`: bundle related params into a config/options struct
- For `large size difference`: Box the large variant(s)
- For the rest: apply the clippy suggestion directly
- Run `cargo clippy -- -D warnings` to verify zero warnings

## Files
- src/commands/run/ready_queue.rs (modify — struct for params)
- src/commands/run/wave.rs (modify — struct for params)
- src/mcp/resources.rs (modify — is_none_or)
- Other files as identified by clippy output

## Don't
- Don't change any public API behavior
- Don't add `#[allow(clippy::...)]` suppressions — fix the root cause
