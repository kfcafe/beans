//! # Beans Library API
//!
//! Programmatic access to beans operations. Use this module when embedding beans
//! in another application (e.g., a GUI, MCP server, or custom tooling).
//!
//! The API is organized into layers:
//!
//! - **Types** — Core data structures (`Bean`, `Index`, `Status`, etc.)
//! - **Discovery** — Find `.beans/` directories and bean files
//! - **Query** — Read-only operations (list, get, tree, status, graph)
//! - **Mutations** — Write operations (create, update, close, delete)
//! - **Orchestration** — Agent dispatch, monitoring, and control
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use bn::api::*;
//!
//! // Find the .beans/ directory
//! let beans_dir = find_beans_dir(std::path::Path::new(".")).unwrap();
//!
//! // Load the index (cached, rebuilds if stale)
//! let index = Index::load_or_rebuild(&beans_dir).unwrap();
//!
//! // Get a specific bean
//! let bean = get_bean(&beans_dir, "1").unwrap();
//! println!("{}: {}", bean.id, bean.title);
//! ```
//!
//! ## Design Principles
//!
//! - **No I/O side effects** — Library functions never print to stdout/stderr.
//!   All output is returned as structured data.
//! - **Structured params and results** — Each operation takes a `Params` struct
//!   and returns a `Result` type. No raw CLI argument passing.
//! - **Serializable** — All types derive `Serialize`/`Deserialize` for easy
//!   IPC (Tauri, JSON-RPC, MCP).
//! - **Composable** — Functions take `&Path` (beans_dir) and return owned data.
//!   No global state, no singletons.

use std::path::Path;

use anyhow::Result;

// ---------------------------------------------------------------------------
// Re-exported core types
// ---------------------------------------------------------------------------

// Bean and related types
pub use crate::bean::{
    AttemptOutcome, AttemptRecord, Bean, OnCloseAction, OnFailAction, RunRecord, RunResult, Status,
};

// Index types
pub use crate::index::{Index, IndexEntry};

// Configuration
pub use crate::config::Config;

// Discovery functions
pub use crate::discovery::{
    archive_path_for_bean, find_archived_bean, find_bean_file, find_beans_dir,
};

// Graph functions
pub use crate::graph::{
    build_dependency_tree, build_full_graph, count_subtree_attempts, detect_cycle, find_all_cycles,
};

// Utility
pub use crate::bean::validate_priority;

// ---------------------------------------------------------------------------
// Query functions
// ---------------------------------------------------------------------------

/// Load a bean by ID.
///
/// Finds the bean file in the `.beans/` directory and deserializes it.
/// Works for both active and legacy bean formats.
///
/// # Errors
/// - Bean ID is invalid
/// - No bean file found for the given ID
/// - File cannot be parsed
pub fn get_bean(beans_dir: &Path, id: &str) -> Result<Bean> {
    let path = find_bean_file(beans_dir, id)?;
    Bean::from_file(&path)
}

/// Load a bean from the archive by ID.
///
/// # Errors
/// - Bean ID not found in archive
/// - File cannot be parsed
pub fn get_archived_bean(beans_dir: &Path, id: &str) -> Result<Bean> {
    let path = find_archived_bean(beans_dir, id)?;
    Bean::from_file(&path)
}

/// Load the index, rebuilding from bean files if stale.
///
/// This is the main entry point for reading bean metadata.
/// The index is a YAML cache that's faster than reading every bean file.
pub fn load_index(beans_dir: &Path) -> Result<Index> {
    Index::load_or_rebuild(beans_dir)
}

// ---------------------------------------------------------------------------
// Submodules (added as they are implemented)
// ---------------------------------------------------------------------------

// pub mod query;         // Phase 1: 88.2.2
// pub mod mutations;     // Phase 1: 88.2.5, 88.2.6, 88.2.7
// pub mod orchestration; // Phase 1: 88.2.4
