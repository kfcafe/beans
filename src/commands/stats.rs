use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::bean::{Bean, RunResult, Status};
use crate::index::Index;

// ---------------------------------------------------------------------------
// Output types (used for both text rendering and JSON serialization)
// ---------------------------------------------------------------------------

/// Cost and token statistics aggregated from RunRecord history.
#[derive(Debug, Serialize)]
pub struct CostStats {
    pub total_tokens: u64,
    pub total_cost: f64,
    pub avg_tokens_per_bean: f64,
    /// Rate at which closed beans passed on their first attempt (0.0–1.0).
    pub first_pass_rate: f64,
    /// Rate at which attempted beans eventually closed (0.0–1.0).
    pub overall_pass_rate: f64,
    pub most_expensive_bean: Option<BeanRef>,
    pub most_retried_bean: Option<BeanRef>,
    pub beans_with_history: usize,
}

/// Lightweight bean reference for reporting.
#[derive(Debug, Serialize)]
pub struct BeanRef {
    pub id: String,
    pub title: String,
    pub value: u64,
}

/// Machine-readable snapshot of all stats.
#[derive(Debug, Serialize)]
pub struct StatsOutput {
    pub total: usize,
    pub open: usize,
    pub in_progress: usize,
    pub closed: usize,
    pub blocked: usize,
    pub completion_pct: f64,
    pub priority_counts: [usize; 5],
    pub cost: Option<CostStats>,
}

// ---------------------------------------------------------------------------
// Bean file discovery
// ---------------------------------------------------------------------------

/// Returns all beans loaded from YAML files in `beans_dir` (non-recursive,
/// skips files that don't look like bean files or fail to parse).
fn load_all_beans(beans_dir: &Path) -> Vec<Bean> {
    let Ok(entries) = fs::read_dir(beans_dir) else {
        return vec![];
    };
    let mut beans = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if !is_bean_file(filename) {
            continue;
        }
        if let Ok(bean) = Bean::from_file(&path) {
            beans.push(bean);
        }
    }
    beans
}

/// Returns true for files that look like bean YAML files.
fn is_bean_file(filename: &str) -> bool {
    filename.ends_with(".yaml") || filename.ends_with(".md")
}

// ---------------------------------------------------------------------------
// Aggregation
// ---------------------------------------------------------------------------

fn aggregate_cost(beans: &[Bean]) -> Option<CostStats> {
    let mut total_tokens: u64 = 0;
    let mut total_cost: f64 = 0.0;
    let mut beans_with_history: usize = 0;

    // For first-pass rate: closed beans where first RunRecord result is Pass
    let mut closed_with_history: usize = 0;
    let mut first_pass_count: usize = 0;

    // For overall pass rate: closed / attempted (has any history)
    let mut attempted: usize = 0;
    let mut closed_count: usize = 0;

    // For most expensive and most retried
    let mut most_expensive: Option<(&Bean, u64)> = None;
    let mut most_retried: Option<(&Bean, usize)> = None;

    for bean in beans {
        if bean.history.is_empty() {
            continue;
        }

        beans_with_history += 1;
        attempted += 1;

        if bean.status == Status::Closed {
            closed_count += 1;
        }

        // Accumulate tokens/cost from all RunRecords
        let bean_tokens: u64 = bean.history.iter().filter_map(|r| r.tokens).sum();
        let bean_cost: f64 = bean.history.iter().filter_map(|r| r.cost).sum();

        total_tokens += bean_tokens;
        total_cost += bean_cost;

        // First-pass rate: closed beans where first RunRecord is a Pass
        if bean.status == Status::Closed {
            closed_with_history += 1;
            if bean
                .history
                .first()
                .map(|r| r.result == RunResult::Pass)
                .unwrap_or(false)
            {
                first_pass_count += 1;
            }
        }

        // Track most expensive (by total tokens across all attempts)
        if bean_tokens > 0 && most_expensive.is_none_or(|(_, t)| bean_tokens > t) {
            most_expensive = Some((bean, bean_tokens));
        }

        // Track most retried (by number of history entries)
        let attempt_count = bean.history.len();
        if attempt_count > 1 && most_retried.is_none_or(|(_, c)| attempt_count > c) {
            most_retried = Some((bean, attempt_count));
        }
    }

    // Don't show the section at all when nothing has been tracked
    if beans_with_history == 0 {
        return None;
    }

    let avg_tokens_per_bean = if beans_with_history > 0 {
        total_tokens as f64 / beans_with_history as f64
    } else {
        0.0
    };

    let first_pass_rate = if closed_with_history > 0 {
        first_pass_count as f64 / closed_with_history as f64
    } else {
        0.0
    };

    let overall_pass_rate = if attempted > 0 {
        closed_count as f64 / attempted as f64
    } else {
        0.0
    };

    Some(CostStats {
        total_tokens,
        total_cost,
        avg_tokens_per_bean,
        first_pass_rate,
        overall_pass_rate,
        most_expensive_bean: most_expensive.map(|(b, tokens)| BeanRef {
            id: b.id.clone(),
            title: b.title.clone(),
            value: tokens,
        }),
        most_retried_bean: most_retried.map(|(b, count)| BeanRef {
            id: b.id.clone(),
            title: b.title.clone(),
            value: count as u64,
        }),
        beans_with_history,
    })
}

