---
id: '93'
title: 'refactor: Split commands/run.rs into a module directory'
slug: refactor-split-commandsrunrs-into-a-module-directo
status: closed
priority: 2
created_at: 2026-02-27T05:22:03.364632Z
updated_at: 2026-02-27T05:36:25.999609Z
closed_at: 2026-02-27T05:36:25.999609Z
verify: cargo test --lib commands::run
is_archived: true
tokens: 19174
tokens_updated: 2026-02-27T05:22:03.368805Z
history:
- attempt: 1
  started_at: 2026-02-27T05:36:26.000178Z
  finished_at: 2026-02-27T05:36:28.613017Z
  duration_secs: 2.612
  result: pass
  exit_code: 0
outputs:
  text: |-
    running 31 tests
    test commands::run::plan::tests::plan_dispatch_no_ready_beans ... ok
    test commands::run::ready_queue::tests::assemble_bean_context_includes_rules ... ok
    test commands::run::plan::tests::plan_dispatch_filters_by_id ... ok
    test commands::run::ready_queue::tests::is_bean_ready_dep_outside_set_treated_as_met ... ok
    test commands::run::ready_queue::tests::is_bean_ready_diamond_both_deps_needed ... ok
    test commands::run::ready_queue::tests::is_bean_ready_explicit_dep_met ... ok
    test commands::run::ready_queue::tests::is_bean_ready_explicit_dep_not_met ... ok
    test commands::run::ready_queue::tests::is_bean_ready_no_deps ... ok
    test commands::run::ready_queue::tests::is_bean_ready_requires_met ... ok
    test commands::run::ready_queue::tests::is_bean_ready_requires_not_met ... ok
    test commands::run::ready_queue::tests::assemble_bean_context_returns_empty_for_missing_bean ... ok
    test commands::run::tests::agent_result_tracks_tokens_and_cost ... ok
    test commands::run::ready_queue::tests::ready_queue_starts_independent_beans_immediately ... ok
    test commands::run::tests::determine_spawn_mode_direct_when_no_run ... ok
    test commands::run::tests::determine_spawn_mode_template_when_run_set ... ok
    test commands::run::plan::tests::auto_plan_includes_large_beans_in_waves ... ok
    test commands::run::plan::tests::large_bean_classified_as_plan ... ok
    test commands::run::tests::format_duration_formats_correctly ... ok
    test commands::run::wave::tests::compute_waves_diamond ... ok
    test commands::run::plan::tests::plan_dispatch_returns_ready_beans ... ok
    test commands::run::wave::tests::compute_waves_linear_chain ... ok
    test commands::run::wave::tests::compute_waves_no_deps ... ok
    test commands::run::tests::dry_run_with_json_stream ... ok
    test commands::run::wave::tests::template_wave_plan_without_template_errors ... ok
    test commands::run::plan::tests::plan_dispatch_parent_id_gets_children ... ok
    1
    test commands::run::plan::tests::dry_run_simulate_respects_produces_requires ... ok
    test commands::run::tests::dry_run_does_not_spawn ... ok
    test commands::run::plan::tests::dry_run_simulate_shows_all_waves ... ok
    test commands::run::wave::tests::template_wave_failed_command ... ok
    test commands::run::wave::tests::template_wave_execution_with_echo ... ok
    test commands::run::tests::cmd_run_errors_when_no_run_template_and_no_pi ... ok

    test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 822 filtered out; finished in 2.54s
---

## Task
Split the monolithic `src/commands/run.rs` (2,216 lines) into a module directory with focused files.

## Target Structure
- `src/commands/run/mod.rs` — Top-level orchestration: `RunArgs`, `BeanAction`, `cmd_run`, `run_once`, `run_loop`, `determine_spawn_mode`, `pi_available`, `SpawnMode`, `AgentResult`, `format_duration`, `find_bean_file`
- `src/commands/run/plan.rs` — Dispatch planning: `DispatchPlan`, `SizedBean`, `plan_dispatch`, `print_plan`, `print_plan_json`
- `src/commands/run/wave.rs` — Wave scheduling and execution: `Wave`, `compute_waves`, `run_wave`, `run_wave_template`, `run_wave_direct`
- `src/commands/run/ready_queue.rs` — Ready-queue dispatch: `run_ready_queue_direct`, `run_single_direct`, `is_bean_ready`, `all_deps_closed`, `assemble_bean_context`

## Steps
1. Create `src/commands/run/` directory
2. Move `src/commands/run.rs` to `src/commands/run/mod.rs` (preserves git blame)
3. Extract plan, wave, and ready_queue functions into their respective files
4. Add `mod plan; mod wave; mod ready_queue;` to mod.rs
5. Re-export all public items from mod.rs so `commands::run::RunArgs`, `commands::run::cmd_run`, etc. still work — callers must not change
6. Move tests to their relevant files (tests about waves go in wave.rs, etc.)

## External API (must not break)
These are used outside the module:
- `commands::run::RunArgs` (used in main.rs)
- `commands::run::cmd_run` (re-exported via commands/mod.rs)
- `commands::run::find_bean_file` (used in other command files)

## Constraints
- Do NOT change any function signatures or behavior
- Do NOT modify files outside src/commands/run/
- Re-export everything public from mod.rs so no external imports break
- `commands/mod.rs` already has `pub mod run;` — this works for both file and directory modules
