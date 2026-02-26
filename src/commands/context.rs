use std::path::Path;

use anyhow::{Context, Result};

use crate::bean::Bean;
use crate::config::Config;
use crate::ctx_assembler::{assemble_context, extract_paths};
use crate::discovery::find_bean_file;

/// Load project rules from the configured rules file.
///
/// Returns `None` if the file doesn't exist or is empty.
/// Warns to stderr if the file is very large (>1000 lines).
fn load_rules(beans_dir: &Path) -> Option<String> {
    let config = Config::load(beans_dir).ok()?;
    let rules_path = config.rules_path(beans_dir);

    let content = std::fs::read_to_string(&rules_path).ok()?;
    let trimmed = content.trim();

    if trimmed.is_empty() {
        return None;
    }

    let line_count = content.lines().count();
    if line_count > 1000 {
        eprintln!(
            "Warning: RULES.md is very large ({} lines). Consider trimming it.",
            line_count
        );
    }

    Some(content)
}

/// Format rules content with delimiters for agent context injection.
fn format_rules_section(rules: &str) -> String {
    format!(
        "═══ PROJECT RULES ═══════════════════════════════════════════\n\
         {}\n\
         ═════════════════════════════════════════════════════════════\n\n",
        rules.trim_end()
    )
}

/// Assemble context for a bean from its description and referenced files.
///
/// Extracts file paths mentioned in the bean's description and outputs
/// the content of those files in a markdown format suitable for LLM prompts.
/// If a `.beans/RULES.md` file exists, its contents are prepended as a
/// "Project Rules" section before the bean-specific context.
pub fn cmd_context(beans_dir: &Path, id: &str, json: bool) -> Result<()> {
    let bean_path =
        find_bean_file(beans_dir, id).context(format!("Could not find bean with ID: {}", id))?;

    let bean = Bean::from_file(&bean_path).context(format!(
        "Failed to parse bean from: {}",
        bean_path.display()
    ))?;

    // Get the project directory (parent of beans_dir which is .beans)
    let project_dir = beans_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid .beans/ path: {}", beans_dir.display()))?;

    // Extract file paths from the bean description
    let description = bean.description.as_deref().unwrap_or("");
    let paths = extract_paths(description);

    // Load project rules (silently skipped if missing/empty)
    let rules = load_rules(beans_dir);

    if paths.is_empty() {
        if json {
            let mut obj = serde_json::json!({"id": id, "files": []});
            if let Some(ref rules_content) = rules {
                obj["rules"] = serde_json::Value::String(rules_content.clone());
            }
            println!("{}", obj);
        } else {
            // Still output rules even if no files referenced
            if let Some(ref rules_content) = rules {
                print!("{}", format_rules_section(rules_content));
            }
            eprintln!("No file paths found in bean description.");
            eprintln!("Tip: Reference files in description with paths like 'src/foo.rs' or 'src/commands/bar.rs'");
        }
        return Ok(());
    }

    if json {
        // Output as JSON: array of {path, content} objects
        let mut files = Vec::new();
        for path_str in &paths {
            let full_path = project_dir.join(path_str);
            let content = if full_path.exists() {
                std::fs::read_to_string(&full_path).unwrap_or_else(|_| "(read error)".to_string())
            } else {
                "(not found)".to_string()
            };
            files.push(serde_json::json!({
                "path": path_str,
                "exists": full_path.exists(),
                "content": content,
            }));
        }
        let mut obj = serde_json::json!({
            "id": id,
            "files": files,
        });
        if let Some(ref rules_content) = rules {
            obj["rules"] = serde_json::Value::String(rules_content.clone());
        }
        println!("{}", serde_json::to_string_pretty(&obj)?);
    } else {
        let mut output = String::new();

        // Prepend project rules if available
        if let Some(ref rules_content) = rules {
            output.push_str(&format_rules_section(rules_content));
        }

        // Assemble the bean-specific context from referenced files
        let context = assemble_context(paths, project_dir).context("Failed to assemble context")?;
        output.push_str(&context);

        // Output the assembled markdown to stdout
        print!("{}", output);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_env() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().unwrap();
        let beans_dir = dir.path().join(".beans");
        fs::create_dir(&beans_dir).unwrap();
        (dir, beans_dir)
    }

    #[test]
    fn context_with_no_paths_in_description() {
        let (dir, beans_dir) = setup_test_env();

        // Create a bean with no file paths in description
        let mut bean = crate::bean::Bean::new("1", "Test bean");
        bean.description = Some("A description with no file paths".to_string());
        let slug = crate::util::title_to_slug(&bean.title);
        let bean_path = beans_dir.join(format!("1-{}.md", slug));
        bean.to_file(&bean_path).unwrap();

        // Should succeed but print a tip
        let result = cmd_context(&beans_dir, "1", false);
        assert!(result.is_ok());
    }

    #[test]
    fn context_with_paths_in_description() {
        let (dir, beans_dir) = setup_test_env();
        let project_dir = dir.path();

        // Create a source file
        let src_dir = project_dir.join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("foo.rs"), "fn main() {}").unwrap();

        // Create a bean referencing the file
        let mut bean = crate::bean::Bean::new("1", "Test bean");
        bean.description = Some("Check src/foo.rs for implementation".to_string());
        let slug = crate::util::title_to_slug(&bean.title);
        let bean_path = beans_dir.join(format!("1-{}.md", slug));
        bean.to_file(&bean_path).unwrap();

        let result = cmd_context(&beans_dir, "1", false);
        assert!(result.is_ok());
    }

    #[test]
    fn context_bean_not_found() {
        let (_dir, beans_dir) = setup_test_env();

        let result = cmd_context(&beans_dir, "999", false);
        assert!(result.is_err());
    }
}
