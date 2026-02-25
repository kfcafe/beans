---
id: '87'
title: 'Test bean: verify bean_run works'
slug: test-bean-verify-beanrun-works
status: closed
priority: 2
created_at: 2026-02-25T09:03:12.779473Z
updated_at: 2026-02-25T09:03:23.994076Z
closed_at: 2026-02-25T09:03:23.994076Z
verify: test -f /tmp/bean-run-test-sentinel && rm /tmp/bean-run-test-sentinel
fail_first: true
claimed_by: pi-agent
claimed_at: 2026-02-25T09:03:16.957587Z
is_archived: true
tokens: 32
tokens_updated: 2026-02-25T09:03:12.780428Z
history:
- attempt: 1
  started_at: 2026-02-25T09:03:23.994303Z
  finished_at: 2026-02-25T09:03:24.001986Z
  duration_secs: 0.007
  result: pass
  exit_code: 0
---

Create the file /tmp/bean-run-test-sentinel. That is all. Run: touch /tmp/bean-run-test-sentinel
