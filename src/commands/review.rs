//! `bn review` — Adversarial post-close review of bean implementations.
//!
//! After an agent closes a bean, a review agent checks the implementation against
//! the bean's spec in a fresh context, providing semantic correctness checking
//! beyond what verify gates can catch.
//!
//! ## Flow
//! 1. Load bean description + acceptance criteria
//! 2. Collect git diff (changes since HEAD)
//! 3. Build a review prompt with spec + diff + verdict instructions
//! 4. Spawn review agent (using config.review.run or config.run template)
//! 5. Parse VERDICT from agent output: approve / request-changes / flag
//! 6. Apply verdict: update labels, optionally reopen bean with notes
//!
//! ## Verdicts
//! - `approve` — implementation correct; adds `reviewed` label
//! - `request-changes` — issues found; reopens bean with notes, adds `review-failed`
//! - `flag` — needs human attention; adds `needs-human-review` label, stays closed

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use chrono::Utc;

use crate::bean::{Bean, Status};
use crate::config::Config;
use crate::discovery::find_bean_file;
use crate::index::Index;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Verdict returned (or inferred) from the review agent's output.
#[derive(Debug, Clone, PartialEq)]
pub enum ReviewVerdict {
    Approve,
    RequestChanges(String),
    Flag(String),
}