// ---------------------------------------------------------------------------
// Command entry point
// ---------------------------------------------------------------------------

/// Show project statistics: counts by status, priority, and completion percentage.
/// When `--json` is passed, emits machine-readable JSON instead.
pub fn cmd_stats(beans_dir: &Path, json: bool) -> Result<()> {
    let index = Index::load_or_rebuild(beans_dir)?;

    // Count by status
    let total = index.beans.len();
    let open = index
        .beans
        .iter()
        .filter(|e| e.status == Status::Open)
        .count();
    let in_progress = index
        .beans
        .iter()
        .filter(|e| e.status == Status::InProgress)
        .count();
    let closed = index
        .beans
        .iter()
        .filter(|e| e.status == Status::Closed)
        .count();

    // Count blocked (open with unresolved dependencies)
    let blocked = index
        .beans
        .iter()
        .filter(|e| {
            if e.status != Status::Open {
                return false;
            }
            for dep_id in &e.dependencies {
                if let Some(dep) = index.beans.iter().find(|d| &d.id == dep_id) {
                    if dep.status != Status::Closed {
                        return true;
                    }
                } else {
                    return true;
                }
            }
            false
        })
        .count();

    // Count by priority
    let mut priority_counts = [0usize; 5];
    for entry in &index.beans {
        if (entry.priority as usize) < 5 {
            priority_counts[entry.priority as usize] += 1;
        }
    }

    // Calculate completion percentage
    let completion_pct = if total > 0 {
        (closed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    // Aggregate cost/token data from full bean files
    let all_beans = load_all_beans(beans_dir);
    let cost = aggregate_cost(&all_beans);

    if json {
        let output = StatsOutput {
            total,
            open,
            in_progress,
            closed,
            blocked,
            completion_pct,
            priority_counts,
            cost,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    // Human-readable output
    println!("=== Bean Statistics ===");
    println!();
    println!("Total:        {}", total);
    println!("Open:         {}", open);
    println!("In Progress:  {}", in_progress);
    println!("Closed:       {}", closed);
    println!("Blocked:      {}", blocked);
    println!();
    println!("Completion:   {:.1}%", completion_pct);
    println!();
    println!("By Priority:");
    println!("  P0: {}", priority_counts[0]);
    println!("  P1: {}", priority_counts[1]);
    println!("  P2: {}", priority_counts[2]);
    println!("  P3: {}", priority_counts[3]);
    println!("  P4: {}", priority_counts[4]);

    if let Some(c) = &cost {
        println!();
        println!("=== Tokens & Cost ===");
        println!();
        println!("Beans tracked:    {}", c.beans_with_history);
        println!("Total tokens:     {}", c.total_tokens);
        if c.total_cost > 0.0 {
            println!("Total cost:       ${:.4}", c.total_cost);
        }
        println!("Avg tokens/bean:  {:.0}", c.avg_tokens_per_bean);
        println!();
        println!("First-pass rate:  {:.1}%", c.first_pass_rate * 100.0);
        println!("Overall pass rate:{:.1}%", c.overall_pass_rate * 100.0);
        if let Some(ref bean) = c.most_expensive_bean {
            println!();
            println!(
                "Most expensive:   {} — {} ({} tokens)",
                bean.id, bean.title, bean.value
            );
        }
        if let Some(ref bean) = c.most_retried_bean {
            println!(
                "Most retried:     {} — {} ({} attempts)",
                bean.id, bean.title, bean.value
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bean::Bean;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_beans() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().unwrap();
        let beans_dir = dir.path().join(".beans");
        fs::create_dir(&beans_dir).unwrap();

        // Create beans with different statuses and priorities
        let mut b1 = Bean::new("1", "Open P0");
        b1.priority = 0;

        let mut b2 = Bean::new("2", "In Progress P1");
        b2.status = Status::InProgress;
        b2.priority = 1;

        let mut b3 = Bean::new("3", "Closed P2");
        b3.status = Status::Closed;
        b3.priority = 2;

        let mut b4 = Bean::new("4", "Open P3");
        b4.priority = 3;

        let mut b5 = Bean::new("5", "Open depends on 1");
        b5.dependencies = vec!["1".to_string()];

        b1.to_file(beans_dir.join("1.yaml")).unwrap();
        b2.to_file(beans_dir.join("2.yaml")).unwrap();
        b3.to_file(beans_dir.join("3.yaml")).unwrap();
        b4.to_file(beans_dir.join("4.yaml")).unwrap();
        b5.to_file(beans_dir.join("5.yaml")).unwrap();

        (dir, beans_dir)
    }

    #[test]
    fn stats_calculates_counts() {
        let (_dir, beans_dir) = setup_test_beans();
        let index = Index::load_or_rebuild(&beans_dir).unwrap();

        // Verify counts
        assert_eq!(
            index
                .beans
                .iter()
                .filter(|e| e.status == Status::Open)
                .count(),
            3
        ); // 1, 4, 5
        assert_eq!(
            index
                .beans
                .iter()
                .filter(|e| e.status == Status::InProgress)
                .count(),
            1
        ); // 2
        assert_eq!(
            index
                .beans
                .iter()
                .filter(|e| e.status == Status::Closed)
                .count(),
            1
        ); // 3
    }

    #[test]
    fn stats_command_works() {
        let (_dir, beans_dir) = setup_test_beans();
        let result = cmd_stats(&beans_dir, false);
        assert!(result.is_ok());
    }

    #[test]
    fn stats_command_json() {
        let (_dir, beans_dir) = setup_test_beans();
        let result = cmd_stats(&beans_dir, true);
        assert!(result.is_ok());
    }

    #[test]
    fn empty_project() {
        let dir = TempDir::new().unwrap();
        let beans_dir = dir.path().join(".beans");
        fs::create_dir(&beans_dir).unwrap();

        let result = cmd_stats(&beans_dir, false);
        assert!(result.is_ok());
    }

    #[test]
    fn aggregate_cost_no_history() {
        let beans = vec![Bean::new("1", "No history")];
        let result = aggregate_cost(&beans);
        assert!(
            result.is_none(),
            "Should return None when no beans have history"
        );
    }

    #[test]
    fn aggregate_cost_with_history() {
        use crate::bean::{RunRecord, RunResult};
        use chrono::Utc;

        let mut bean = Bean::new("1", "With history");
        bean.status = Status::Closed;
        bean.history = vec![RunRecord {
            attempt: 1,
            started_at: Utc::now(),
            finished_at: None,
            duration_secs: None,
            agent: None,
            result: RunResult::Pass,
            exit_code: Some(0),
            tokens: Some(1000),
            cost: Some(0.05),
            output_snippet: None,
        }];

        let stats = aggregate_cost(&[bean]).unwrap();
        assert_eq!(stats.total_tokens, 1000);
        assert!((stats.total_cost - 0.05).abs() < 1e-9);
        assert_eq!(stats.beans_with_history, 1);
        assert!((stats.first_pass_rate - 1.0).abs() < 1e-9);
        assert!((stats.overall_pass_rate - 1.0).abs() < 1e-9);
    }

    #[test]
    fn aggregate_cost_most_expensive_and_retried() {
        use crate::bean::{RunRecord, RunResult};
        use chrono::Utc;

        let make_record = |tokens: u64, result: RunResult| RunRecord {
            attempt: 1,
            started_at: Utc::now(),
            finished_at: None,
            duration_secs: None,
            agent: None,
            result,
            exit_code: None,
            tokens: Some(tokens),
            cost: None,
            output_snippet: None,
        };

        let mut cheap = Bean::new("1", "Cheap bean");
        cheap.history = vec![make_record(100, RunResult::Fail)];

        let mut expensive = Bean::new("2", "Expensive bean");
        expensive.history = vec![
            make_record(5000, RunResult::Fail),
            make_record(3000, RunResult::Pass),
        ];
        expensive.status = Status::Closed;

        let stats = aggregate_cost(&[cheap, expensive]).unwrap();
        assert_eq!(stats.total_tokens, 8100);
        let exp = stats.most_expensive_bean.unwrap();
        assert_eq!(exp.id, "2");
        assert_eq!(exp.value, 8000);

        let retried = stats.most_retried_bean.unwrap();
        assert_eq!(retried.id, "2");
        assert_eq!(retried.value, 2);
    }
}
