---
id: '77'
title: Post-close and on-fail hooks
slug: post-close-and-on-fail-hooks
status: closed
priority: 2
created_at: '2026-02-23T12:13:50.588573Z'
updated_at: '2026-02-27T06:36:03.976894Z'
closed_at: '2026-02-27T06:36:03.976894Z'
verify: cargo test hooks && bn config set on_close 'echo done' 2>/dev/null; bn config get on_close 2>&1 | grep -q 'echo done'
claimed_by: pi-agent
claimed_at: '2026-02-27T06:16:48.565Z'
is_archived: true
tokens: 43055
tokens_updated: '2026-02-23T12:13:50.595521Z'
history:
- attempt: 1
  started_at: '2026-02-27T06:36:03.981082Z'
  finished_at: '2026-02-27T06:36:34.310479Z'
  duration_secs: 30.329
  result: pass
  exit_code: 0
outputs:
  text: |-
    running 36 tests
    test commands::trust::tests::test_cmd_trust_enables_hooks ... ok
    test commands::trust::tests::test_cmd_trust_revoke_disables_hooks ... ok
    test hooks::tests::test_create_trust_creates_trust_file ... ok
    test hooks::tests::test_execute_hook_respects_non_trusted_status ... ok
    test hooks::tests::test_execute_config_hook_failure_does_not_panic ... ok
    test hooks::tests::test_execute_hook_skips_when_not_trusted ... ok
    test hooks::tests::test_expand_template_empty_template ... ok
    test hooks::tests::test_expand_template_missing_vars_left_as_is ... ok
    test hooks::tests::test_expand_template_multiple_same_var ... ok
    test hooks::tests::test_expand_template_no_placeholders ... ok
    test hooks::tests::test_expand_template_output_truncated_to_1000_chars ... ok
    test hooks::tests::test_expand_template_with_all_vars ... ok
    test hooks::tests::test_hook_event_string_representation ... ok
    test hooks::tests::test_get_hook_path ... ok
    test hooks::tests::test_hook_payload_serializes_to_json ... ok
    test hooks::tests::test_hook_payload_with_all_bean_fields ... ok
    test hooks::tests::test_hook_payload_with_reason ... ok
    test hooks::tests::test_hook_receives_json_payload_on_stdin ... ok
    test commands::create::tests::untrusted_hooks_are_silently_skipped ... ok
    test hooks::tests::test_is_hook_executable_with_executable_file ... ok
    test hooks::tests::test_is_hook_executable_with_missing_file ... ok
    test hooks::tests::test_is_hook_executable_with_non_executable_file ... ok
    test hooks::tests::test_is_trusted_returns_false_when_trust_file_does_not_exist ... ok
    test hooks::tests::test_is_trusted_returns_true_when_trust_file_exists ... ok
    test hooks::tests::test_missing_hook_returns_ok_true ... ok
    test hooks::tests::test_non_executable_hook_returns_error ... ok
    test hooks::tests::test_revoke_trust_errors_if_file_does_not_exist ... ok
    test hooks::tests::test_revoke_trust_removes_trust_file ... ok
    test commands::close::tests::test_close_with_untrusted_hooks_silently_skips ... ok
    test hooks::tests::test_execute_config_hook_with_template_expansion ... ok
    test hooks::tests::test_execute_config_hook_writes_to_file ... ok
    test hooks::tests::test_execute_hook_runs_when_trusted ... ok
    test hooks::tests::test_hook_execution_with_failure_exit_code ... ok
    test hooks::tests::test_successful_hook_execution ... ok
    test commands::update::tests::test_update_with_multiple_fields_triggers_hooks ... ok
    test hooks::tests::test_hook_timeout ... ok

    test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 855 filtered out; finished in 30.11s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 38 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.00s

    Set on_close = echo done
---

## Task

Extend the existing hook system with post-close and on-fail lifecycle hooks, enabling CI/PR integration without baking specific integrations into beans.

## Motivation
Agent Orchestrator auto-creates PRs on completion and posts failure info to issues. Rather than building GitHub/Linear integrations directly, beans should expose lifecycle hooks that users wire up to whatever they use.

## What to implement

### New hook types:
1. `post-close` — runs after a bean successfully closes (verify passed). Receives bean ID, title, close reason.
2. `on-fail` — runs after a verify attempt fails. Receives bean ID, attempt number, failure output.
3. `post-plan` — runs after `bn plan` creates children. Receives parent ID, child IDs.

### Configuration:
```bash
# Project-level hooks via config
bn config set on_close 'gh pr create --title "{title}" --body "Closes bean {id}"'
bn config set on_fail 'echo "Bean {id} failed attempt {attempt}" >> failures.log'
bn config set post_plan 'echo "Planned {parent} into {children}"'

# Or via .beans/config.yaml
hooks:
  post_close: 'gh pr create --title "{title}"'
  on_fail: 'echo "failed {id}"'
  post_plan: 'echo "planned {parent}"'
```

### Template variables available in hooks:
- `{id}` — bean ID
- `{title}` — bean title
- `{status}` — new status
- `{attempt}` — attempt number (on-fail only)
- `{output}` — verify output, truncated to 1000 chars (on-fail only)
- `{parent}` — parent ID (post-plan only)
- `{children}` — comma-separated child IDs (post-plan only)
- `{branch}` — current git branch

### Behavior:
- Hooks run asynchronously (don't block the close/fail flow)
- Hook failures are logged but don't affect the bean operation
- Hooks inherit the current working directory
- Multiple hooks per event supported (array in config)

## Files
- src/hooks.rs (modify — add new hook types and template expansion)
- src/commands/close.rs (modify — call post-close hook after successful close)
- src/commands/close.rs (modify — call on-fail hook after verify failure)
- src/commands/plan.rs (modify — call post-plan hook)
- src/config.rs (modify — add hook config keys)
- tests/ (add hook integration tests)

## Context

### Existing hooks system (src/hooks.rs)
The pre-close hook system already exists with trust management. Extend this pattern for the new hook types.

## Edge Cases
- Hook command fails → log warning, continue (never block the operation)
- Missing template variables → leave placeholder as-is
- Very long verify output in {output} → truncate to 1000 chars
- No hooks configured → skip silently

## Acceptance
- [ ] post-close hook fires after successful `bn close`
- [ ] on-fail hook fires after failed verify attempt
- [ ] Template variables are expanded correctly
- [ ] Hook failures don't break bean operations
- [ ] cargo test hooks passes
