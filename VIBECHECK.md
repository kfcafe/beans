# 🩺 VIBECHECK — Code Health Report

> **Score: 61/100** 🟡 · Mode: Full · Last updated: Feb 28, 2026, 01:54 PM

> Snapshot: `eb089168-a680-45fa-88f9-93515752f3aa` · Run: `280a04ef`

## Score Breakdown

| Dimension | Score | Detail |
| --------- | ----- | ------ |
| Code Quality | 50 | 42 issues · 10 high, 20 med, 12 low |
| Test Coverage | 77 | 56/70 source files tested |
| Project Quality | 70 | 7/10 checks pass |

## 🔧 Tooling

- Static tools run: tokei, qlty check, qlty smells, semgrep, biome
- Static tools skipped: knip, madge, cargo-udeps
- Static issues: 0
- LLM findings: 42

## 🔴 High Priority (10)

- **`src/commands/close.rs:267`** `single-responsibility` — `cmd_close` is 464 lines and handles 10+ distinct responsibilities: verify execution, fail-first tracking, attempt/history recording, circuit-breaker evaluation, on_fail action dispatch (retry/escalate), worktree merge, bean archival, post-close hooks, on_close action dispatch, config hook execution, worktree cleanup, auto-close parent, and index rebuild. This makes the function extremely difficult to reason about, test in isolation, or safely modify.
  > *Extract into a module directory `commands/close/` with sub-functions: `run_verify()`, `record_failure()`, `check_circuit_breaker()`, `process_on_fail()`, `archive_bean()`, `fire_hooks()`, `try_auto_close_parent()`. The outer `cmd_close` becomes a thin orchestrator calling these steps. Bean 94 already tracks this.*
- **`src/mcp/tools.rs:458`** `architecture` — `handle_close_bean` reimplements close logic (verify execution, archival, auto-close parent) independently from `commands/close.rs`. Key behavioral differences: (1) MCP close doesn't record RunRecord history entries, (2) doesn't run pre-close/post-close hooks, (3) doesn't process on_fail/on_close actions, (4) doesn't handle worktree merge, (5) doesn't implement circuit breaker, (6) doesn't capture stdout as outputs. Users closing beans via MCP get silently different behavior.
  > *Extract shared close logic into `api/mod.rs` as a `close_bean()` function that both `cmd_close` and `handle_close_bean` call. The CLI layer adds stdout printing; the MCP layer returns JSON. Same pattern for claim and create.*
- **`src/mcp/tools.rs:840`** `edge-cases` — `truncate_str` slices a `&str` at byte offset `s.len() - max`, which will panic if that offset lands inside a multi-byte UTF-8 character. `String::from_utf8_lossy` produces replacement chars (U+FFFD = 3 bytes), so a slice at `s.len() - max` can hit mid-character and panic at runtime.
  > *Use `s.char_indices()` to find a char-aligned boundary: `let start = s.char_indices().rev().nth(max).map(|(i,_)| i).unwrap_or(0); format!("...{}", &s[start..])`. Or walk backward from the target byte offset with `s.is_char_boundary()` like `close.rs` already does.*
- **`src/main.rs:51`** `function-length` — `main()` is 660 lines — a single match statement that destructures CLI args and dispatches to command handlers. Every new command adds 10-30 lines here. The `Command::Create` branch alone is ~200 lines with inline logic for `--run`, `--json`, stdin resolution, and interactive mode. This is the highest-churn file (40 commits) and growing.
  > *Move the Create dispatch into `commands/create.rs` as a `dispatch_create()` function that takes the raw CLI args. Generalize: each command gets a `dispatch()` method that takes raw args and calls `cmd_*`. The main match becomes one-liners: `Command::Create { args } => commands::create::dispatch(&beans_dir, args)`.*
- **`src/commands/close.rs:267`** `function-length` — `cmd_close` is 458 lines doing at least 8 distinct responsibilities: input validation, pre-close hooks, verify execution, failure recording with history, circuit breaker logic, on_fail actions, worktree merge, archiving, on_close actions, config hooks, worktree cleanup, and parent auto-close. This makes it extremely hard to test individual steps in isolation or modify safely.
  > *Extract into focused functions: `verify_and_record()`, `process_on_fail()`, `archive_bean()`, `process_on_close_actions()`, `handle_parent_auto_close()`. The main `cmd_close` should be a thin orchestrator calling these steps sequentially.*
