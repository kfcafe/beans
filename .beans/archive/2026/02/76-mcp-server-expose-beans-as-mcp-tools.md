---
id: '76'
title: 'MCP server: expose beans as MCP tools'
slug: mcp-server-expose-beans-as-mcp-tools
status: closed
priority: 2
created_at: 2026-02-23T12:13:30.494728Z
updated_at: 2026-02-27T06:24:13.654389Z
closed_at: 2026-02-27T06:24:13.654389Z
verify: cargo test mcp && bn mcp --help 2>&1 | grep -qi 'serve\|server\|mcp'
claimed_by: pi-agent
claimed_at: 2026-02-27T06:16:04.959701Z
is_archived: true
tokens: 5166
tokens_updated: 2026-02-23T12:13:30.498416Z
history:
- attempt: 1
  started_at: 2026-02-27T06:24:13.657060Z
  finished_at: 2026-02-27T06:24:13.821213Z
  duration_secs: 0.164
  result: pass
  exit_code: 0
outputs:
  text: |-
    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 891 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s


    running 38 tests
    test mcp_create_bean_missing_title_returns_error ... ok
    test mcp_claim_bean_already_claimed_returns_error ... ok
    test mcp_claim_bean_sets_in_progress ... ok
    test mcp_context_bean_no_paths ... ok
    test mcp_json_rpc_request_deserializes ... ok
    test mcp_json_rpc_request_without_id_is_notification ... ok
    test mcp_json_rpc_response_error_serializes ... ok
    test mcp_json_rpc_response_success_serializes ... ok
    test mcp_error_result_has_is_error_flag ... ok
    test mcp_close_bean_force_skips_verify ... ok
    test mcp_list_beans_filter_by_priority ... ok
    test mcp_required_tools_have_required_params ... ok
    test mcp_resource_definitions_present ... ok
    test mcp_list_beans_returns_all_open ... ok
    test mcp_ready_beans_excludes_blocked ... ok
    test mcp_create_bean_basic ... ok
    test mcp_create_bean_with_priority ... ok
    test mcp_resource_read_bean ... ok
    test mcp_resource_read_rules_missing ... ok
    test mcp_server_dispatch_initialize ... ok
    test mcp_resource_read_rules_present ... ok
    test mcp_resource_read_status ... ok
    test mcp_resource_read_unknown_uri_returns_error ... ok
    test mcp_show_bean_missing_id_returns_error ... ok
    test mcp_close_bean_with_passing_verify ... ok
    test mcp_tool_definitions_have_valid_json_schemas ... ok
    test mcp_show_bean_invalid_id_returns_error ... ok
    test mcp_tool_definitions_returns_all_ten_tools ... ok
    test mcp_show_bean_returns_full_details ... ok
    test mcp_verify_bean_no_verify_command ... ok
    test mcp_tree_shows_all_beans ... ok
    test mcp_close_bean_with_failing_verify_returns_error ... ok
    test mcp_tool_call_result_format_matches_spec ... ok
    test mcp_unknown_tool_returns_error ... ok
    test mcp_status_overview ... ok
    test mcp_tree_with_parent_child ... ok
    test mcp_create_then_close_roundtrip ... ok
    test mcp_verify_bean_passing ... ok

    test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s


    running 0 tests

    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.00s
---

## Task

Add an MCP (Model Context Protocol) server mode to beans so it can be used from Cursor, Windsurf, Claude Desktop, Cline, and any MCP-compatible client.

## Motivation
Shrimp Task Manager has ~2,000 GitHub stars largely because it's an MCP server — it works with every MCP client out of the box. This is the single biggest reach multiplier for beans. Most AI coding tool users are in IDEs (Cursor, Windsurf) not terminals.

## What to implement

### New command: `bn mcp serve`
Starts an MCP server (stdio transport, per the MCP spec) that exposes beans operations as tools.

### MCP Tools to expose:
1. `list_beans` — list beans with optional status/priority filters
2. `show_bean` — get full bean details (the prompt)
3. `ready_beans` — get beans ready to work on
4. `create_bean` — create a new bean (title, verify, description, parent, priority)
5. `claim_bean` — claim a bean for work
6. `close_bean` — close a bean (runs verify gate)
7. `verify_bean` — run verify without closing
8. `context_bean` — get assembled context for a bean
9. `status` — project status overview
10. `tree` — hierarchical bean tree

### MCP Resources to expose:
- `beans://status` — current project status
- `beans://rules` — project rules (if RULES.md exists)
- `beans://bean/{id}` — individual bean as a resource

### Transport
- stdio (primary, required by most MCP clients)
- Optional: SSE for web-based clients

## Files
- src/commands/mcp.rs (create — new MCP server command)
- src/mcp/ (create — MCP protocol handling module)
  - src/mcp/server.rs — server lifecycle
  - src/mcp/tools.rs — tool definitions and handlers
  - src/mcp/resources.rs — resource definitions
  - src/mcp/transport.rs — stdio transport
- src/cli.rs (modify — add mcp subcommand)
- Cargo.toml (modify — add serde_json, tokio deps if not present)
- tests/mcp_test.rs (create)

## MCP Config Example (for users)
```json
{
  "mcpServers": {
    "beans": {
      "command": "bn",
      "args": ["mcp", "serve"],
      "cwd": "/path/to/project"
    }
  }
}
```

## Edge Cases
- No .beans/ directory → return error tool result, don't crash
- Concurrent access (MCP client + CLI user) → file locking or accept-and-retry
- Large bean descriptions → respect MCP message size limits
- Invalid bean IDs → proper MCP error responses

## Acceptance
- [ ] `bn mcp serve` starts and responds to MCP initialize handshake
- [ ] All 10 tools callable from an MCP client
- [ ] create_bean + close_bean roundtrip works
- [ ] cargo test mcp passes
- [ ] Works with at least one real MCP client (Claude Desktop or Cursor)
