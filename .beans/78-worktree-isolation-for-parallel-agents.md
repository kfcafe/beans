---
id: '78'
title: Worktree isolation for parallel agents
slug: worktree-isolation-for-parallel-agents
status: open
priority: 2
created_at: 2026-02-23T12:14:09.908279Z
updated_at: 2026-02-23T12:14:09.908279Z
verify: cargo test worktree && bn run --help 2>&1 | grep -qi 'worktree\|isolat'
tokens: 50884
tokens_updated: 2026-02-23T12:14:09.913528Z
---

## Task

Complete the git worktree isolation system so parallel agents work in separate worktrees and merge back on close. This builds on the existing work in beans 12, 12.1, 12.3.

## Motivation  
TSK's strongest differentiator is Docker-based isolation. Git worktrees are a lighter-weight alternative that achieves the same goal: parallel agents can't interfere with each other's work. No port conflicts, no file contention, no merge disasters.

## What to implement

### Agent isolation flow:
1. `bn run` (or `bn claim --worktree`) creates a git worktree for the bean
2. Agent runs in the worktree directory (isolated copy of the repo)
3. Agent does work, commits to the worktree's branch
4. `bn close` merges the worktree branch back to the main branch
5. Worktree is cleaned up after successful merge

### Commands:
```bash
bn run                          # Auto-creates worktrees for each agent
bn run --no-worktree            # Opt out (run in main repo)
bn config set worktree true     # Enable by default
```

### Worktree management:
- Worktrees created in `.beans/worktrees/<bean-id>/` or a configurable path
- Branch naming: `beans/<bean-id>-<slug>`
- Auto-cleanup on close (successful or failed)
- `bn agents` shows which worktree each agent is in

### Merge strategy:
- Default: fast-forward if possible, merge commit if not
- On conflict: mark bean as failed with conflict details in notes
- `bn config set merge_strategy rebase|merge|ff-only`

## Files
- src/worktree.rs (modify — complete the worktree module)
- src/orchestrator.rs (modify — create worktree before spawning agent)
- src/spawner.rs (modify — set cwd to worktree path)
- src/commands/close.rs (modify — merge worktree on close)
- src/commands/agents.rs (modify — show worktree info)
- src/config.rs (modify — add worktree config keys)
- tests/ (add worktree integration tests)

## Context

### Existing worktree work
Beans 12, 12.1, 12.3 have started this. Check their status and build on whatever's been implemented. Key files: src/worktree.rs, src/spawner.rs.

## Edge Cases
- Merge conflicts → bean stays open, conflict details in notes, agent can retry
- Worktree already exists for bean → reuse it
- Agent crashes mid-work → worktree persists, cleanup on next `bn tidy`
- Nested worktrees (worktree inside worktree) → prevent this
- Dirty main branch → still works (worktree is a snapshot)

## Acceptance
- [ ] `bn run` creates isolated worktrees per agent
- [ ] Agents work in separate directories, no file contention
- [ ] `bn close` merges worktree branch back
- [ ] Merge conflicts are handled gracefully
- [ ] Worktrees cleaned up after close
- [ ] cargo test worktree passes