- **`src/hooks.rs:284`** `input-sanitization` — `expand_template()` performs naive string substitution of `{title}`, `{output}`, `{branch}`, and other variables into shell command templates without any shell escaping. These expanded strings are then passed directly to `sh -c` in `execute_config_hook()` (line 354). A bean title like `"; rm -rf / #` or verify output containing shell metacharacters will be interpolated directly into the command, enabling shell injection. The `{output}` variable is truncated to 1000 chars but not escaped. While `on_close`/`on_fail` templates are set by the project owner (config.yaml), the *values* substituted — especially `{title}` and `{output}` — come from bean files that agents or collaborators can create.
  > *Use `shell_escape` (the `shell-escape` crate is already in Cargo.toml) to escape all variable values before interpolating them into the template string. Alternatively, pass variables as environment variables to the subprocess instead of inlining them in the shell command string.*
- **`src/mcp/tools.rs:488`** `edge-cases` — In `handle_close_bean`, the output truncation `&combined[combined.len() - 2000..]` slices at a raw byte offset. If that offset falls inside a multi-byte UTF-8 character, this panics. The `combined` variable comes from `from_utf8_lossy` and may contain 3-byte replacement chars.
  > *Use `truncate_to_char_boundary` from `close.rs` (or a shared version) to find a safe boundary. For example: `let start = (combined.len() - 2000).max(0); let safe = start; while !combined.is_char_boundary(safe) { safe += 1; }; &combined[safe..]`*
- **`src/commands/plan.rs:192`** `input-validation` — `bean_path_str` (from `find_bean_file`) is interpolated directly into a shell command via `format!("pi @{} {}", bean_path_str, escaped_prompt)`. The prompt is shell-escaped but the file path is NOT. If the `.beans/` directory path contains spaces, backticks, `$`, or other shell metacharacters, this creates a command injection vector.
  > *Apply the same `shell_escape()` to `bean_path_str`: `format!("pi @{} {}", shell_escape(&bean_path_str), escaped_prompt)`.*
- **`src/main.rs:196`** `consistency` — The `--run` agent spawning logic is copy-pasted between the `Create::Next` handler (lines 196-227) and the normal `Create` handler (lines 316-351). Both blocks load config, check `config.run`, substitute `{id}`, spawn via `sh -c`, and handle errors identically. This violates DRY — any bug fix or behavior change must be applied in two places.
  > *Extract a `spawn_run_agent(beans_dir: &Path, bean_id: &str) -> Result<()>` helper and call it from both locations. The function already exists conceptually in `spawner.rs` as `substitute_template` + spawn.*
- **`src/spawner.rs:287`** `architecture` — `claim_bean()` and `release_bean()` shell out to `bn claim` and `bn claim --release` as subprocesses — the CLI calling its own binary. This is fragile (depends on `bn` being in PATH), adds process overhead, and fails when running via `cargo run` during development. The functions `cmd_claim` and `cmd_release` already exist in `src/commands/claim.rs`.
  > *Call `cmd_claim()` and `cmd_release()` directly as library functions instead of shelling out. Import from `crate::commands::claim`.*

## 🟡 Medium Priority (20)

- **`src/mcp/tools.rs:420`** `architecture` — `handle_claim_bean` reimplements claim logic without running the fail-first verify check that `commands/claim.rs` performs. CLI claim runs the verify command to prove it fails (checkpoint), while MCP claim just sets the status. This means agents claiming via MCP skip the TDD safety net — the project's core differentiating feature.
  > *Share the claim logic through `api/mod.rs` or call through `cmd_claim` with stdout suppression.*
- **`src/main.rs:194`** `consistency` — The `--run` post-create spawn logic is duplicated twice in main.rs — once for `Command::Create` (line ~297-333) and once for `CreateSubcommand::Next` (line ~194-238). Both implement identical template resolution, command spawning, and error handling. If one is updated, the other will diverge.
  > *Extract into a shared helper `spawn_run_command(beans_dir, bean_id, config)` called from both paths, or consolidate through cmd_create_next.*
