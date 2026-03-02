---
id: '104'
title: Fix flaky close.rs tests (temp dir race conditions)
slug: fix-flaky-closers-tests-temp-dir-race-conditions
status: open
priority: 0
created_at: '2026-03-02T00:46:09.748647Z'
updated_at: '2026-03-02T01:06:25.148376Z'
notes: |-
  ---
  2026-03-02T01:06:25.145636+00:00
  ## Attempt Failed (20m10s, 2.6M tokens, $2.387)

  ### What was tried

  - 0 tool calls over 44 turns in 20m10s

  ### Why it failed

  - Timeout (20m)

  ### Verify command

  `cargo test 2>&1 | tail -1 | grep -q 'test result: ok'`

  ### Suggestion for next attempt

  - Agent ran out of time. Consider increasing the timeout or simplifying the task scope.
labels:
- bug
- hn-launch
verify: 'cargo test 2>&1 | tail -1 | grep -q ''test result: ok'''
tokens: 35943
tokens_updated: '2026-03-02T01:06:25.148375Z'
---

Flaky tests when running full suite (`cargo test`). Tests pass individually but fail when run together due to temp directory race conditions.

Failing tests:
- `commands::close::tests::test_close_with_passing_pre_close_hook` — panics with "cannot change to temp dir: No such file or directory" during `git merge`
- `commands::close::tests::worktree_merge::test_close_with_merge_conflict_aborts` — likely same root cause
- `commands::close::verify_timeout_tests::verify_timeout_does_not_affect_fast_commands` — intermittent

Root cause: Tests in `src/commands/close.rs` share temp directory state or have race conditions when git worktree operations run in parallel. Each test needs fully isolated temp directories.

Files to examine:
- `src/commands/close.rs` — look at the test helper functions creating temp dirs, especially around worktree and git merge tests
- Focus on the `tests` and `worktree_merge` and `verify_timeout_tests` modules

Fix: Ensure each test creates its own unique temp directory and doesn't share any git state with other tests.
