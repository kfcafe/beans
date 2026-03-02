---
id: '109'
title: 'bn next: smart single-bean work recommendation'
slug: bn-next-smart-single-bean-work-recommendation
status: open
priority: 3
created_at: '2026-03-02T01:14:02.454190Z'
updated_at: '2026-03-02T01:14:02.454190Z'
labels:
- feature
- agent-dx
verify: cd /Users/asher/beans && cargo test && bn next --help 2>/dev/null | grep -qi next
tokens: 9014
tokens_updated: '2026-03-02T01:14:02.457853Z'
---

## Summary

`bn ready` lists all ready beans. `bn next` picks the single best one — the answer to "what should I work on?"

## Scoring

Rank ready beans by:
1. Priority (P0 > P1 > P2 > ...)
2. Dependency depth (beans that unblock the most other work score higher)
3. Age (older beans score higher — prevents starvation)
4. Attempt count (fewer attempts = fresher, score higher)

Return the top-scored bean. `bn next --json` for agents, `bn next -n 3` for top N.

## Example

```
$ bn next
P1  84.3  Signal handling for bn run (clean agent shutdown)
      Unblocks: 84.10, 84.11
      Age: 5 days | Attempts: 0

$ bn next --json
{"id":"84.3","title":"Signal handling...","score":0.92}
```

## Files
- `src/commands/next.rs` — new command
- `src/cli.rs` — register subcommand
- `src/commands/mod.rs` — module declaration