- **`src/bean.rs:151`** `single-responsibility` — The `Bean` struct has 35+ fields spanning 4 different concerns: core identity (id, title, status), verification lifecycle (verify, attempts, history, checkpoint, fail_first), memory/fact system (bean_type, last_verified, stale_after, paths, attempt_log), and orchestration metadata (tokens, tokens_updated, max_loops, outputs). Adding a field to any concern requires touching this 1631-line file.
  > *Group related fields into sub-structs: `VerifyState { verify, fail_first, checkpoint, attempts, max_attempts, history }`, `MemoryMeta { bean_type, last_verified, stale_after, paths, attempt_log }`. The Bean struct composes these. Bean 95 tracks splitting bean.rs.*
- **`src/commands/close.rs:41`** `dead-code` — `VerifyResult.stderr` is captured but never read anywhere — annotated with `#[allow(dead_code)]`. The `output` field already contains combined stdout+stderr, making `stderr` redundant. Dead fields add cognitive overhead and suggest unfinished design.
  > *Remove the `stderr` field and the `#[allow(dead_code)]` annotation. If stderr-only access is needed later, it can be re-added.*
- **`src/commands/verify.rs:1`** `test-coverage` — `cmd_verify` has zero tests. It runs shell commands and is a critical path — agents call `bn verify` to check their work before closing. Edge cases not covered: empty verify command, bean not found, project_root resolution failure, verify with non-UTF8 output.
  > *Add unit tests similar to `close.rs` test patterns using tempdir and known-passing/failing shell commands.*
- **`src/commands/status.rs:1`** `test-coverage` — `cmd_status` has zero tests. This is the primary dashboard command users run to understand project state. It performs filtering and grouping logic (claimed/ready/goals/blocked) that should be verified.
  > *Add tests that create beans in various states and verify the status output categorization is correct.*
- **`src/mcp/tools.rs:363`** `consistency` — MCP's `handle_create_bean` uses `Config::load()` (no inheritance) while the CLI's `cmd_create` also uses `Config::load()`. However, `cmd_run` uses `Config::load_with_extends()`. This inconsistency means beans created via MCP or CLI won't respect inherited config values (like `max_tokens` from a parent config), while `bn run` will.
  > *Use `Config::load_with_extends()` consistently in all code paths that read config. The `load()` method should probably be made private, with `load_with_extends()` as the default public API.*
- **`src/commands/run/ready_queue.rs:73`** `comprehensibility` — `run_ready_queue_direct` is 167 lines with deeply nested control flow: a loop containing a filter, sort, spawn loop, capacity check, channel recv, success/failure handling, and keep_going logic. The threading model (spawn threads that send results back via mpsc) is interleaved with bean-state management, making it hard to follow which operations are thread-safe.
  > *Extract the inner loop body into named functions: `find_and_spawn_ready()`, `handle_completion()`. Consider a small state struct to avoid passing 8+ variables between functions.*
- **`src/mcp/tools.rs:720`** `architecture` — `all_children_closed` and `auto_close_parent` in `mcp/tools.rs` are separate implementations from the identically-named functions in `commands/close.rs`. The MCP version has subtly different semantics: it returns `false` when a parent has no children (close.rs returns `true`), it doesn't handle archived beans, and it doesn't recurse to grandparents.
  > *Move the canonical `all_children_closed` and `auto_close_parent` into a shared module (e.g., `bean_lifecycle.rs` or `api/mod.rs`). Both CLI and MCP call the same implementation.*
- **`src/spawner.rs:77`** `input-sanitization` — `substitute_template()` replaces `{id}` in the run/plan command template with the raw bean ID and passes the result to `sh -c`. Although `validate_bean_id()` restricts IDs to alphanumeric, dots, underscores, and hyphens (making injection difficult today), this function performs no escaping and relies entirely on the validator never being relaxed. If the ID validation is ever broadened, this becomes a direct shell injection vector. Defense in depth is missing.
  > *Apply shell escaping to the bean ID before substituting it into the template, or pass the bean ID as an environment variable (e.g., `BEAN_ID`) instead of inlining it.*