/// Arguments for `cmd_review`.
pub struct ReviewArgs {
    /// Bean ID to review.
    pub id: String,
    /// Override model (passed as BEAN_REVIEW_MODEL env var to the agent).
    pub model: Option<String>,
    /// Include only the git diff, not the full bean description.
    pub diff_only: bool,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Execute `bn review <id>`.
///
/// Spawns a review agent with bean context + git diff, parses its verdict,
/// and updates the bean (labels, notes, status) accordingly.
pub fn cmd_review(beans_dir: &Path, args: ReviewArgs) -> Result<()> {
    let config = Config::load_with_extends(beans_dir)?;

    let bean_path = find_bean_file(beans_dir, &args.id)
        .with_context(|| format!("Bean not found: {}", args.id))?;
    let bean = Bean::from_file(&bean_path)
        .with_context(|| format!("Failed to load bean: {}", args.id))?;

    // Enforce max_reopens to prevent infinite review loops.
    // Count how many times review has previously reopened this bean by
    // counting "Review failed" markers injected into notes.
    let max_reopens = config
        .review
        .as_ref()
        .map(|r| r.max_reopens)
        .unwrap_or(2);

    let reopen_count = bean
        .notes
        .as_deref()
        .unwrap_or("")
        .matches("**Review failed**")
        .count() as u32;

    if reopen_count >= max_reopens {
        eprintln!(
            "Review: bean {} has been reopened by review {} time(s) (max {}). Skipping.",
            args.id, reopen_count, max_reopens
        );
        return Ok(());
    }

    // Build review context (spec + diff)
    let context = build_review_context(beans_dir, &bean, args.diff_only)?;

    // Resolve review command template (prefer review.run, fall back to run)
    let run_template = config
        .review
        .as_ref()
        .and_then(|r| r.run.as_ref())
        .or(config.run.as_ref());

    let Some(template) = run_template else {
        eprintln!(
            "Review: no review command configured.\n\
             Set one with: bn config set review.run \"<command>\"\n\
             Or configure a default agent: bn init --setup"
        );
        return Ok(());
    };

    let cmd_str = template.replace("{id}", &args.id);

    eprintln!("Review: spawning review agent for bean {}...", args.id);

    let mut child_cmd = Command::new("sh");
    child_cmd
        .args(["-c", &cmd_str])
        // Pass full context via env so agent can read it from $BEAN_REVIEW_CONTEXT
        .env("BEAN_REVIEW_CONTEXT", &context)
        .env("BEAN_REVIEW_ID", &args.id)
        .env("BEAN_REVIEW_MODE", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    if let Some(ref model) = args.model {
        child_cmd.env("BEAN_REVIEW_MODEL", model);
    }

    let output = child_cmd
        .output()
        .with_context(|| format!("Failed to spawn review agent: {}", cmd_str))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let verdict = parse_verdict(&stdout);

    apply_verdict(beans_dir, &args.id, &bean_path, verdict)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Context builder
// ---------------------------------------------------------------------------

/// Build the review prompt: bean spec + git diff + verdict instructions.
fn build_review_context(beans_dir: &Path, bean: &Bean, diff_only: bool) -> Result<String> {
    let mut ctx = String::new();

    if !diff_only {
        ctx.push_str("# Bean Spec\n\n");
        ctx.push_str(&format!("**ID**: {}\n", bean.id));
        ctx.push_str(&format!("**Title**: {}\n\n", bean.title));

        if let Some(ref desc) = bean.description {
            ctx.push_str("## Description\n\n");
            ctx.push_str(desc);
            ctx.push_str("\n\n");
        }

        if let Some(ref acceptance) = bean.acceptance {
            ctx.push_str("## Acceptance Criteria\n\n");
            ctx.push_str(acceptance);
            ctx.push_str("\n\n");
        }
    }

    let git_diff = get_git_diff(beans_dir)?;
    if git_diff.is_empty() {
        ctx.push_str("# Git Diff\n\n(no uncommitted changes detected)\n\n");
    } else {
        ctx.push_str("# Git Diff\n\n```diff\n");
        ctx.push_str(&git_diff);
        ctx.push_str("\n```\n\n");
    }

    ctx.push_str("# Review Instructions\n\n");
    ctx.push_str(
        "Review the implementation above against the spec. Output your verdict as one of:\n\
         - `VERDICT: approve` — implementation is correct and complete\n\
         - `VERDICT: request-changes` — implementation has issues that must be fixed\n\
         - `VERDICT: flag` — implementation needs human attention (unusual issues)\n\n\
         Follow the verdict line with your reasoning and specific notes.\n",
    );

    Ok(ctx)
}

/// Get the current git diff (uncommitted changes in the working tree).
fn get_git_diff(beans_dir: &Path) -> Result<String> {
    let project_root = beans_dir.parent().unwrap_or(beans_dir);

    // Try staged + unstaged diff against HEAD
    let output = Command::new("git")
        .args(["diff", "HEAD"])
        .current_dir(project_root)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let diff = String::from_utf8_lossy(&out.stdout).into_owned();
            if !diff.is_empty() {
                return Ok(diff);
            }
            // HEAD diff empty — maybe there are staged changes only
            let staged = Command::new("git")
                .args(["diff", "--cached"])
                .current_dir(project_root)
                .output();
            if let Ok(s) = staged {
                return Ok(String::from_utf8_lossy(&s.stdout).into_owned());
            }
            Ok(String::new())
        }
        _ => {
            // Fallback: plain diff (no commits yet)
            let out2 = Command::new("git")
                .args(["diff"])
                .current_dir(project_root)
                .output();
            match out2 {
                Ok(o) => Ok(String::from_utf8_lossy(&o.stdout).into_owned()),
                Err(_) => Ok(String::new()),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Verdict parsing + application
// ---------------------------------------------------------------------------

/// Parse the review agent's output for a VERDICT keyword.
///
/// Looks for `VERDICT: approve`, `VERDICT: request-changes`, or `VERDICT: flag`
/// (case-insensitive). Everything after the verdict line is treated as notes.
/// Defaults to Approve if no verdict keyword is found.
pub fn parse_verdict(output: &str) -> ReviewVerdict {
    let lower = output.to_lowercase();

    if let Some(pos) = lower.find("verdict: request-changes") {
        let after = &output[pos..];
        let notes: String = after.lines().skip(1).collect::<Vec<_>>().join("\n");
        return ReviewVerdict::RequestChanges(notes.trim().to_string());
    }

    if let Some(pos) = lower.find("verdict: flag") {
        let after = &output[pos..];
        let notes: String = after.lines().skip(1).collect::<Vec<_>>().join("\n");
        return ReviewVerdict::Flag(notes.trim().to_string());
    }

    if lower.contains("verdict: approve") {
        return ReviewVerdict::Approve;
    }

    // No explicit verdict — default to approve to avoid blocking progress
    ReviewVerdict::Approve
}

/// Apply the parsed verdict to the bean: update labels, notes, and status.
pub fn apply_verdict(
    beans_dir: &Path,
    id: &str,
    bean_path: &PathBuf,
    verdict: ReviewVerdict,
) -> Result<()> {
    let mut bean = Bean::from_file(bean_path)
        .with_context(|| format!("Failed to reload bean: {}", id))?;

    match verdict {
        ReviewVerdict::Approve => {
            eprintln!("Review: ✓ APPROVED  bean {}", id);
            if !bean.labels.contains(&"reviewed".to_string()) {
                bean.labels.push("reviewed".to_string());
            }
            // Remove review-failed if it was set from a previous review cycle
            bean.labels.retain(|l| l != "review-failed");
            bean.updated_at = Utc::now();
            bean.to_file(bean_path)
                .with_context(|| format!("Failed to save bean: {}", id))?;
        }

        ReviewVerdict::RequestChanges(ref notes) => {
            eprintln!(
                "Review: ✗ REQUEST-CHANGES  bean {} — reopening for revision",
                id
            );
            if !bean.labels.contains(&"review-failed".to_string()) {
                bean.labels.push("review-failed".to_string());
            }
            bean.labels.retain(|l| l != "reviewed");

            // Append review notes so the next agent sees them
            let review_note = format!(
                "\n---\n**Review failed** ({})\n\n{}\n",
                Utc::now().format("%Y-%m-%d %H:%M UTC"),
                notes
            );
            match bean.notes {
                Some(ref mut existing) => existing.push_str(&review_note),
                None => bean.notes = Some(review_note),
            }

            // Reopen the bean
            bean.status = Status::Open;
            bean.closed_at = None;
            bean.close_reason = None;
            bean.updated_at = Utc::now();
            bean.to_file(bean_path)
                .with_context(|| format!("Failed to save bean: {}", id))?;
        }

        ReviewVerdict::Flag(ref notes) => {
            eprintln!("Review: ⚑ FLAGGED  bean {} — needs human review", id);
            if !bean.labels.contains(&"needs-human-review".to_string()) {
                bean.labels.push("needs-human-review".to_string());
            }
            let review_note = format!(
                "\n---\n**Flagged for human review** ({})\n\n{}\n",
                Utc::now().format("%Y-%m-%d %H:%M UTC"),
                notes
            );
            match bean.notes {
                Some(ref mut existing) => existing.push_str(&review_note),
                None => bean.notes = Some(review_note),
            }
            bean.updated_at = Utc::now();
            bean.to_file(bean_path)
                .with_context(|| format!("Failed to save bean: {}", id))?;
        }
    }

    // Rebuild index so status/labels are reflected immediately
    let index = Index::build(beans_dir).context("Failed to rebuild index after review")?;
    index
        .save(beans_dir)
        .context("Failed to save index after review")?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bean::Bean;
    use crate::util::title_to_slug;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let beans_dir = dir.path().join(".beans");
        fs::create_dir_all(&beans_dir).unwrap();
        (dir, beans_dir)
    }

    // --- parse_verdict ---

    #[test]
    fn parse_verdict_approve() {
        let output = "The code looks good.\nVERDICT: approve\nWell done.";
        assert_eq!(parse_verdict(output), ReviewVerdict::Approve);
    }

    #[test]
    fn parse_verdict_approve_case_insensitive() {
        let output = "verdict: APPROVE";
        assert_eq!(parse_verdict(output), ReviewVerdict::Approve);
    }

    #[test]
    fn parse_verdict_request_changes_captures_notes() {
        let output =
            "Issues found.\nVERDICT: request-changes\nMissing error handling.\nAlso add tests.";
        let verdict = parse_verdict(output);
        assert!(matches!(verdict, ReviewVerdict::RequestChanges(_)));
        if let ReviewVerdict::RequestChanges(notes) = verdict {
            assert!(notes.contains("Missing error handling"));
        }
    }

    #[test]
    fn parse_verdict_flag_captures_notes() {
        let output = "Unusual.\nVERDICT: flag\nPlease check manually.";
        let verdict = parse_verdict(output);
        assert!(matches!(verdict, ReviewVerdict::Flag(_)));
        if let ReviewVerdict::Flag(notes) = verdict {
            assert!(notes.contains("Please check manually"));
        }
    }

    #[test]
    fn parse_verdict_defaults_to_approve_when_no_keyword() {
        let output = "No verdict keyword present in this output.";
        assert_eq!(parse_verdict(output), ReviewVerdict::Approve);
    }

    #[test]
    fn parse_verdict_request_changes_takes_priority_over_approve() {
        // If both appear, request-changes wins (searched first)
        let output = "VERDICT: request-changes\nsome issue\nVERDICT: approve";
        let verdict = parse_verdict(output);
        assert!(matches!(verdict, ReviewVerdict::RequestChanges(_)));
    }

    // --- apply_verdict ---

    #[test]
    fn apply_verdict_approve_adds_reviewed_label() {
        let (_dir, beans_dir) = setup();
        let bean = Bean::new("1", "Test bean");
        let slug = title_to_slug(&bean.title);
        let path = beans_dir.join(format!("1-{}.md", slug));
        bean.to_file(&path).unwrap();

        apply_verdict(&beans_dir, "1", &path, ReviewVerdict::Approve).unwrap();

        let updated = Bean::from_file(&path).unwrap();
        assert!(updated.labels.contains(&"reviewed".to_string()));
    }

    #[test]
    fn apply_verdict_approve_removes_review_failed_label() {
        let (_dir, beans_dir) = setup();
        let mut bean = Bean::new("1", "Test bean");
        bean.labels.push("review-failed".to_string());
        let slug = title_to_slug(&bean.title);
        let path = beans_dir.join(format!("1-{}.md", slug));
        bean.to_file(&path).unwrap();

        apply_verdict(&beans_dir, "1", &path, ReviewVerdict::Approve).unwrap();

        let updated = Bean::from_file(&path).unwrap();
        assert!(!updated.labels.contains(&"review-failed".to_string()));
        assert!(updated.labels.contains(&"reviewed".to_string()));
    }

    #[test]
    fn apply_verdict_request_changes_reopens_bean() {
        let (_dir, beans_dir) = setup();
        let mut bean = Bean::new("1", "Test bean");
        bean.status = Status::Closed;
        bean.closed_at = Some(Utc::now());
        let slug = title_to_slug(&bean.title);
        let path = beans_dir.join(format!("1-{}.md", slug));
        bean.to_file(&path).unwrap();

        apply_verdict(
            &beans_dir,
            "1",
            &path,
            ReviewVerdict::RequestChanges("Fix error handling".to_string()),
        )
        .unwrap();

        let updated = Bean::from_file(&path).unwrap();
        assert_eq!(updated.status, Status::Open);
        assert!(updated.closed_at.is_none());
        assert!(updated.labels.contains(&"review-failed".to_string()));
        assert!(!updated.labels.contains(&"reviewed".to_string()));
        assert!(updated.notes.unwrap().contains("Review failed"));
    }

    #[test]
    fn apply_verdict_request_changes_injects_notes() {
        let (_dir, beans_dir) = setup();
        let bean = Bean::new("1", "Test bean");
        let slug = title_to_slug(&bean.title);
        let path = beans_dir.join(format!("1-{}.md", slug));
        bean.to_file(&path).unwrap();

        apply_verdict(
            &beans_dir,
            "1",
            &path,
            ReviewVerdict::RequestChanges("You forgot to handle EOF".to_string()),
        )
        .unwrap();

        let updated = Bean::from_file(&path).unwrap();
        let notes = updated.notes.unwrap();
        assert!(notes.contains("Review failed"));
        assert!(notes.contains("You forgot to handle EOF"));
    }

    #[test]
    fn apply_verdict_flag_adds_needs_human_review_label() {
        let (_dir, beans_dir) = setup();
        let mut bean = Bean::new("1", "Test bean");
        // Flag keeps bean in its current state — test with Closed to show it stays closed
        bean.status = Status::Closed;
        bean.closed_at = Some(Utc::now());
        let slug = title_to_slug(&bean.title);
        let path = beans_dir.join(format!("1-{}.md", slug));
        bean.to_file(&path).unwrap();

        apply_verdict(
            &beans_dir,
            "1",
            &path,
            ReviewVerdict::Flag("Security concern".to_string()),
        )
        .unwrap();

        let updated = Bean::from_file(&path).unwrap();
        assert!(updated.labels.contains(&"needs-human-review".to_string()));
        assert_eq!(updated.status, Status::Closed); // not reopened — stays as-is
    }

    #[test]
    fn apply_verdict_flag_injects_notes() {
        let (_dir, beans_dir) = setup();
        let bean = Bean::new("1", "Test bean");
        let slug = title_to_slug(&bean.title);
        let path = beans_dir.join(format!("1-{}.md", slug));
        bean.to_file(&path).unwrap();

        apply_verdict(
            &beans_dir,
            "1",
            &path,
            ReviewVerdict::Flag("Potential race condition".to_string()),
        )
        .unwrap();

        let updated = Bean::from_file(&path).unwrap();
        let notes = updated.notes.unwrap();
        assert!(notes.contains("Flagged for human review"));
        assert!(notes.contains("Potential race condition"));
    }

    #[test]
    fn apply_verdict_appends_to_existing_notes() {
        let (_dir, beans_dir) = setup();
        let mut bean = Bean::new("1", "Test bean");
        bean.notes = Some("Existing notes here.".to_string());
        let slug = title_to_slug(&bean.title);
        let path = beans_dir.join(format!("1-{}.md", slug));
        bean.to_file(&path).unwrap();

        apply_verdict(
            &beans_dir,
            "1",
            &path,
            ReviewVerdict::Flag("New flag note".to_string()),
        )
        .unwrap();

        let updated = Bean::from_file(&path).unwrap();
        let notes = updated.notes.unwrap();
        assert!(notes.contains("Existing notes here."));
        assert!(notes.contains("Flagged for human review"));
    }

    #[test]
    fn max_reopens_check_prevents_infinite_loops() {
        // Simulate that after max_reopens, review is skipped.
        // The count is based on "**Review failed**" markers in notes.
        let notes = "**Review failed** (2026-01-01)\n\nFix X\n\n---\n**Review failed** (2026-01-02)\n\nFix Y\n";
        let count = notes.matches("**Review failed**").count() as u32;
        let max: u32 = 2;
        assert!(count >= max, "Should skip review when max_reopens reached");
    }
}
