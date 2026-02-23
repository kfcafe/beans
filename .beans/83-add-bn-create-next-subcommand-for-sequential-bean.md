---
id: '83'
title: Add 'bn create next' subcommand for sequential bean chaining
slug: add-bn-create-next-subcommand-for-sequential-bean
status: open
priority: 2
created_at: 2026-02-23T23:52:02.594320Z
updated_at: 2026-02-23T23:52:07.358762Z
dependencies:
- '82'
verify: cargo test create_next && bn create next --help 2>&1 | grep -qi 'next\|chain\|after'
fail_first: true
tokens: 34056
tokens_updated: 2026-02-23T23:52:02.595653Z
---

## Task
Add a `bn create next` subcommand that creates a bean with an automatic dependency on the most recently created bean. This enables easy sequential chaining:

```bash
bn create "Design auth types" --verify "test -f docs/auth.md" -p
bn create next "Implement JWT" --verify "cargo test jwt"
bn create next "Integration tests" --verify "cargo test auth::integration"
```

Each `next` automatically adds a dependency on `@latest` (the previously created bean). Equivalent to `bn create "..." --deps @latest`.

## What to implement

1. Add a `Next` variant to the `Command` enum in src/cli.rs with the same fields as `Create` (minus --deps, since we auto-set it)
2. In src/commands/create.rs (or a new src/commands/next.rs), implement `cmd_create_next` that:
   - Resolves `@latest` to the most recently created bean ID
   - Calls the normal create flow with that ID added to deps
   - Prints the dependency in the output so the user sees the chain
3. Wire it up in src/main.rs

## Files
- src/cli.rs (modify — add Next subcommand)
- src/commands/create.rs (modify — add cmd_create_next or extend cmd_create)
- src/commands/mod.rs (modify if new file)
- src/main.rs (modify — handle Next command)

## Context

### @latest selector resolution (src/selector.rs)
```
@latest resolves to the most recently created bean ID.
The selector system already exists — use resolve_selector().
```

### Current Create command in cli.rs
The Create variant has all the fields we need. Next should accept the same fields except --deps (auto-populated) and print the auto-dependency.

### How deps work in create (src/commands/create.rs)
The `deps` field is `Option<String>` — comma-separated IDs. To chain, set it to the resolved @latest ID.

## Edge Cases
- No previous bean exists (@latest fails) → error with helpful message
- User also passes --deps → merge: auto-dep from @latest + explicit deps
- Works with --parent (child bean that also depends on @latest)

## Acceptance
- [ ] `bn create next "title" --verify "cmd"` creates a bean depending on @latest
- [ ] Chaining 3 beans with `next` creates a linear dependency chain
- [ ] `bn create next --help` shows the subcommand
- [ ] cargo test passes