- **`src/config.rs:190`** `input-validation` — `resolve_extends_path()` allows `~/` expansion and relative paths but does not validate that the resolved path stays within any trusted directory. An `extends` entry like `../../etc/shadow` or `~/../../etc/passwd` would be resolved and read as a YAML config file. While `extends` is set by the project owner, malicious `.beans/config.yaml` files in cloned repositories could point to sensitive files. The path is canonicalized later but only after the file content has been read and parsed.
  > *After resolving the path, canonicalize it and verify it stays within the project root or home directory. Reject paths that escape these boundaries.*
- **`src/commands/close.rs:647`** `input-sanitization` — `on_close` `Run` actions execute commands from bean YAML files directly via `sh -c`. While this is gated behind the trust mechanism (`is_trusted()`), the commands themselves come from `.beans/` files which are committed to git. A contributor could submit a PR with a bean file containing a malicious `on_close` action. The trust flag is a single file (`.beans/.hooks-trusted`) that once enabled stays enabled for all beans and all on_close commands without per-command review.
  > *Consider displaying on_close commands before executing them (similar to how verify commands are printed), or implement per-bean trust. At minimum, log a warning that shows the exact command being executed when trust is first enabled. Consider requiring explicit trust per on_close command rather than a global flag.*
- **`src/hooks.rs:346`** `input-sanitization` — `execute_config_hook()` spawns the expanded command via `sh -c` in fire-and-forget mode (no wait, no exit code check, no output capture). If the shell command fails silently or if the template expansion produces a malformed command due to unescaped variables, there's no way to detect it. Combined with the `expand_template()` injection issue, this is doubly risky because errors from injected commands are invisible.
  > *At minimum, wait for the process and log its exit code. Use the `shell-escape` crate (already a dependency) to sanitize interpolated variables.*
- **`src/mcp/server.rs:30`** `input-validation` — The MCP server reads JSON-RPC messages from stdin with no per-message size limit. A malicious MCP client could send an extremely large JSON payload to consume memory. The server also has no authentication — any process that can write to stdin can invoke all tools (create, close, verify, etc.). While MCP's security model typically relies on the host process, the lack of any rate limiting or size bounds could lead to denial of service.
  > *Add a maximum message size limit (e.g., 1MB) when reading lines from stdin. Consider adding optional API key authentication for production MCP deployments.*
- **`src/mcp/tools.rs:458`** `input-validation` — `handle_close_bean()` in the MCP server does not execute pre-close hooks or on_close actions — it directly closes and archives the bean. This bypasses the security controls that the CLI `cmd_close()` enforces (pre-close hooks, trust checks for on_close, on_fail config hooks, worktree handling). This inconsistency means the MCP path has fewer protections, and could also skip important business logic hooks.
  > *Factor the close logic into a shared function used by both the CLI and MCP paths, or explicitly call the same hook/action pipeline in the MCP handler.*
- **`src/commands/run/mod.rs:63`** `architecture` — `BeanAction` in `run/mod.rs` and `AgentAction` in `spawner.rs` are identical enums (`Implement`, `Plan`) serving the same purpose. Having two types for the same concept creates confusion about which to use and requires conversion between them.
  > *Keep one enum (suggest `BeanAction` in a shared location or `run/mod.rs`) and delete the duplicate. Update `spawner.rs` to import and use the canonical type.*
- **`src/bean.rs:82`** `comprehensibility` — The section header comment says `// OnCloseAction` but the code block that follows is `enum OnFailAction`. The `OnCloseAction` enum is defined at line 110. This mismatch is confusing when scanning the file — the section header lies about what's defined below it.
  > *Change the section header at line 82-84 to `// OnFailAction` or separate the two enums with distinct section headers: `// OnFailAction` then `// OnCloseAction`.*
- **`src/commands/close.rs:298`** `error-handling` — Pre-close hook execution errors (not executable, timeout, I/O failure) are silently treated as "pass" (`true`), allowing the close to proceed. A hook that can't run is semantically different from a hook that approves — if a security-critical hook times out or crashes, the bean is closed without validation.
  > *Make hook error handling configurable. Add a `strict_hooks` config option that treats hook execution errors as failures in security-sensitive deployments. At minimum, log at a clear warning level (not just stderr eprintln) distinguishing "hook approved" from "hook failed to run."*
