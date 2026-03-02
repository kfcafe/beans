---
id: '108'
title: Duplicate detection on bn create
slug: duplicate-detection-on-bn-create
status: open
priority: 3
created_at: '2026-03-02T01:13:51.366617Z'
updated_at: '2026-03-02T01:13:51.366617Z'
labels:
- feature
- agent-dx
verify: cd /Users/asher/beans && cargo test
tokens: 25891
tokens_updated: '2026-03-02T01:13:51.369337Z'
---

## Summary

When creating a bean, check existing open beans for similar titles and warn before creating a duplicate. Agents are prolific task-creators and routinely create redundant beans.

## Approach

Simple fuzzy title matching — normalize titles (lowercase, strip punctuation, split words) and check for high overlap with existing open beans. No embedding model needed.

For example, "Fix auth timeout" and "Fix authentication timeout handling" should trigger a warning.

## Behavior

- On match: print warning with the similar bean(s), ask for confirmation (or skip in non-interactive/--force mode)
- Threshold: configurable, default ~70% word overlap
- Only check open/in_progress beans (not closed/archived)
- `--force` or `--no-check-dupes` skips the check

## Files
- `src/commands/create.rs` — add duplicate check before creation
- `src/commands/quick.rs` — same check
- `src/util.rs` or new `src/similarity.rs` — title similarity function
