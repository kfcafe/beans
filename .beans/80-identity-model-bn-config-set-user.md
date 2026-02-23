---
id: '80'
title: 'Identity model: bn config set user'
slug: identity-model-bn-config-set-user
status: open
priority: 2
created_at: 2026-02-23T12:52:08.498902Z
updated_at: 2026-02-23T12:52:08.498902Z
verify: bn config set user testuser 2>/dev/null && bn config get user 2>&1 | grep -q testuser
tokens: 46147
tokens_updated: 2026-02-23T12:52:08.505145Z
---

## Task

Add a user identity concept so claimed_by is meaningful and agents/humans are distinguishable.

## What to implement

### Set identity
```bash
bn config set user alice           # Set your name
bn config set user.email alice@co  # Optional email (for git integration)
```

Identity stored in `.beans/config.yaml` (project-level) or `~/.config/beans/config.yaml` (global, preferred).

### Use identity
- `bn claim <id>` sets `claimed_by: alice` instead of `claimed_by: agent-<pid>`
- `bn agents` shows who spawned each agent
- `bn status` groups by user: 'Alice has 3 claimed, Bob has 2'  
- `bn list --mine` filters to beans claimed by current user
- `bn create` sets `created_by: alice` in frontmatter

### Identity resolution (priority order)
1. `bn config get user` (project config)
2. `~/.config/beans/config.yaml` user field (global config)
3. `git config user.name` (fallback)
4. `$USER` / `whoami` (last resort)

### Agent identity
When `bn run` spawns agents, they get identity like `alice/agent-1`, `alice/agent-2` — namespaced under the user who spawned them.

## Files
- src/config.rs (modify — add user, user.email fields + global config support)
- src/commands/claim.rs (modify — use resolved identity for claimed_by)
- src/commands/create.rs (modify — add created_by field)
- src/commands/status.rs (modify — group by user)
- src/commands/list.rs (modify — add --mine filter)
- src/orchestrator.rs (modify — namespace agent identity under user)
- src/bean.rs (modify — add created_by field)

## Edge Cases
- No identity configured → fall back to git config, then $USER
- Global vs project config → project overrides global
- Agent crashed, claimed_by is stale → bn tidy already handles this

## Acceptance
- [ ] `bn config set user X` and `bn config get user` roundtrip works
- [ ] `bn claim` uses resolved identity
- [ ] `bn list --mine` filters correctly
- [ ] Falls back to git user.name when no explicit config
