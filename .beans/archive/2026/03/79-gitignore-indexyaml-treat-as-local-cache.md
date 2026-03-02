---
id: '79'
title: Gitignore index.yaml — treat as local cache
slug: gitignore-indexyaml-treat-as-local-cache
status: closed
priority: 2
created_at: '2026-02-23T12:51:53.142977Z'
updated_at: '2026-03-02T00:50:11.223869Z'
closed_at: '2026-03-02T00:50:11.223869Z'
verify: grep -q 'index.yaml' .beans/.gitignore 2>/dev/null || grep -q 'index.yaml' .gitignore
fail_first: true
checkpoint: '6012d23d80212314dee9be4624d3f5e44f5a1786'
claimed_by: pi-agent
claimed_at: '2026-03-02T00:46:15.144902Z'
is_archived: true
tokens: 6411
tokens_updated: '2026-02-23T12:51:53.147230Z'
history:
- attempt: 1
  started_at: '2026-03-02T00:50:11.226773Z'
  finished_at: '2026-03-02T00:50:11.281014Z'
  duration_secs: 0.054
  result: pass
  exit_code: 0
attempt_log:
- num: 1
  outcome: success
  agent: pi-agent
  started_at: '2026-03-02T00:46:15.144902Z'
  finished_at: '2026-03-02T00:50:11.223869Z'
---

## Task

Stop tracking .beans/index.yaml in git. It's a regenerable cache (bn sync rebuilds it from the .md files), and it causes merge conflicts for anyone collaborating on the same repo.

## What to implement

1. Add `index.yaml` to `.beans/.gitignore` (or to the root `.gitignore` under the .beans section)
2. Remove index.yaml from git tracking: `git rm --cached .beans/index.yaml`
3. Make sure `bn init` creates the gitignore entry by default
4. `bn sync` should auto-run on first command if index.yaml is missing (it may already do this — verify)
5. Document that index.yaml is local-only

## Files
- src/commands/init.rs (modify — add index.yaml to gitignore during init)
- .gitignore or .beans/.gitignore (modify)
- docs/ (update if index is mentioned as tracked)

## Edge Cases
- Existing repos that already track index.yaml — bn doctor could suggest removing it
- Missing index.yaml on fresh clone — bn sync should handle this transparently
- Performance: first command after clone needs to rebuild index — should be fast for <1000 beans

## Acceptance
- [ ] index.yaml is in gitignore
- [ ] bn commands work fine with missing index.yaml (auto-rebuilds)
- [ ] bn init sets up the gitignore entry