- **`src/commands/close.rs:539`** `edge-cases` — When `bean.to_file()` succeeds but `fs::rename()` (archive) fails, the bean is marked as closed in its original location but never archived. The subsequent index rebuild will show it as closed-but-not-archived. There's no rollback of the status change, leaving the system in an inconsistent state.
  > *Either: (1) archive first (rename), then update metadata in the new location, or (2) catch the rename error and revert the bean status to its previous state before returning the error.*
- **`src/spawner.rs:142`** `resource-cleanup` — When `Spawner::spawn()` successfully starts a child process but `register_agent` fails (e.g., I/O error writing the agents file), the child process runs untracked. The claim was already acquired, so the bean stays claimed with no way to monitor or release it.
  > *Wrap post-spawn registration in a cleanup guard: if `register_agent` or the HashMap insert fails, kill the child process and release the bean claim before returning the error.*

## 🟢 Low Priority (12)

- **`src/bean.rs:487`** `error-handling` — `Bean::hash()` calls `serde_json::to_string(&canonical).unwrap()` — the only `unwrap()` in non-test production code. While serde_json serialization of a Bean is unlikely to fail (no maps with non-string keys), this breaks the project's convention of using `anyhow::Result` everywhere.
  > *Change to `serde_json::to_string(&canonical)?` and make `hash()` return `Result<String>`, or use `.expect("Bean serialization to JSON should never fail")` to document the invariant.*
- **`src/commands/run/mod.rs:484`** `dead-code` — `pub fn find_bean_file(beans_dir, id)` is a public wrapper around `crate::discovery::find_bean_file` that is never called from anywhere outside this module. It re-exports existing functionality without adding value.
  > *Remove the function. All callers already use `crate::discovery::find_bean_file` directly.*
- **`src/commands/close.rs:121`** `performance` — `all_children_closed` calls `Index::build()` (full disk scan + parse of all bean files) every time, even when called in a loop for recursive auto-close. With many beans, this is O(n) disk I/O per parent in the chain. For a 3-level deep hierarchy, this rebuilds the index 3 times.
  > *Accept an `&Index` parameter instead of rebuilding. The caller (`cmd_close`) already has a fresh index or can build one once. Pass it through to `auto_close_parent` → `all_children_closed`.*
- **`src/commands/close.rs:25`** `consistency` — `truncate_to_char_boundary` is a general-purpose UTF-8 utility function buried inside `close.rs`. It's also needed by `mcp/tools.rs` (which has a buggy version). Other modules that handle string truncation (e.g., `pi_output.rs`, `hooks.rs`) could benefit from it.
  > *Move to `util.rs` and make it `pub`. Replace the unsafe version in `mcp/tools.rs`.*
- **`src/config.rs:1`** `consistency` — Test fixtures throughout the codebase (close.rs, create.rs, run/mod.rs, spawner.rs, etc.) construct `Config` structs manually with all fields, and each added field requires updating 30+ test sites. The `on_close`, `on_fail`, `post_plan` fields have inconsistent indentation (spaces vs struct-body alignment) in these constructors, suggesting copy-paste maintenance.
  > *Implement `Default` for `Config` (or a `Config::test_default()`) so test fixtures can use `Config { project: "test".into(), ..Config::default() }`. This eliminates the need to update every test when a new config field is added.*
- **`src/bean.rs:418`** `input-validation` — `from_string()` deserializes arbitrary YAML from bean files with no size limit or schema validation beyond struct field matching. serde_yml will accept any valid YAML, including YAML bombs (deeply nested structures, long strings, or alias-based amplification) that could cause excessive memory allocation. While beans are typically small, a malicious bean file could be crafted to consume excessive memory during parsing.
  > *Add a file size check before parsing (e.g., reject files >1MB). Consider using `serde_yml`'s deserialization with a size-limited reader.*
