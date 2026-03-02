---
id: '106'
title: 'TTY detection: auto-JSON when stdout is piped'
slug: tty-detection-auto-json-when-stdout-is-piped
status: open
priority: 2
created_at: '2026-03-02T01:13:23.436157Z'
updated_at: '2026-03-02T01:13:23.436157Z'
labels:
- enhancement
- agent-dx
verify: 'cd /Users/asher/beans && cargo test && echo ''{}'' | bn list 2>/dev/null | head -1 | grep -q ''^\['' '
tokens: 11701
tokens_updated: '2026-03-02T01:13:23.439568Z'
---

## Summary

When stdout is a TTY, output human-friendly format (tree, tables). When stdout is a pipe or redirect, output JSON automatically. This matches what rg, fd, eza do for colors/formatting.

Agents almost always capture output (pipe or redirect). Humans almost always read terminals. So it Just Works for both without configuration.

- `bn list` at terminal → pretty tree (current behavior)
- `bn list | jq` → auto-JSON
- `bn list > file` → auto-JSON
- `bn list --json` → explicit JSON override (already works)
- `bn list --no-json` → explicit pretty override even in pipe

## Implementation

Check `atty::is(atty::Stream::Stdout)` or use `std::io::IsTerminal` (stable since Rust 1.70). Apply to all commands that support `--json`: list, show, status, ready, blocked, verify, context, agents, trace, recall, stats.

## Files
- `src/commands/list.rs`, `show.rs`, `status.rs`, `ready_queue.rs`, and other commands with --json
- `src/cli.rs` — possibly add global --no-json flag
