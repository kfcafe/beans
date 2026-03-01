use std::path::Path;

use anyhow::Result;

use crate::bean::Status;
use crate::config::Config;
use crate::index::{Index, IndexEntry};
use crate::stream::{self, StreamEvent};
use crate::tokens;

use super::BeanAction;
use super::ready_queue::all_deps_closed;
use super::wave::{compute_waves, Wave};

/// A bean with sizing and dispatch action.
#[derive(Debug, Clone)]
pub struct SizedBean {
    pub id: String,
    pub title: String,
    pub tokens: u64,
    pub action: BeanAction,
    pub priority: u8,
    pub dependencies: Vec<String>,
    pub parent: Option<String>,
    pub produces: Vec<String>,
    pub requires: Vec<String>,
    pub paths: Vec<String>,
}

/// Result from planning dispatch.
pub struct DispatchPlan {
    pub waves: Vec<Wave>,
    pub skipped: Vec<SizedBean>,
    /// Flat list of all beans to dispatch (for ready-queue mode).
    pub all_beans: Vec<SizedBean>,
    /// The index snapshot used for planning.
    pub index: Index,
}

/// Plan dispatch: get ready beans, size them, compute waves.
pub(super) fn plan_dispatch(
    beans_dir: &Path,
    config: &Config,
    filter_id: Option<&str>,
    auto_plan: bool,
    simulate: bool,
) -> Result<DispatchPlan> {
    let index = Index::load_or_rebuild(beans_dir)?;
    let workspace = beans_dir.parent().unwrap_or(Path::new("."));

    // Get beans to dispatch.
    // In simulate mode (dry-run), include all open beans with verify — even those
    // whose deps aren't met yet — so compute_waves can show the full execution plan.
    // In normal mode, only include beans whose deps are already closed.
    let mut ready_entries: Vec<&IndexEntry> = index
        .beans
        .iter()
        .filter(|e| {
            e.has_verify && e.status == Status::Open && (simulate || all_deps_closed(e, &index))
        })
        .collect();

    // Filter by ID if provided
    if let Some(filter_id) = filter_id {
        // Check if it's a parent — if so, get its ready children
        let is_parent = index
            .beans
            .iter()
            .any(|e| e.parent.as_deref() == Some(filter_id));
        if is_parent {
            ready_entries.retain(|e| e.parent.as_deref() == Some(filter_id));
        } else {
            ready_entries.retain(|e| e.id == filter_id);
        }
    }

    // Size each bean
    let mut sized: Vec<SizedBean> = Vec::new();
    for entry in &ready_entries {
        let bean_path = crate::discovery::find_bean_file(beans_dir, &entry.id)?;
        let bean = crate::bean::Bean::from_file(&bean_path)?;
        let token_count = tokens::calculate_tokens(&bean, workspace);
        let action = if token_count > config.max_tokens as u64 {
            BeanAction::Plan
        } else {
            BeanAction::Implement
        };

        sized.push(SizedBean {
            id: entry.id.clone(),
            title: entry.title.clone(),
            tokens: token_count,
            action,
            priority: entry.priority,
            dependencies: entry.dependencies.clone(),
            parent: entry.parent.clone(),
            produces: entry.produces.clone(),
            requires: entry.requires.clone(),
            paths: bean.paths.clone(),
        });
    }

    // Separate: implement beans go into waves; plan beans go to skipped (unless auto_plan)
    let (implement_beans, plan_beans): (Vec<SizedBean>, Vec<SizedBean>) = sized
        .into_iter()
        .partition(|sb| sb.action == BeanAction::Implement);

    let skipped = if auto_plan {
        // Include plan beans in waves too (they use the plan template)
        Vec::new()
    } else {
        plan_beans.clone()
    };

    let dispatch_beans = if auto_plan {
        let mut all = implement_beans;
        all.extend(plan_beans);
        all
    } else {
        implement_beans
    };

    let waves = compute_waves(&dispatch_beans, &index);

    Ok(DispatchPlan {
        waves,
        skipped,
        all_beans: dispatch_beans,
        index,
    })
}

/// Print the dispatch plan without executing.
pub(super) fn print_plan(plan: &DispatchPlan) {
    for (wave_idx, wave) in plan.waves.iter().enumerate() {
        println!("Wave {}: {} bean(s)", wave_idx + 1, wave.beans.len());
        for sb in &wave.beans {
            println!(
                "  {}  {}  {}  ({}k tokens)",
                sb.id,
                sb.title,
                sb.action,
                sb.tokens / 1000
            );
        }
    }

    if !plan.skipped.is_empty() {
        println!();
        println!("Skipped ({} — need planning):", plan.skipped.len());
        for sb in &plan.skipped {
            println!(
                "  ⚠ {}  {}  ({}k tokens)",
                sb.id,
                sb.title,
                sb.tokens / 1000
            );
        }
    }
}

