---
id: '100'
title: Fix 6 clippy warnings blocking CI
slug: fix-6-clippy-warnings-blocking-ci
status: closed
priority: 1
created_at: '2026-02-27T07:34:48.443825Z'
updated_at: '2026-02-27T07:40:58.038908Z'
closed_at: '2026-02-27T07:40:58.038908Z'
verify: cd /Users/asher/beans && cargo clippy -- -D warnings 2>&1 | tail -1 | grep -q 'Finished'
fail_first: true
checkpoint: e30502b6eb6f12893c6f5cb1f723def743f41612
claimed_by: pi-agent
claimed_at: '2026-02-27T07:34:54.034308Z'
is_archived: true
tokens: 12099
tokens_updated: '2026-02-27T07:34:48.466252Z'
history:
- attempt: 1
  started_at: '2026-02-27T07:40:58.044792Z'
  finished_at: '2026-02-27T07:40:58.200636Z'
  duration_secs: 0.155
  result: pass
  exit_code: 0
attempt_log:
- num: 1
  outcome: success
  agent: pi-agent
  started_at: '2026-02-27T07:34:54.034308Z'
  finished_at: '2026-02-27T07:40:58.038908Z'
---

## Task
Fix all 6 clippy warnings so `cargo clippy -- -D warnings` exits cleanly (required by `.github/workflows/ci.yml`).

## Current warnings (run `cargo clippy 2>&1 | grep 'warning:'`):
1. `large size difference between variants` — Box the large variant in the CLI Command enum
2. `this function has too many arguments (9/8)` — src/commands/run/ready_queue.rs:74 — bundle params into a struct
3. `calling push_str() using a single-character string literal` — use `push('\n')` instead
4. `this function has too many arguments (9/8)` — src/commands/run/wave.rs:99 — bundle params into a struct
5. `this if statement can be collapsed` — combine nested `if` into `if a &amp;&amp; b`
6. `this map_or can be simplified` — src/mcp/resources.rs:68, use `.is_none_or()` instead

## Approach
- For `too many arguments`: bundle related params into a RunConfig/RunOptions struct
- For `large size difference`: Box the large variant(s) in the enum
- For the rest: apply the clippy suggestion directly

## Files
- src/commands/run/ready_queue.rs (modify)
- src/commands/run/wave.rs (modify)
- src/mcp/resources.rs (modify)
- Other files as identified by full `cargo clippy` output

## Don't
- Don't change any public API behavior
- Don't add `#[allow(clippy::...)]` suppressions — fix the root cause
- Don't modify tests unless they reference changed signatures
