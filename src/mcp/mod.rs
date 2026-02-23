//! MCP (Model Context Protocol) server for beans.
//!
//! Exposes beans operations as MCP tools over stdio transport,
//! enabling integration with Cursor, Windsurf, Claude Desktop, Cline,
//! and any MCP-compatible client.

pub mod protocol;
pub mod resources;
pub mod server;
pub mod tools;
