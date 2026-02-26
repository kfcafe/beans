//! MCP resource definitions and handlers.

use std::path::Path;

use anyhow::{Context, Result};
use serde_json::json;

use crate::bean::Bean;
use crate::discovery::find_bean_file;
use crate::index::Index;
use crate::mcp::protocol::{ResourceContent, ResourceDefinition};

/// Return static resource definitions.
pub fn resource_definitions() -> Vec<ResourceDefinition> {
    vec![
        ResourceDefinition {
            uri: "beans://status".to_string(),
            name: "Project Status".to_string(),
            description: Some(
                "Current project status: claimed, ready, goals, and blocked beans".to_string(),
            ),
            mime_type: Some("application/json".to_string()),
        },
        ResourceDefinition {
            uri: "beans://rules".to_string(),
            name: "Project Rules".to_string(),
            description: Some("Project rules from RULES.md (if it exists)".to_string()),
            mime_type: Some("text/markdown".to_string()),
        },
    ]
}

/// Handle a resource read request.
pub fn handle_resource_read(uri: &str, beans_dir: &Path) -> Result<Vec<ResourceContent>> {
    if uri == "beans://status" {
        return read_status_resource(beans_dir);
    }

    if uri == "beans://rules" {
        return read_rules_resource(beans_dir);
    }

    // beans://bean/{id}
    if let Some(id) = uri.strip_prefix("beans://bean/") {
        return read_bean_resource(id, beans_dir);
    }

    anyhow::bail!("Unknown resource URI: {}", uri)
}

fn read_status_resource(beans_dir: &Path) -> Result<Vec<ResourceContent>> {
    let index = Index::load_or_rebuild(beans_dir)?;

    let mut claimed = 0u32;
    let mut ready = 0u32;
    let mut goals = 0u32;
    let mut blocked = 0u32;
    let mut closed = 0u32;

    for entry in &index.beans {
        match entry.status {
            crate::bean::Status::InProgress => claimed += 1,
            crate::bean::Status::Closed => closed += 1,
            crate::bean::Status::Open => {
                if entry.has_verify {
                    // Check if blocked
                    let is_blocked = entry.dependencies.iter().any(|dep_id| {
                        index
                            .beans
                            .iter()
                            .find(|e| &e.id == dep_id)
                            .map_or(true, |e| e.status != crate::bean::Status::Closed)
                    });
                    if is_blocked {
                        blocked += 1;
                    } else {
                        ready += 1;
                    }
                } else {
                    goals += 1;
                }
            }
        }
    }

    let text = serde_json::to_string_pretty(&json!({
        "total": index.beans.len(),
        "claimed": claimed,
        "ready": ready,
        "goals": goals,
        "blocked": blocked,
        "closed": closed,
    }))?;

    Ok(vec![ResourceContent {
        uri: "beans://status".to_string(),
        mime_type: Some("application/json".to_string()),
        text,
    }])
}

fn read_rules_resource(beans_dir: &Path) -> Result<Vec<ResourceContent>> {
    let project_dir = beans_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine project root"))?;

    let rules_path = project_dir.join("RULES.md");
    if !rules_path.exists() {
        return Ok(vec![ResourceContent {
            uri: "beans://rules".to_string(),
            mime_type: Some("text/markdown".to_string()),
            text: "No RULES.md found in project root.".to_string(),
        }]);
    }

    let text = std::fs::read_to_string(&rules_path).context("Failed to read RULES.md")?;

    Ok(vec![ResourceContent {
        uri: "beans://rules".to_string(),
        mime_type: Some("text/markdown".to_string()),
        text,
    }])
}

fn read_bean_resource(id: &str, beans_dir: &Path) -> Result<Vec<ResourceContent>> {
    crate::util::validate_bean_id(id)?;
    let bean_path = find_bean_file(beans_dir, id)?;
    let bean = Bean::from_file(&bean_path)?;

    let text = serde_json::to_string_pretty(&bean).context("Failed to serialize bean")?;

    Ok(vec![ResourceContent {
        uri: format!("beans://bean/{}", id),
        mime_type: Some("application/json".to_string()),
        text,
    }])
}
