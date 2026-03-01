---
id: '41'
title: Beans Memory System
slug: beans-memory-system
status: closed
priority: 2
created_at: 2026-02-05T18:33:19.000683Z
updated_at: 2026-02-27T06:32:15.203265Z
closed_at: 2026-02-27T06:32:15.203265Z
close_reason: 'Implemented memory system: attempt tracking in claim/close/release, bn close --failed, suspect propagation in verify-facts, SKILL.md memory workflow docs'
verify: cd packages/beans && npm test
claimed_by: pi-agent
claimed_at: 2026-02-27T06:10:24.619752Z
is_archived: true
tokens: 1391
tokens_updated: 2026-02-05T18:47:19.828247Z
---

# Beans Memory System

Transform beans from a task tracker into a verified knowledge graph with temporal state.

## Core Insight

Tasks and memories are the same data structure with different lifecycles:
- Task: "needs to happen" → work → "happened" (verified)  
- Memory: "is true" → time → "still true?" (re-verified)

Both need hierarchy, relationships, search, scoping, and validation. Beans already has the primitives.

## Knowledge Separation

| Type | Where | Verification |
|------|-------|--------------|
| Verifiable facts | `bn fact` | **Required** — that's the point |
| Unverifiable knowledge | `agents.md` | None — prose, conventions, preferences |
| Tasks | `bn create` | Required — completion gate |

**If you can't write a verify command, it's not a bn fact — it belongs in agents.md.**

## What This Enables

### Passive Memory (automatic)
- Closed beans → episodic memory (what happened, with close reasons)
- Failed attempts → negative memory (what didn't work)
- Verification failures → staleness detection

### Active Memory (explicit)
- `bn fact` → verified semantic knowledge with proof (verify required)

### Retrieval
- `bn context` → auto-injected relevant memories at session start
- `bn recall` → search when needed (substring MVP, embeddings roadmap)

### Maintenance
- `bn verify --facts` → re-check all facts, detect staleness
- Dependency invalidation → suspect propagation through produces/requires graph

## Key Design Principles

1. **Facts ARE beans** — same commands, just `--type=fact`
2. **Facts require verification** — no verify = not a fact, use agents.md
3. **Verification unifies everything** — tasks verify completion, facts verify truth
4. **Passive over active** — context injected, not queried
5. **UNIX-friendly** — text output, pipeable, composable
6. **Graceful degradation** — works without embeddings, without global scope
7. **Self-healing** — stale facts surface automatically

## bn context: The Keystone

Returns relevant memories for RIGHT NOW. Designed for session-start injection.

**Sections (priority order):**
1. WARNINGS — stale facts, past failures (never truncated)
2. WORKING ON — claimed beans with attempt history
3. RELEVANT FACTS — scored by path overlap, dependencies
4. RECENT WORK — closed beans from last 7 days

**Relevance scoring (MVP, no embeddings):**
```
score = path_overlap × 3 + dependency_match × 5 + recency × 1
```

**Token budget:** ~4000 default, truncate from bottom (recent work first)

**Output format:**
```
═══ BEANS CONTEXT ═══════════════════════════════════════════

⚠ WARNINGS
│ STALE: "Session timeout is 24h" — not verified in 35d
│ PAST FAILURE [2.1]: "token not in httponly cookie"

► WORKING ON
│ [2.1] Implement JWT refresh tokens
│   Attempt #2 (previous: "httponly cookie issue")

✓ RELEVANT FACTS
│ "Auth uses RS256 signing" ✓ 1d ago

◷ RECENT WORK
│ [2.0] Auth module setup (closed 2d ago)
│   "Created JwtClaims, RS256 with env vars..."
```

## Attempt Tracking

When an agent claims a bean and fails, that's valuable knowledge.

- **Start**: `bn claim` starts an attempt
- **End**: `bn close` (success), `bn close --failed` (failure), `bn claim --release` (abandoned)
- Attempts stored as JSON array: `[{num, outcome, notes}]`
- `bn context` surfaces past failures to prevent repeating mistakes

## Staleness

- Default TTL: 30 days (configurable via `--ttl`)
- Facts not verified within TTL show as warnings in `bn context`
- `bn verify --facts` re-runs all fact verifications
- Suspect propagation: facts requiring invalid facts become suspect (depth limit 3)

## Schema Additions

```sql
ALTER TABLE beans ADD COLUMN bean_type TEXT DEFAULT 'task';
  -- 'task' | 'fact'

ALTER TABLE beans ADD COLUMN scope TEXT DEFAULT 'project';
  -- 'project' | 'global'

ALTER TABLE beans ADD COLUMN last_verified INTEGER;
  -- Unix timestamp of last successful verify

ALTER TABLE beans ADD COLUMN stale_after INTEGER;
  -- Unix timestamp = created + ttl (default 30 days)

ALTER TABLE beans ADD COLUMN paths TEXT;
  -- JSON array: ["src/auth.rs"] for relevance matching

ALTER TABLE beans ADD COLUMN attempts TEXT;
  -- JSON array: [{num, outcome, notes}]
```

## Roadmap (not MVP)

- **Embeddings**: Semantic search for `bn recall`, embedding-based relevance in `bn context`
- **Global scope**: ~/.beans/global.db for cross-project facts
- **Model choice**: TBD (voyage-code-3, voyage-2, local, etc.)

## Implementation Phases

### Phase 1: Foundation
- Schema migration (add columns)
- Attempt tracking (start/end/failed flow)

### Phase 2: Context
- `bn context` command (path + dependency relevance, no embeddings)
- Token budget and truncation

### Phase 3: Facts
- `bn fact` command (requires --verify)
- `bn verify --facts` (staleness detection, suspect propagation)

### Phase 4: Search
- `bn recall` (substring search MVP)
- Skill update (teach agents the workflow)

### Future
- Embedding-based semantic search
- Global scope
- Auto-injection in pi

## Files

- packages/beans/src/db.ts — schema migration
- packages/beans/src/commands/context.ts — new command
- packages/beans/src/commands/fact.ts — convenience wrapper  
- packages/beans/src/commands/recall.ts — search command
- packages/beans/src/commands/verify.ts — extend for --facts
- packages/beans/src/relevance.ts — scoring algorithm
- skills/beans/SKILL.md — update with memory workflow
