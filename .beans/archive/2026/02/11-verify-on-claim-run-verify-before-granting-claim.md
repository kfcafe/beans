---
id: '11'
title: 'Verify-on-claim: run verify before granting claim'
slug: verify-on-claim-run-verify-before-granting-claim
status: closed
priority: 2
created_at: 2026-02-03T03:21:42.823080Z
updated_at: 2026-02-27T06:11:31.940278Z
closed_at: 2026-02-27T06:11:31.940278Z
verify: cargo test claim::tests::verify_on_claim
claimed_by: pi-agent
claimed_at: 2026-02-27T06:04:55.821720Z
is_archived: true
history:
- attempt: 1
  started_at: 2026-02-27T06:11:31.942386Z
  finished_at: 2026-02-27T06:11:32.145689Z
  duration_secs: 0.203
  result: pass
  exit_code: 0
outputs:
  text: |-
    running 5 tests
    test commands::claim::tests::verify_on_claim_force_overrides ... ok
    test commands::claim::tests::verify_on_claim_no_verify_skips_check ... ok
    test commands::claim::tests::verify_on_claim_passing_verify_rejected ... ok
    test commands::claim::tests::verify_on_claim_failing_verify_succeeds ... ok
    test commands::claim::tests::verify_on_claim_checkpoint_sha_stored ... ok

    test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 860 filtered out; finished in 0.08s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.00s
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
