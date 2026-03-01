---
id: '101'
title: Fix indentation in spawner.rs test Config structs
slug: fix-indentation-in-spawnerrs-test-config-structs
status: closed
priority: 2
created_at: '2026-02-27T07:34:48.498054Z'
updated_at: '2026-02-27T07:36:06.055523Z'
notes: "\n## Attempt 1 — 2026-02-27T07:35:50Z\nExit code: 2\n\n```\nawk: syntax error at source line 1\n context is\n\t/on_close: None|on_fail: None|post_plan: None/ { if (match($0, /^( >>>  *)/, <<< \nawk: illegal statement at source line 1\nawk: illegal statement at source line 1\n```\n"
closed_at: '2026-02-27T07:36:06.055523Z'
verify: 'cd /Users/asher/beans && gawk ''/on_close: None|on_fail: None|post_plan: None/ { if (match($0, /^( *)/, a) && length(a[1]) != 12) exit 1 }'' src/spawner.rs'
fail_first: true
checkpoint: e30502b6eb6f12893c6f5cb1f723def743f41612
attempts: 1
claimed_by: pi-agent
claimed_at: '2026-02-27T07:34:51.959685Z'
is_archived: true
tokens: 5682
tokens_updated: '2026-02-27T07:34:48.506436Z'
history:
- attempt: 1
  started_at: '2026-02-27T07:35:50.928633Z'
  finished_at: '2026-02-27T07:35:50.937938Z'
  duration_secs: 0.009
  result: fail
  exit_code: 2
  output_snippet: "awk: syntax error at source line 1\n context is\n\t/on_close: None|on_fail: None|post_plan: None/ { if (match($0, /^( >>>  *)/, <<< \nawk: illegal statement at source line 1\nawk: illegal statement at source line 1"
- attempt: 2
  started_at: '2026-02-27T07:36:06.056912Z'
  finished_at: '2026-02-27T07:36:06.075864Z'
  duration_secs: 0.018
  result: pass
  exit_code: 0
attempt_log:
- num: 1
  outcome: success
  agent: pi-agent
  started_at: '2026-02-27T07:34:51.959685Z'
  finished_at: '2026-02-27T07:36:06.055523Z'
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
- src/spawner.rs (modify — fix indentation only, 6 lines total)