/// Print the dispatch plan as JSON stream events.
pub(super) fn print_plan_json(plan: &DispatchPlan, parent_id: Option<&str>) {
    let parent_id = parent_id.unwrap_or("all").to_string();
    let rounds: Vec<stream::RoundPlan> = plan
        .waves
        .iter()
        .enumerate()
        .map(|(i, wave)| stream::RoundPlan {
            round: i + 1,
            beans: wave
                .beans
                .iter()
                .map(|b| stream::BeanInfo {
                    id: b.id.clone(),
                    title: b.title.clone(),
                    round: i + 1,
                })
                .collect(),
        })
        .collect();

    stream::emit(&StreamEvent::DryRun { parent_id, rounds });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn make_beans_dir() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().unwrap();
        let beans_dir = dir.path().join(".beans");
        fs::create_dir(&beans_dir).unwrap();
        (dir, beans_dir)
    }

    fn write_config(beans_dir: &Path, run: Option<&str>) {
        let run_line = match run {
            Some(r) => format!("run: \"{}\"\n", r),
            None => String::new(),
        };
        fs::write(
            beans_dir.join("config.yaml"),
            format!("project: test\nnext_id: 1\n{}", run_line),
        )
        .unwrap();
    }

    #[test]
    fn plan_dispatch_no_ready_beans() {
        let (_dir, beans_dir) = make_beans_dir();
        write_config(&beans_dir, Some("echo {id}"));

        let config = Config::load_with_extends(&beans_dir).unwrap();
        let plan = plan_dispatch(&beans_dir, &config, None, false, false).unwrap();

        assert!(plan.waves.is_empty());
        assert!(plan.skipped.is_empty());
    }

    #[test]
    fn plan_dispatch_returns_ready_beans() {
        let (_dir, beans_dir) = make_beans_dir();
        write_config(&beans_dir, Some("echo {id}"));

        let mut bean = crate::bean::Bean::new("1", "Task one");
        bean.verify = Some("echo ok".to_string());
        bean.to_file(beans_dir.join("1-task-one.md")).unwrap();

        let mut bean2 = crate::bean::Bean::new("2", "Task two");
        bean2.verify = Some("echo ok".to_string());
        bean2.to_file(beans_dir.join("2-task-two.md")).unwrap();

        let config = Config::load_with_extends(&beans_dir).unwrap();
        let plan = plan_dispatch(&beans_dir, &config, None, false, false).unwrap();

        assert_eq!(plan.waves.len(), 1);
        assert_eq!(plan.waves[0].beans.len(), 2);
    }

    #[test]
    fn plan_dispatch_filters_by_id() {
        let (_dir, beans_dir) = make_beans_dir();
        write_config(&beans_dir, Some("echo {id}"));

        let mut bean = crate::bean::Bean::new("1", "Task one");
        bean.verify = Some("echo ok".to_string());
        bean.to_file(beans_dir.join("1-task-one.md")).unwrap();

        let mut bean2 = crate::bean::Bean::new("2", "Task two");
        bean2.verify = Some("echo ok".to_string());
        bean2.to_file(beans_dir.join("2-task-two.md")).unwrap();

        let config = Config::load_with_extends(&beans_dir).unwrap();
        let plan = plan_dispatch(&beans_dir, &config, Some("1"), false, false).unwrap();

        assert_eq!(plan.waves.len(), 1);
        assert_eq!(plan.waves[0].beans.len(), 1);
        assert_eq!(plan.waves[0].beans[0].id, "1");
    }

    #[test]
    fn plan_dispatch_parent_id_gets_children() {
        let (_dir, beans_dir) = make_beans_dir();
        write_config(&beans_dir, Some("echo {id}"));

        let parent = crate::bean::Bean::new("1", "Parent");
        parent.to_file(beans_dir.join("1-parent.md")).unwrap();

        let mut child1 = crate::bean::Bean::new("1.1", "Child one");
        child1.parent = Some("1".to_string());
        child1.verify = Some("echo ok".to_string());
        child1.to_file(beans_dir.join("1.1-child-one.md")).unwrap();

        let mut child2 = crate::bean::Bean::new("1.2", "Child two");
        child2.parent = Some("1".to_string());
        child2.verify = Some("echo ok".to_string());
        child2.to_file(beans_dir.join("1.2-child-two.md")).unwrap();

        let config = Config::load_with_extends(&beans_dir).unwrap();
        let plan = plan_dispatch(&beans_dir, &config, Some("1"), false, false).unwrap();

        assert_eq!(plan.waves.len(), 1);
        assert_eq!(plan.waves[0].beans.len(), 2);
    }

    #[test]
    fn large_bean_classified_as_plan() {
        let (_dir, beans_dir) = make_beans_dir();
        // Use a very low max_tokens so our bean is "large"
        fs::write(
            beans_dir.join("config.yaml"),
            "project: test\nnext_id: 1\nrun: \"echo {id}\"\nmax_tokens: 1\n",
        )
        .unwrap();

        let mut bean = crate::bean::Bean::new(
            "1",
            "Large bean with lots of description text that should exceed the token limit",
        );
        bean.verify = Some("echo ok".to_string());
        bean.description = Some("x".repeat(1000));
        bean.to_file(beans_dir.join("1-large.md")).unwrap();

        let config = Config::load_with_extends(&beans_dir).unwrap();
        let plan = plan_dispatch(&beans_dir, &config, None, false, false).unwrap();

        // Should be skipped (needs planning)
        assert_eq!(plan.skipped.len(), 1);
        assert_eq!(plan.skipped[0].action, BeanAction::Plan);
    }

    #[test]
    fn auto_plan_includes_large_beans_in_waves() {
        let (_dir, beans_dir) = make_beans_dir();
        fs::write(
            beans_dir.join("config.yaml"),
            "project: test\nnext_id: 1\nrun: \"echo {id}\"\nmax_tokens: 1\n",
        )
        .unwrap();

        let mut bean = crate::bean::Bean::new("1", "Large bean");
        bean.verify = Some("echo ok".to_string());
        bean.description = Some("x".repeat(1000));
        bean.to_file(beans_dir.join("1-large.md")).unwrap();

        let config = Config::load_with_extends(&beans_dir).unwrap();
        let plan = plan_dispatch(&beans_dir, &config, None, true, false).unwrap();

        // With auto_plan, large beans go into waves, not skipped
        assert!(plan.skipped.is_empty());
        assert_eq!(plan.waves.len(), 1);
        assert_eq!(plan.waves[0].beans[0].action, BeanAction::Plan);
    }

    #[test]
    fn dry_run_simulate_shows_all_waves() {
        let (_dir, beans_dir) = make_beans_dir();
        write_config(&beans_dir, Some("echo {id}"));

        // Create a chain: 1.1 → 1.2 → 1.3 (parent=1)
        let parent = crate::bean::Bean::new("1", "Parent");
        parent.to_file(beans_dir.join("1-parent.md")).unwrap();

        let mut a = crate::bean::Bean::new("1.1", "Step A");
        a.parent = Some("1".to_string());
        a.verify = Some("echo ok".to_string());
        a.to_file(beans_dir.join("1.1-step-a.md")).unwrap();

        let mut b = crate::bean::Bean::new("1.2", "Step B");
        b.parent = Some("1".to_string());
        b.verify = Some("echo ok".to_string());
        b.dependencies = vec!["1.1".to_string()];
        b.to_file(beans_dir.join("1.2-step-b.md")).unwrap();

        let mut c = crate::bean::Bean::new("1.3", "Step C");
        c.parent = Some("1".to_string());
        c.verify = Some("echo ok".to_string());
        c.dependencies = vec!["1.2".to_string()];
        c.to_file(beans_dir.join("1.3-step-c.md")).unwrap();

        // Without simulate: only wave 1 (1.1) is ready
        let config = Config::load_with_extends(&beans_dir).unwrap();
        let plan = plan_dispatch(&beans_dir, &config, Some("1"), false, false).unwrap();
        assert_eq!(plan.waves.len(), 1);
        assert_eq!(plan.waves[0].beans.len(), 1);
        assert_eq!(plan.waves[0].beans[0].id, "1.1");

        // With simulate: all 3 waves shown
        let plan = plan_dispatch(&beans_dir, &config, Some("1"), false, true).unwrap();
        assert_eq!(plan.waves.len(), 3);
        assert_eq!(plan.waves[0].beans[0].id, "1.1");
        assert_eq!(plan.waves[1].beans[0].id, "1.2");
        assert_eq!(plan.waves[2].beans[0].id, "1.3");
    }

    #[test]
    fn dry_run_simulate_respects_produces_requires() {
        let (_dir, beans_dir) = make_beans_dir();
        write_config(&beans_dir, Some("echo {id}"));

        let parent = crate::bean::Bean::new("1", "Parent");
        parent.to_file(beans_dir.join("1-parent.md")).unwrap();

        let mut a = crate::bean::Bean::new("1.1", "Types");
        a.parent = Some("1".to_string());
        a.verify = Some("echo ok".to_string());
        a.produces = vec!["types".to_string()];
        a.to_file(beans_dir.join("1.1-types.md")).unwrap();

        let mut b = crate::bean::Bean::new("1.2", "Impl");
        b.parent = Some("1".to_string());
        b.verify = Some("echo ok".to_string());
        b.requires = vec!["types".to_string()];
        b.to_file(beans_dir.join("1.2-impl.md")).unwrap();

        // Without simulate: only 1.1 is ready (1.2 blocked on requires)
        let config = Config::load_with_extends(&beans_dir).unwrap();
        let plan = plan_dispatch(&beans_dir, &config, Some("1"), false, false).unwrap();
        assert_eq!(plan.waves.len(), 1);
        assert_eq!(plan.waves[0].beans[0].id, "1.1");

        // With simulate: both shown in correct wave order
        let plan = plan_dispatch(&beans_dir, &config, Some("1"), false, true).unwrap();
        assert_eq!(plan.waves.len(), 2);
        assert_eq!(plan.waves[0].beans[0].id, "1.1");
        assert_eq!(plan.waves[1].beans[0].id, "1.2");
    }
}