- **`src/commands/edit.rs:174`** `input-validation` — `open_editor()` reads `$EDITOR` from the environment and passes a file path to it without validating the editor binary. While this is standard Unix convention, if `$EDITOR` is set to a malicious value (e.g., via a `.env` file or compromised environment), it executes arbitrary commands. The path argument is also not validated beyond UTF-8 checking.
  > *This is an accepted Unix convention, but consider logging a warning if the editor binary doesn't match common editors, or at minimum document the trust assumption.*
- **`src/ctx_assembler.rs:175`** `edge-cases` — `assemble_context()` correctly uses `canonicalize()` to prevent path traversal via symlinks, but it reads file contents into memory without a size limit. A bean description could reference very large files (e.g., binary blobs or multi-GB logs), causing OOM. The `read_file` function may have limits, but the aggregation of many files could still exhaust memory.
  > *Add a per-file size limit (e.g., 1MB) and a total context size budget to `assemble_context()`. Skip files exceeding the limit with a warning.*
- **`src/commands/tidy.rs:48`** `input-sanitization` — `pgrep_running()` passes user-influenced patterns (derived from the `run` config template) directly to `pgrep -f`. While `pgrep` doesn't execute its argument as a shell command, regex metacharacters in the run template could cause `pgrep` to match unintended processes, leading to `tidy` incorrectly determining agents are running and skipping cleanup.
  > *Escape regex metacharacters in the pattern before passing to `pgrep`, or use a different mechanism to detect running agents (e.g., PID files, which the spawner already creates).*
- **`src/commands/logs.rs:6`** `resource-cleanup` — `log_dir()` falls back to `/tmp/beans/logs` when `dirs::data_local_dir()` returns `None`. On shared systems, `/tmp` is world-readable. Agent logs may contain bean descriptions, verify command output, and file paths — potentially sensitive context. The directory is created with default permissions (likely 0755).
  > *When falling back to `/tmp`, create the directory with restrictive permissions (0o700) using `std::os::unix::fs::DirBuilder` with mode, or use a user-specific subdirectory like `/tmp/beans-{uid}/logs`.*
- **`src/api/mod.rs:95`** `dead-code` — Three commented-out submodule declarations (`pub mod query`, `pub mod mutations`, `pub mod orchestration`) reference phase numbers that don't correspond to any existing beans or tracking. Commented-out code acts as a stale TODO that never gets actioned.
  > *Either implement the planned submodules or remove the comments. If this is planned work, create beans to track it rather than leaving code comments as a roadmap.*
- **`src/bean.rs:150`** `architecture` — The `Bean` struct has 35+ fields spanning 6 distinct domains: core metadata, verification state, claim management, memory system, dependency tracking, and hooks/actions. This is a god struct where every feature adds fields to a single type, making it increasingly hard to understand which fields interact.
  > *Group related fields into sub-structs: `VerifyState { verify, fail_first, checkpoint, attempts, max_attempts, history }`, `ClaimState { claimed_by, claimed_at }`, `MemoryState { bean_type, last_verified, stale_after, attempt_log }`. Use `#[serde(flatten)]` to preserve flat YAML serialization while improving code navigability.*

## ✅ Project Checklist

| Check | Status |
| ----- | ------ |
| README | ✅ |
| Onboarding scripts | ✅ |
| CI config | ✅ |
| Linter config | ✅ |
| .gitignore | ✅ |
| Dependencies declared | ✅ |
| Architecture docs | ✅ |
| .env.example | ❌ |
| Observability | ❌ |
| Environment parity | ❌ |

## 📈 History

| # | Date | Score | Δ | Issues |
| - | ---- | ----- | - | ------ |
| 1 | Feb 26 | 86 | — | 0 |
| 2 | Feb 26 | 86 | — | 0 |
| 3 | Feb 26 | 86 | — | 0 |
| 4 | Feb 26 | 86 | — | 0 |
| 5 | Feb 26 | 86 | — | 0 |
| 6 | Feb 26 | 86 | — | 0 |
| 7 | Feb 27 | 61 | -25 ▼ | 50 |
| 8 | Feb 27 | 61 | — | 23 |
| 9 | Feb 28 | 86 | +25 ▲ | 0 |
| 10 | Feb 28 | 61 | -25 ▼ | 42 ← now |

---

*Generated by vibecheck · Run `/vibecheck` or `/vibecheck update` to refresh*