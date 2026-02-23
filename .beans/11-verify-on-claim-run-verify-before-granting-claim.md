---
id: '11'
title: 'Verify-on-claim: run verify before granting claim'
slug: verify-on-claim-run-verify-before-granting-claim
status: open
priority: 2
created_at: 2026-02-03T03:21:42.823080Z
updated_at: 2026-02-23T23:35:43.635274Z
verify: cargo test claim::tests::verify_on_claim
---

## Summary
When claiming a bean with a verify command, run verify FIRST.
- PASSES → reject claim (nothing to do or test is bogus)  
- FAILS → grant claim, record checkpoint, set fail_first: true

## Why
Enforces TDD automatically. Checkpoint proves test was meaningful.

## Files
- src/commands/claim.rs
- src/bean.rs (add checkpoint field)

## Acceptance
- bn claim with passing verify rejected
- bn claim with failing verify succeeds
- bn claim --force overrides
- checkpoint SHA stored in bean
