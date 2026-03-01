---
id: '99'
title: Hoist Config::load out of per-bean loop in cmd_close
slug: hoist-configload-out-of-per-bean-loop-in-cmdclose
status: closed
priority: 2
created_at: '2026-02-27T07:30:26.161384Z'
updated_at: '2026-02-27T07:33:15.181412Z'
closed_at: '2026-02-27T07:33:15.181412Z'
verify: 'cd /Users/asher/beans && cargo test close -- --include-ignored 2>&1 | tail -3 | grep -q ''test result: ok'''
is_archived: true
tokens: 33004
tokens_updated: '2026-02-27T07:30:26.169111Z'
history:
- attempt: 1
  started_at: '2026-02-27T07:33:15.186653Z'
  finished_at: '2026-02-27T07:33:31.205994Z'
  duration_secs: 16.019
  result: pass
  exit_code: 0
---

## Task
`Config::load(beans_dir)` is called inside the per-bean `for id in &ids` loop in `cmd_close` — once for the `on_fail` hook and once for the `on_close` hook. This is redundant I/O on every bean in a batch close. Load it once before the loop.

## Steps
1. Open `src/commands/close.rs`
2. In `cmd_close()`, add `let config = Config::load(beans_dir).ok();` before the `for id in &ids` loop
3. Replace the two `if let Ok(config) = Config::load(beans_dir)` blocks inside the loop with `if let Some(ref config) = config`
4. The `auto_close_parent` config load (~line 690) can also use the pre-loaded config
5. Run close tests to verify

## Context
```rust
// Current (inside loop, called per-bean):
if let Ok(config) = Config::load(beans_dir) {
    if let Some(ref on_fail_template) = config.on_fail { ... }
}
// ...later in same loop iteration:
if let Ok(config) = Config::load(beans_dir) {
    if let Some(ref on_close_template) = config.on_close { ... }
}
```

## Files
- src/commands/close.rs (modify — hoist config load)

## Don't
- Don't change hook behavior or ordering
- Don't modify tests
