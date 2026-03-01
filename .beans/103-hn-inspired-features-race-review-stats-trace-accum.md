---
id: '103'
title: 'HN-inspired features: race, review, stats, trace, accumulated knowledge'
slug: hn-inspired-features-race-review-stats-trace-accum
status: open
priority: 1
created_at: '2026-02-28T21:42:21.803808Z'
updated_at: '2026-02-28T21:42:21.803808Z'
labels:
- feature
- hn-inspired
verify: cargo test && cargo clippy --all-targets -- -D warnings
tokens: 220
tokens_updated: '2026-02-28T21:42:21.823212Z'
---

## Overview

Features inspired by the HN discussion on Verified Spec-Driven Development (VSDD).
These fill gaps between beans' mechanical verification and semantic correctness.

## Features

1. **Accumulated Knowledge Primitive** — attempt_log notes auto-injected into agent context on every dispatch
2. **Race Mode** — `bn race` to dispatch N agents, wait for all, human picks best
3. **Review** — `bn run --review` and `bn review <id>` for adversarial review post-close
4. **Cost Dashboard** — enhance `bn stats` with token/cost/pass-rate aggregation
5. **Verify Timeout** — kill verify command if it exceeds a time limit
6. **Trace Command** — `bn trace <id>` walks the bean graph showing full lineage
7. **Codebase Context Enrichment** — enrich `bn context` with AST structure from bean paths
