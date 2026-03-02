---
id: '107'
title: 'bn onboard: smart agent setup across coding agents'
slug: bn-onboard-smart-agent-setup-across-coding-agents
status: open
priority: 2
created_at: '2026-03-02T01:13:39.885086Z'
updated_at: '2026-03-02T01:13:39.885086Z'
labels:
- feature
- agent-dx
verify: cd /Users/asher/beans && cargo test && bn onboard --help 2>/dev/null | grep -qi onboard
tokens: 9196
tokens_updated: '2026-03-02T01:13:39.886731Z'
---

## Summary

An intelligent onboarding command that detects which coding agents are configured in the project and writes the appropriate integration files. Idempotent — safe to run multiple times (uses marker comments like `# [beans-onboard]` to detect existing setup).

## Detection → Action

| Detects | Action |
|---------|--------|
| `.claude/settings.json` | Add SessionStart + PreCompact hooks for `bn context` |
| `CLAUDE.md` | Append beans workflow instructions |
| `AGENTS.md` | Append beans skill reference + workflow |
| `.cursor/rules` or `.cursorrules` | Append beans-aware rules |
| `.pi/` directory | Write skill to `.pi/agent/skills/beans/SKILL.md` |
| `.cline/` or `cline_docs/` | Write MCP config or rules |
| `opencode.yaml` or `.opencode/` | Write plugin or config |
| `.aider.conf.yml` | Append conventions |
| None of the above | Offer to create AGENTS.md from scratch |

## Design principles

- Detect, don't ask — scan the project and act
- Idempotent — marker comments prevent double-append
- Minimal — don't dump a wall of instructions, just wire the tool in
- Show what was done — print a summary of files modified/created

## Example

```
$ bn onboard
Detected: Claude Code, pi, Cursor
  ✓ .claude/settings.json — added SessionStart hook
  ✓ .pi/agent/skills/beans/SKILL.md — created skill
  ✓ .cursorrules — appended beans workflow
Done. 3 agents configured.
```

## Files
- `src/commands/onboard.rs` — new command
- `src/cli.rs` — register subcommand
- `src/commands/mod.rs` — module declaration
