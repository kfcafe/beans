//! MCP server command: `bn mcp serve`

use std::path::Path;

use anyhow::Result;

/// Start the MCP server on stdio.
///
/// Reads JSON-RPC 2.0 messages from stdin, dispatches to beans operations,
/// and writes responses to stdout. Designed for use with MCP clients like
/// Cursor, Windsurf, Claude Desktop, and Cline.
pub fn cmd_mcp_serve(beans_dir: &Path) -> Result<()> {
    crate::mcp::server::run(beans_dir)
}
