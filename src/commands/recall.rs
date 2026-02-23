use std::path::Path;

use anyhow::Result;

use crate::bean::{Bean, Status};
use crate::discovery::{find_archived_bean, find_bean_file};
use crate::index::Index;

/// Search beans by substring matching (MVP — no embeddings).
///
/// Searches title, description, notes, close_reason, and paths.
/// Returns matching beans sorted by relevance (title match first, then recency).
pub fn cmd_recall(beans_dir: &Path, query: &str, all: bool, json: bool) -> Result<()> {
    let query_lower = query.to_lowercase();
    let index = Index::load_or_rebuild(beans_dir)?;

    let mut matches: Vec<(Bean, u32)> = Vec::new(); // (bean, score)

    // Search active beans
    for entry in &index.beans {
        if !all && entry.status == Status::Closed {
            continue;
        }

        let bean_path = match find_bean_file(beans_dir, &entry.id) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let bean = match Bean::from_file(&bean_path) {
            Ok(b) => b,
            Err(_) => continue,
        };

        if let Some(score) = score_match(&bean, &query_lower) {
            matches.push((bean, score));
        }
    }

    // Search archived beans too
    if all {
        let archived = Index::collect_archived(beans_dir).unwrap_or_default();
        for entry in &archived {
            let bean_path = match find_archived_bean(beans_dir, &entry.id) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let bean = match Bean::from_file(&bean_path) {
                Ok(b) => b,
                Err(_) => continue,
            };

            if let Some(score) = score_match(&bean, &query_lower) {
                matches.push((bean, score));
            }
        }
    }

    // Sort by score (descending), then by recency (descending)
    matches.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| b.0.updated_at.cmp(&a.0.updated_at))
    });

    if json {
        let results: Vec<serde_json::Value> = matches
            .iter()
            .map(|(bean, score)| {
                serde_json::json!({
                    "id": bean.id,
                    "title": bean.title,
                    "type": bean.bean_type,
                    "status": bean.status,
                    "score": score,
                    "close_reason": bean.close_reason,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        if matches.is_empty() {
            println!("No matches for \"{}\"", query);
            return Ok(());
        }

        println!("Found {} result(s) for \"{}\":\n", matches.len(), query);

        for (bean, _score) in &matches {
            let type_icon = if bean.bean_type == "fact" {
                "📌"
            } else {
                match bean.status {
                    Status::Closed => "✓",
                    Status::InProgress => "►",
                    Status::Open => "○",
                }
            };

            let status_str = match bean.status {
                Status::Closed => {
                    let reason = bean
                        .close_reason
                        .as_deref()
                        .unwrap_or("closed");
                    format!("({})", reason)
                }
                _ => format!("({})", bean.status),
            };

            println!(
                "  {} [{}] {} {}",
                type_icon, bean.id, bean.title, status_str
            );

            // Show failed attempts as negative memory
            let failed_attempts: Vec<_> = bean
                .attempt_log
                .iter()
                .filter(|a| a.outcome == crate::bean::AttemptOutcome::Failed)
                .collect();

            for attempt in &failed_attempts {
                if let Some(ref notes) = attempt.notes {
                    println!("    ⚠ Attempt #{} failed: {}", attempt.num, notes);
                }
            }

            // Show description preview
            if let Some(ref desc) = bean.description {
                let preview: String = desc.chars().take(120).collect();
                let preview = preview.lines().next().unwrap_or("");
                if !preview.is_empty() {
                    println!("    {}", preview);
                }
            }
        }
    }

    Ok(())
}

/// Score how well a bean matches a query. Returns None if no match.
fn score_match(bean: &Bean, query_lower: &str) -> Option<u32> {
    let mut score = 0u32;

    // Title match (highest weight)
    if bean.title.to_lowercase().contains(query_lower) {
        score += 10;
    }

    // Description match
    if let Some(ref desc) = bean.description {
        if desc.to_lowercase().contains(query_lower) {
            score += 5;
        }
    }

    // Notes match
    if let Some(ref notes) = bean.notes {
        if notes.to_lowercase().contains(query_lower) {
            score += 3;
        }
    }

    // Close reason match
    if let Some(ref reason) = bean.close_reason {
        if reason.to_lowercase().contains(query_lower) {
            score += 3;
        }
    }

    // Path match
    for path in &bean.paths {
        if path.to_lowercase().contains(query_lower) {
            score += 4;
            break;
        }
    }

    // Labels match
    for label in &bean.labels {
        if label.to_lowercase().contains(query_lower) {
            score += 2;
            break;
        }
    }

    // Attempt notes match (negative memory search)
    for attempt in &bean.attempt_log {
        if let Some(ref notes) = attempt.notes {
            if notes.to_lowercase().contains(query_lower) {
                score += 4;
                break;
            }
        }
    }

    if score > 0 {
        Some(score)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bean(id: &str, title: &str) -> Bean {
        Bean::new(id, title)
    }

    #[test]
    fn score_match_title() {
        let bean = make_bean("1", "Auth uses RS256");
        assert!(score_match(&bean, "rs256").is_some());
        assert!(score_match(&bean, "auth").is_some());
        assert!(score_match(&bean, "xyz").is_none());
    }

    #[test]
    fn score_match_description() {
        let mut bean = make_bean("1", "Config");
        bean.description = Some("Uses YAML format for configuration".to_string());
        assert!(score_match(&bean, "yaml").is_some());
    }

    #[test]
    fn score_match_paths() {
        let mut bean = make_bean("1", "Config");
        bean.paths = vec!["src/auth.rs".to_string()];
        assert!(score_match(&bean, "auth").is_some());
    }

    #[test]
    fn score_match_notes() {
        let mut bean = make_bean("1", "Task");
        bean.notes = Some("Blocked by database migration".to_string());
        assert!(score_match(&bean, "migration").is_some());
    }

    #[test]
    fn score_match_close_reason() {
        let mut bean = make_bean("1", "Task");
        bean.close_reason = Some("Superseded by new approach".to_string());
        assert!(score_match(&bean, "superseded").is_some());
    }

    #[test]
    fn title_scores_higher_than_description() {
        let mut bean = make_bean("1", "Auth module");
        bean.description = Some("Auth is important".to_string());

        let score = score_match(&bean, "auth").unwrap();
        // Title (10) + Description (5) = 15
        assert_eq!(score, 15);
    }
}
