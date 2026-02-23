---
id: '76'
title: 'MCP server: expose beans as MCP tools'
slug: mcp-server-expose-beans-as-mcp-tools
status: open
priority: 2
created_at: 2026-02-23T12:13:30.494728Z
updated_at: 2026-02-23T12:13:30.494728Z
verify: cargo test mcp && bn mcp --help 2>&1 | grep -qi 'serve\|server\|mcp'
tokens: 5166
tokens_updated: 2026-02-23T12:13:30.498416Z
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
