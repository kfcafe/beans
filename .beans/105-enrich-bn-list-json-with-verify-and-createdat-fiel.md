---
id: '105'
title: Enrich bn list --json with verify and created_at fields
slug: enrich-bn-list-json-with-verify-and-createdat-fiel
status: open
priority: 2
created_at: '2026-03-02T01:10:32.227053Z'
updated_at: '2026-03-02T01:10:32.227053Z'
labels:
- enhancement
- agent-dx
verify: cd /Users/asher/beans && cargo test && bn list --json 2>/dev/null | jq '.[0] | has("verify", "created_at")' | grep -q true
tokens: 10909
tokens_updated: '2026-03-02T01:10:32.230269Z'
---

## Problem

`bn list --json` is missing two fields that `bn show --json` has: `verify` (the actual command string) and `created_at`. This forces agents to make N+1 calls — one `bn list` to find work, then `bn show` per bean to see the verify command.

## Current state

`IndexEntry` in `src/index.rs` already carries most fields (parent, dependencies, produces, requires, claimed_by, assignee, attempts). It has `has_verify: bool` but not the verify command itself. It also lacks `created_at`.

The `bn list --json` output is currently ~20KB for 74 beans. Adding these two fields should bring it to ~25KB — still very cheap for a single agent call.

## What to change

### 1. Add fields to `IndexEntry` (`src/index.rs`)

Add to the struct:
```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub verify: Option&lt;String&gt;,
pub created_at: DateTime&lt;Utc&gt;,
```

### 2. Update `From<Bean>` impl (`src/index.rs`)

Map `bean.verify.clone()` and `bean.created_at` into the new fields.

### 3. Update all `IndexEntry` construction sites

Any place that constructs `IndexEntry` directly (tests, etc.) needs the new fields. Search for `IndexEntry {` across the codebase.

### 4. Keep `has_verify` for backward compat

Don't remove `has_verify` — it's useful for quick filtering without parsing the verify string.

## What NOT to change

- Don't add `description` to the index — it's 125KB of bulk and belongs in `bn show`
- Don't change the human-readable `bn list` output (tree format)
- Don't change `bn show --json` output

## Files
- `src/index.rs` — IndexEntry struct and From impl
- `src/commands/list.rs` — tests that construct IndexEntry

## Acceptance criteria
- `bn list --json` output includes `verify` and `created_at` for each bean
- `bn show --json` still works unchanged
- All existing tests pass
- Index rebuild works correctly with new fields
